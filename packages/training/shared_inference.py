"""Shared memory inference for Apple Silicon unified memory.

Uses multiprocessing.shared_memory to share observation/result arrays
between workers and the inference server with zero copies.

On Apple Silicon, CPU and GPU share the same physical RAM. The old
mp.Queue pipeline pickled numpy arrays through pipes, copying data
that already lived in the same address space. This module replaces
those queues with pre-allocated shared memory buffers and atomic
flag arrays for synchronization.

Architecture:
    - Pre-allocate shared arrays for N worker slots:
        - obs_buffer:      (N, input_dim)  float32 -- workers write observations
        - logits_buffer:   (N, action_dim) float32 -- server writes logits
        - values_buffer:   (N,)            float32 -- server writes values
        - request_flags:   (N,)            int32   -- worker sets 1 when ready
        - response_flags:  (N,)            int32   -- server sets 1 when done
        - n_actions:       (N,)            int32   -- valid action count per slot

    - Worker flow:
        1. Write observation to obs_buffer[slot]
        2. Set n_actions[slot]
        3. Set request_flags[slot] = 1  (atomic signal)
        4. Spin-wait on response_flags[slot] == 1
        5. Read logits_buffer[slot], values_buffer[slot]
        6. Clear response_flags[slot] = 0

    - Server flow (daemon thread):
        1. Scan request_flags for slots with value 1
        2. Batch all ready observations
        3. Run MLX forward pass on batch
        4. Write results to logits_buffer, values_buffer
        5. Set response_flags = 1 for completed slots
        6. Clear request_flags = 0 for completed slots
"""

from __future__ import annotations

import logging
import time
from multiprocessing import shared_memory
from typing import Optional, Tuple

import numpy as np

logger = logging.getLogger(__name__)

# Unique prefix -- kept short because macOS POSIX shm names are limited to 31 chars
_SHM_PREFIX = "si_"


class SharedInferenceBuffers:
    """Manages shared memory arrays for zero-copy inference.

    Allocates named shared memory blocks and wraps them as numpy arrays.
    Both the server (main process) and workers (child processes) can
    create views into the same physical memory.

    Args:
        max_workers: number of worker slots to pre-allocate
        input_dim: observation vector dimension (480 for RunStateEncoder)
        action_dim: action space dimension (512 for MODEL_ACTION_DIM)
        create: if True, allocate new shared memory; if False, attach to existing
        session_id: unique identifier to avoid collisions between runs
    """

    def __init__(
        self,
        max_workers: int,
        input_dim: int,
        action_dim: int,
        create: bool = True,
        session_id: str = "",
    ):
        self.max_workers = max_workers
        self.input_dim = input_dim
        self.action_dim = action_dim
        self._session_id = session_id
        self._prefix = f"{_SHM_PREFIX}{session_id}_" if session_id else _SHM_PREFIX
        self._owns_memory = create

        # Shared memory blocks (kept alive for lifetime of this object)
        self._shm_blocks: list[shared_memory.SharedMemory] = []

        # Allocate or attach to shared memory
        # Names kept short: macOS POSIX shm names max 31 chars
        self.obs_buffer = self._make_array(
            "ob", (max_workers, input_dim), np.float32, create
        )
        self.logits_buffer = self._make_array(
            "lg", (max_workers, action_dim), np.float32, create
        )
        self.values_buffer = self._make_array(
            "vl", (max_workers,), np.float32, create
        )
        self.request_flags = self._make_array(
            "rq", (max_workers,), np.int32, create
        )
        self.response_flags = self._make_array(
            "rs", (max_workers,), np.int32, create
        )
        self.n_actions = self._make_array(
            "na", (max_workers,), np.int32, create
        )

        # Zero all buffers on creation
        if create:
            self.obs_buffer[:] = 0
            self.logits_buffer[:] = 0
            self.values_buffer[:] = 0
            self.request_flags[:] = 0
            self.response_flags[:] = 0
            self.n_actions[:] = 0

        if create:
            logger.info(
                "SharedInferenceBuffers created: %d slots, input=%d, action=%d, "
                "total_bytes=%d",
                max_workers,
                input_dim,
                action_dim,
                sum(shm.size for shm in self._shm_blocks),
            )

    def _make_array(
        self,
        name: str,
        shape: tuple,
        dtype: np.dtype,
        create: bool,
    ) -> np.ndarray:
        """Create or attach to a named shared memory block, return numpy view."""
        full_name = f"{self._prefix}{name}"
        nbytes = int(np.prod(shape)) * np.dtype(dtype).itemsize

        if create:
            # Unlink any stale block with the same name
            try:
                old = shared_memory.SharedMemory(name=full_name, create=False)
                old.close()
                old.unlink()
            except FileNotFoundError:
                pass

            shm = shared_memory.SharedMemory(name=full_name, create=True, size=nbytes)
        else:
            shm = shared_memory.SharedMemory(name=full_name, create=False)

        self._shm_blocks.append(shm)
        return np.ndarray(shape, dtype=dtype, buffer=shm.buf)

    @property
    def shm_names(self) -> dict:
        """Return dict of shared memory block names for passing to workers."""
        return {
            "session_id": self._session_id,
            "max_workers": self.max_workers,
            "input_dim": self.input_dim,
            "action_dim": self.action_dim,
        }

    def cleanup(self) -> None:
        """Close and unlink all shared memory blocks.

        Call from the process that created the buffers (create=True).
        Worker processes should call close() instead.
        """
        for shm in self._shm_blocks:
            try:
                shm.close()
            except Exception:
                pass
            if self._owns_memory:
                try:
                    shm.unlink()
                except FileNotFoundError:
                    pass
        self._shm_blocks.clear()
        logger.debug("SharedInferenceBuffers cleaned up (owner=%s)", self._owns_memory)

    def close(self) -> None:
        """Close shared memory views without unlinking (for worker processes)."""
        for shm in self._shm_blocks:
            try:
                shm.close()
            except Exception:
                pass
        self._shm_blocks.clear()


class SharedMemoryClient:
    """Client for worker processes to submit inference requests via shared memory.

    Replaces InferenceClient's queue-based approach. The worker writes its
    observation directly into the shared buffer and spin-waits for the
    server to write results back.

    Args:
        buffers: SharedInferenceBuffers (attached in worker process)
        slot: worker slot index
        timeout_s: max seconds to wait for server response
    """

    def __init__(
        self,
        buffers: SharedInferenceBuffers,
        slot: int,
        timeout_s: float = 5.0,
    ):
        self.buffers = buffers
        self.slot = slot
        self.timeout_s = timeout_s
        self._req_counter = 0
        self.worker_slot = slot  # Compatibility with InferenceClient interface

    def infer_strategic(
        self,
        obs: np.ndarray,
        n_actions: int,
    ) -> Optional[dict]:
        """Submit a strategic inference request via shared memory.

        Args:
            obs: float32 ndarray, observation vector
            n_actions: number of valid actions

        Returns:
            Response dict compatible with InferenceClient format,
            or None on timeout.
        """
        self._req_counter += 1
        slot = self.slot
        bufs = self.buffers

        # 1. Write observation into shared buffer (zero-copy on same machine)
        obs_arr = np.asarray(obs, dtype=np.float32).ravel()
        length = min(obs_arr.shape[0], bufs.input_dim)
        bufs.obs_buffer[slot, :length] = obs_arr[:length]
        if length < bufs.input_dim:
            bufs.obs_buffer[slot, length:] = 0

        # 2. Set metadata
        bufs.n_actions[slot] = n_actions

        # 3. Signal request ready (atomic int32 write)
        bufs.request_flags[slot] = 1

        # 4. Spin-wait for response with backoff
        deadline = time.monotonic() + self.timeout_s
        spin_count = 0
        while bufs.response_flags[slot] != 1:
            spin_count += 1
            if time.monotonic() > deadline:
                # Timeout: clear request flag so server doesn't process stale data
                bufs.request_flags[slot] = 0
                logger.warning(
                    "SharedMemoryClient slot=%d: timeout after %.1fs (req #%d)",
                    slot,
                    self.timeout_s,
                    self._req_counter,
                )
                return None
            # Adaptive backoff: tight spin for first 100 iterations,
            # then yield to OS scheduler to avoid wasting CPU
            if spin_count > 100:
                time.sleep(0.0001)  # 100us yield

        # 5. Read results
        logits = bufs.logits_buffer[slot].copy()
        value = float(bufs.values_buffer[slot])

        # 6. Clear response flag for next request
        bufs.response_flags[slot] = 0

        return {
            "kind": "result",
            "req_id": self._req_counter,
            "ok": True,
            "route": "strategic",
            "version": -1,  # Server can set this if needed
            "logits": logits,
            "value": value,
            "error": None,
        }

    def infer_combat(
        self,
        obs: np.ndarray,
        legal_indices: np.ndarray,
    ) -> Optional[dict]:
        """Combat inference via shared memory.

        Uses the same buffer path as strategic inference but returns
        only the value head output.
        """
        # Use strategic path (server routes by presence of request)
        result = self.infer_strategic(obs, n_actions=len(legal_indices))
        if result is not None:
            result["route"] = "combat"
            result["logits"] = None  # Combat only uses value
        return result


class SharedMemoryServerLoop:
    """Server-side loop that polls shared memory for pending requests.

    Runs as a daemon thread alongside the existing InferenceServer.
    Polls request_flags at high frequency, batches ready requests,
    runs forward pass via the backend, and writes results back.

    Args:
        buffers: SharedInferenceBuffers (created by server)
        get_backend: callable returning the current strategic backend
        batch_timeout_ms: max wait to accumulate a batch
    """

    def __init__(
        self,
        buffers: SharedInferenceBuffers,
        get_backend: callable,
        batch_timeout_ms: float = 2.0,
    ):
        self.buffers = buffers
        self._get_backend = get_backend
        self.batch_timeout_ms = batch_timeout_ms

        # Reusable batch buffers (avoids allocation per batch)
        self._obs_batch: Optional[np.ndarray] = None
        self._mask_batch: Optional[np.ndarray] = None

        # Stats
        self.total_requests = 0
        self.total_batches = 0

    def poll_and_dispatch(self) -> bool:
        """Check for pending requests, batch them, run forward pass.

        Returns True if any requests were processed, False otherwise.
        Called from the server's main loop.
        """
        bufs = self.buffers

        # Scan for ready slots
        pending = np.where(bufs.request_flags == 1)[0]
        if len(pending) == 0:
            return False

        # Optional: brief wait to accumulate more requests
        if len(pending) < bufs.max_workers // 2:
            deadline = time.monotonic() + (self.batch_timeout_ms / 1000.0)
            while time.monotonic() < deadline:
                pending = np.where(bufs.request_flags == 1)[0]
                if len(pending) >= bufs.max_workers // 2:
                    break
                time.sleep(0.0001)  # 100us poll
            # Re-scan after wait
            pending = np.where(bufs.request_flags == 1)[0]
            if len(pending) == 0:
                return False

        backend = self._get_backend()
        if backend is None:
            # No model loaded yet -- clear requests to avoid spin-lock
            for slot in pending:
                bufs.request_flags[slot] = 0
            return False

        n = len(pending)
        input_dim = bufs.input_dim
        action_dim = backend.action_dim

        # Ensure batch buffers are allocated
        if self._obs_batch is None or self._obs_batch.shape[0] < n:
            self._obs_batch = np.empty((bufs.max_workers, input_dim), dtype=np.float32)
            self._mask_batch = np.empty(
                (bufs.max_workers, action_dim), dtype=np.bool_
            )

        obs_batch = self._obs_batch[:n]
        mask_batch = self._mask_batch[:n]

        # Gather observations from shared memory (single memcpy per slot)
        for i, slot in enumerate(pending):
            obs_batch[i] = bufs.obs_buffer[slot]
            na = int(bufs.n_actions[slot])
            mask_batch[i] = False
            if na > 0:
                mask_batch[i, : min(na, action_dim)] = True
            else:
                mask_batch[i] = True

        # Forward pass
        try:
            logits_batch, values_batch = backend.forward_batch(obs_batch, mask_batch)
        except Exception as exc:
            logger.error(
                "SharedMemoryServerLoop: forward_batch failed: %s", exc, exc_info=True
            )
            # Clear request flags so workers time out
            for slot in pending:
                bufs.request_flags[slot] = 0
            return False

        # Scatter results back to shared memory
        for i, slot in enumerate(pending):
            bufs.logits_buffer[slot] = logits_batch[i]
            bufs.values_buffer[slot] = values_batch[i]

        # Signal responses ready and clear request flags
        # Order matters: set response THEN clear request, so workers
        # see a consistent state.
        for slot in pending:
            bufs.response_flags[slot] = 1
            bufs.request_flags[slot] = 0

        self.total_requests += n
        self.total_batches += 1

        return True
