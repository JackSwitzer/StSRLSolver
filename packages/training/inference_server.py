"""
Centralized MLX inference server for the two-model RL architecture.

Runs as a daemon thread in the main process, accepting inference requests
from worker processes via mp.Queue and batching them for efficient
MLX (or fallback CPU PyTorch) forward passes.

Architecture:
    - InferenceServer: daemon thread, batches requests, dispatches responses
    - InferenceClient: used in worker processes, blocks on per-request response
    - MLXStrategicBackend / TorchStrategicBackend: forward_batch implementations

Request/Response protocol uses per-worker queues to avoid contention.
Weight sync is non-blocking: main thread posts to control queue, server
applies on next iteration.

Usage (main process):
    server = InferenceServer(n_workers=8)
    server.start()
    server.sync_strategic_from_pytorch(model, version=1)

    # In each worker process (after spawn):
    InferenceClient.setup_worker(server.request_q, server.response_qs[slot], slot)
    client = get_client()
    result = client.infer_strategic(obs_np, n_actions=10)
"""

from __future__ import annotations

from collections import deque
from dataclasses import dataclass
import logging
import queue
import threading
import time
from typing import Dict, List, Optional, Tuple

import numpy as np

try:
    import mlx.core as mx

    MLX_AVAILABLE = True
except ImportError:
    MLX_AVAILABLE = False

logger = logging.getLogger(__name__)

# Module-level client installed by workers via InferenceClient.setup_worker()
_CLIENT: Optional["InferenceClient"] = None


@dataclass(frozen=True)
class StrategicModelConfig:
    """Serializable config describing the strategic policy architecture."""

    input_dim: int
    hidden_dim: int
    action_dim: int
    num_blocks: int

    @classmethod
    def from_model(cls, model: "StrategicNet") -> "StrategicModelConfig":  # noqa: F821
        return cls(
            input_dim=model.input_dim,
            hidden_dim=model.hidden_dim,
            action_dim=model.action_dim,
            num_blocks=model.num_blocks,
        )

    def to_dict(self) -> Dict[str, int]:
        return {
            "input_dim": self.input_dim,
            "hidden_dim": self.hidden_dim,
            "action_dim": self.action_dim,
            "num_blocks": self.num_blocks,
        }


@dataclass(frozen=True)
class StrategicWeightSync:
    """Versioned weight update for the centralized inference server."""

    version: int
    config: StrategicModelConfig
    state_dict: Dict[str, object]


def build_strategic_weight_sync(
    model: "StrategicNet",  # noqa: F821
    version: int,
) -> StrategicWeightSync:
    """Create a versioned, CPU-cloned weight payload from a PyTorch model."""
    state_dict = {k: v.detach().cpu().clone() for k, v in model.state_dict().items()}
    return StrategicWeightSync(
        version=version,
        config=StrategicModelConfig.from_model(model),
        state_dict=state_dict,
    )


def _copy_obs_into(row: np.ndarray, obs: object, input_dim: int) -> None:
    """Copy an observation into a preallocated row with pad/truncate semantics."""
    if obs is None:
        return

    obs_arr = np.asarray(obs, dtype=np.float32).reshape(-1)
    length = min(obs_arr.shape[0], input_dim)
    if length:
        row[:length] = obs_arr[:length]

# ──────────────────────────────────────────────────────────────
# Backend implementations
# ──────────────────────────────────────────────────────────────


class MLXStrategicBackend:
    """Wraps MLXStrategicNet for batched inference.

    Implements forward_batch(obs_batch, mask_batch) -> (logits, values).
    """

    def __init__(self, net: "MLXStrategicNet"):  # noqa: F821
        if not MLX_AVAILABLE:
            raise RuntimeError("MLX not available; use TorchStrategicBackend")
        self._net = net
        self.action_dim: int = net.action_dim
        self.version: int = 0

    def forward_batch(
        self,
        obs_batch: np.ndarray,
        mask_batch: np.ndarray,
    ) -> Tuple[np.ndarray, np.ndarray]:
        """Run batched forward pass.

        Args:
            obs_batch: float32 [N, input_dim]
            mask_batch: bool [N, action_dim]

        Returns:
            (logits [N, action_dim], values [N]) as numpy float32 arrays
        """
        logits, values = self._net.forward_batch(obs_batch, mask_batch)
        return logits, values

    @classmethod
    def from_state_dict(
        cls,
        state_dict: Dict[str, "torch.Tensor"],  # noqa: F821
        config: dict,
        version: int,
    ) -> "MLXStrategicBackend":
        """Build an MLXStrategicBackend from a PyTorch state_dict.

        Converts each tensor to MLX via numpy. Mirrors the weight-loading
        logic in MLXStrategicNet.from_pytorch().
        """
        from packages.training.mlx_inference import MLXStrategicNet

        net = MLXStrategicNet(
            input_dim=config["input_dim"],
            hidden_dim=config["hidden_dim"],
            action_dim=config["action_dim"],
            num_blocks=config["num_blocks"],
        )

        def _to_mx(t):
            return mx.array(t.detach().cpu().numpy())

        def _load_linear(mlx_linear, prefix):
            mlx_linear.weight = _to_mx(state_dict[f"{prefix}.weight"])
            if f"{prefix}.bias" in state_dict:
                mlx_linear.bias = _to_mx(state_dict[f"{prefix}.bias"])

        def _load_ln(mlx_ln, prefix):
            mlx_ln.weight = _to_mx(state_dict[f"{prefix}.weight"])
            mlx_ln.bias = _to_mx(state_dict[f"{prefix}.bias"])

        _load_linear(net.input_linear, "input_proj.0")
        _load_ln(net.input_norm, "input_proj.1")

        for i, block in enumerate(net.blocks):
            _load_linear(block.linear, f"trunk.{i}.linear")
            _load_ln(block.norm, f"trunk.{i}.norm")

        _load_linear(net.policy_1, "policy_head.0")
        _load_linear(net.policy_2, "policy_head.2")
        _load_linear(net.value_1, "value_head.0")
        _load_linear(net.value_2, "value_head.2")
        _load_linear(net.floor_1, "floor_head.0")
        _load_linear(net.floor_2, "floor_head.2")
        _load_linear(net.act_1, "act_head.0")
        _load_linear(net.act_2, "act_head.2")

        backend = cls(net)
        backend.version = version
        return backend


class TorchStrategicBackend:
    """Wraps CPU StrategicNet for batched inference.

    Fallback when MLX is unavailable (non-Apple Silicon or missing install).
    Runs on CPU to avoid MPS contention with training.
    """

    def __init__(self, net: "StrategicNet"):  # noqa: F821
        import torch

        self._net = net
        self._net.eval()
        # Force CPU to avoid interfering with MPS training in main thread
        self._net = self._net.cpu()
        self.action_dim: int = net.action_dim
        self.version: int = 0
        self._torch = torch

    def forward_batch(
        self,
        obs_batch: np.ndarray,
        mask_batch: np.ndarray,
    ) -> Tuple[np.ndarray, np.ndarray]:
        """Run batched CPU forward pass.

        Args:
            obs_batch: float32 [N, input_dim]
            mask_batch: bool [N, action_dim]

        Returns:
            (logits [N, action_dim], values [N]) as numpy float32 arrays
        """
        torch = self._torch
        with torch.no_grad():
            obs_t = torch.from_numpy(obs_batch).float()
            mask_t = torch.from_numpy(mask_batch).bool()
            out = self._net(obs_t, mask_t)
            logits = out["policy_logits"].numpy()
            values = out["value"].numpy()
        return logits, values

    @classmethod
    def from_state_dict(
        cls,
        state_dict: Dict[str, "torch.Tensor"],  # noqa: F821
        config: dict,
        version: int,
    ) -> "TorchStrategicBackend":
        """Build a TorchStrategicBackend from a state_dict."""
        import torch

        from packages.training.strategic_net import StrategicNet

        net = StrategicNet(
            input_dim=config["input_dim"],
            hidden_dim=config["hidden_dim"],
            action_dim=config["action_dim"],
            num_blocks=config["num_blocks"],
        )
        # Remap tensors to CPU
        cpu_state = {k: v.detach().cpu() for k, v in state_dict.items()}
        net.load_state_dict(cpu_state)
        net.eval()

        backend = cls(net)
        backend.version = version
        return backend


# ──────────────────────────────────────────────────────────────
# InferenceServer
# ──────────────────────────────────────────────────────────────


class InferenceServer:
    """Centralized inference server running as a daemon thread.

    Accepts requests from worker processes via a shared request_q, batches
    them across routes, runs MLX (or CPU PyTorch) forward passes, and
    scatters responses back to per-worker response queues.

    Weight sync is non-blocking: post to _control_q from main thread, applied
    at the top of each serve loop iteration.

    Args:
        n_workers: number of worker slots (sets len of response_qs)
        max_batch_size: maximum requests to batch before forward pass
        batch_timeout_ms: max wait (ms) to fill a batch
        use_mlx: if True and MLX available, prefer MLX backend
    """

    def __init__(
        self,
        n_workers: int,
        max_batch_size: int = 64,
        batch_timeout_ms: float = 5.0,
        use_mlx: bool = True,
    ):
        import multiprocessing as mp

        ctx = mp.get_context("spawn")

        self.n_workers = n_workers
        self.max_batch_size = max_batch_size
        self.batch_timeout_ms = batch_timeout_ms
        self.use_mlx = use_mlx and MLX_AVAILABLE

        # Shared request queue (all workers -> server)
        self.request_q: mp.Queue = ctx.Queue(maxsize=n_workers * max_batch_size * 2)

        # Per-worker response queues (server -> individual worker)
        self.response_qs: List[mp.Queue] = [ctx.Queue(maxsize=128) for _ in range(n_workers)]

        # Slot queue: pre-loaded with [0..n_workers-1] for worker initialization
        self.slot_q: mp.Queue = ctx.Queue()
        for i in range(n_workers):
            self.slot_q.put(i)

        # Control queue for weight syncs (main thread -> server thread)
        self._control_q: queue.SimpleQueue = queue.SimpleQueue()

        # Stop event
        self._stop = threading.Event()

        # Backend state (guarded by server thread only after start)
        self._strategic_backend: Optional[MLXStrategicBackend | TorchStrategicBackend] = None
        self._strategic_obs_buffer: Optional[np.ndarray] = None
        self._strategic_mask_buffer: Optional[np.ndarray] = None
        self._combat_obs_buffer: Optional[np.ndarray] = None
        self._combat_mask_buffer: Optional[np.ndarray] = None

        # Stats
        self._stats_lock = threading.Lock()
        self._ROLLING = 1000  # window size for rolling stats
        self._stats = {
            "total_requests": 0,
            "total_batches": 0,
            "batch_sizes": deque(maxlen=self._ROLLING),
            "queue_wait_ms": deque(maxlen=self._ROLLING),
            "forward_ms": deque(maxlen=self._ROLLING),
            "strategic_requests": 0,
            "combat_requests": 0,
            "errors": 0,
            "version": 0,
            "enqueued_version": 0,
            "applied_version": 0,
        }

        # Daemon thread
        self._thread = threading.Thread(
            target=self._serve_loop,
            name="InferenceServer",
            daemon=True,
        )

    # ------------------------------------------------------------------
    # Public API (main thread)
    # ------------------------------------------------------------------

    def start(self) -> None:
        """Start the server thread. Call once from main process."""
        logger.info(
            "InferenceServer starting (n_workers=%d, max_batch=%d, timeout=%.1fms, backend=%s)",
            self.n_workers,
            self.max_batch_size,
            self.batch_timeout_ms,
            "MLX" if self.use_mlx else "Torch-CPU",
        )
        self._thread.start()

    def stop(self) -> None:
        """Signal server thread to stop and wait for it."""
        self._stop.set()
        self._thread.join(timeout=5.0)
        if self._thread.is_alive():
            logger.warning("InferenceServer thread did not exit cleanly within 5s")

    def sync_strategic_from_pytorch(
        self,
        model: "StrategicNet",  # noqa: F821
        version: int,
    ) -> None:
        """Push updated strategic weights to the server. Non-blocking.

        Called from main thread after PPO train_batch(). The server applies
        the sync at the top of its next loop iteration.

        Args:
            model: trained StrategicNet (on any device)
            version: monotonic integer identifying this checkpoint
        """
        state_dict = {k: v.detach().cpu().clone() for k, v in model.state_dict().items()}
        config = {
            "input_dim": model.input_dim,
            "hidden_dim": model.hidden_dim,
            "action_dim": model.action_dim,
            "num_blocks": model.num_blocks,
        }
        self._control_q.put(("sync_strategic", version, config, state_dict))
        logger.debug("InferenceServer: enqueued strategic sync version=%d", version)

    def enqueue_strategic_weights(self, update: StrategicWeightSync) -> None:
        """Push updated strategic weights using a StrategicWeightSync payload. Non-blocking."""
        config = update.config.to_dict()
        self._control_q.put(("sync_strategic", update.version, config, update.state_dict))
        with self._stats_lock:
            self._stats["enqueued_version"] = update.version
        logger.debug("InferenceServer: enqueued strategic sync version=%d", update.version)

    def get_stats(self) -> dict:
        """Return a snapshot of server stats. Thread-safe."""
        with self._stats_lock:
            s = dict(self._stats)
            # Convert deques to summary stats (avoid serializing large rolling windows)
            s["avg_batch_size"] = sum(s["batch_sizes"]) / max(len(s["batch_sizes"]), 1) if s["batch_sizes"] else 0
            s["avg_queue_wait_ms"] = sum(s["queue_wait_ms"]) / max(len(s["queue_wait_ms"]), 1) if s["queue_wait_ms"] else 0
            s["avg_forward_ms"] = sum(s["forward_ms"]) / max(len(s["forward_ms"]), 1) if s["forward_ms"] else 0
        # Drop raw rolling windows from snapshot (not JSON-serializable, not needed externally)
        s.pop("batch_sizes", None)
        s.pop("queue_wait_ms", None)
        s.pop("forward_ms", None)
        return s

    # ------------------------------------------------------------------
    # Server thread internals
    # ------------------------------------------------------------------

    def _serve_loop(self) -> None:
        """Main server loop. Runs in daemon thread."""
        logger.info("InferenceServer thread started")

        while not self._stop.is_set():
            # Apply any pending weight syncs before touching requests
            self._apply_pending_syncs()

            # Block until first request (with short timeout so we can check stop/syncs)
            try:
                t_enqueue = time.perf_counter()
                first = self.request_q.get(timeout=0.050)
            except Exception:
                # queue.Empty or EOFError (workers died) — just loop
                continue

            t_dequeue = time.perf_counter()
            queue_wait_ms = (t_dequeue - t_enqueue) * 1000.0

            # Collect additional requests to fill batch
            batch = [first]
            deadline = time.perf_counter() + (self.batch_timeout_ms / 1000.0)

            while len(batch) < self.max_batch_size:
                remaining = deadline - time.perf_counter()
                if remaining <= 0:
                    break
                try:
                    batch.append(self.request_q.get(timeout=remaining))
                except Exception:
                    break

            # Dispatch batch
            self._dispatch_batch(batch, queue_wait_ms)

        logger.info("InferenceServer thread stopped")

    def _apply_pending_syncs(self) -> None:
        """Drain the control queue and apply all pending weight syncs."""
        while True:
            try:
                msg = self._control_q.get_nowait()
            except queue.Empty:
                break

            kind = msg[0]
            if kind == "sync_strategic":
                _, version, config, state_dict = msg
                self._do_sync_strategic(version, config, state_dict)
            else:
                logger.warning("InferenceServer: unknown control message kind=%r", kind)

    def _do_sync_strategic(
        self,
        version: int,
        config: dict,
        state_dict: dict,
    ) -> None:
        """Build a new backend from state_dict and swap it in.

        Keeps the old backend alive until the new one is ready so we never
        serve stale or uninitialized weights.
        """
        try:
            if self.use_mlx:
                new_backend = MLXStrategicBackend.from_state_dict(state_dict, config, version)
            else:
                new_backend = TorchStrategicBackend.from_state_dict(state_dict, config, version)

            self._strategic_backend = new_backend
            with self._stats_lock:
                self._stats["version"] = version

            logger.info(
                "InferenceServer: strategic backend synced version=%d (action_dim=%d)",
                version,
                new_backend.action_dim,
            )
        except Exception as exc:
            logger.error(
                "InferenceServer: failed to sync strategic weights version=%d: %s",
                version,
                exc,
                exc_info=True,
            )
            # Keep serving with whatever backend we had before

    def _dispatch_batch(self, batch: list, queue_wait_ms: float) -> None:
        """Group batch by route, forward, scatter responses."""
        # Partition by route
        strategic_reqs = [r for r in batch if r.get("route") == "strategic"]
        combat_reqs = [r for r in batch if r.get("route") == "combat"]
        unknown_reqs = [r for r in batch if r.get("route") not in ("strategic", "combat")]

        t_fwd_start = time.perf_counter()

        if strategic_reqs:
            self._forward_strategic(strategic_reqs)

        if combat_reqs:
            self._forward_combat(combat_reqs)

        for req in unknown_reqs:
            self._send_error(req, f"unknown route: {req.get('route')!r}")

        fwd_ms = (time.perf_counter() - t_fwd_start) * 1000.0

        # Update stats
        n = len(batch)
        with self._stats_lock:
            s = self._stats
            s["total_requests"] += n
            s["total_batches"] += 1
            s["strategic_requests"] += len(strategic_reqs)
            s["combat_requests"] += len(combat_reqs)
            # Rolling windows (deque handles maxlen automatically)
            s["batch_sizes"].append(n)
            s["queue_wait_ms"].append(queue_wait_ms)
            s["forward_ms"].append(fwd_ms)

    def _get_strategic_batch_buffers(
        self,
        input_dim: int,
        action_dim: int,
    ) -> tuple:
        """Return reusable numpy buffers sized for the current backend."""
        obs_shape = (self.max_batch_size, input_dim)
        if self._strategic_obs_buffer is None or self._strategic_obs_buffer.shape != obs_shape:
            self._strategic_obs_buffer = np.empty(obs_shape, dtype=np.float32)

        mask_shape = (self.max_batch_size, action_dim)
        if self._strategic_mask_buffer is None or self._strategic_mask_buffer.shape != mask_shape:
            self._strategic_mask_buffer = np.empty(mask_shape, dtype=np.bool_)

        return self._strategic_obs_buffer, self._strategic_mask_buffer

    def _forward_strategic(self, reqs: list) -> None:
        """Build batch tensor, forward pass, scatter responses."""
        if self._strategic_backend is None:
            for req in reqs:
                self._send_error(req, "strategic backend not initialized (no sync yet)")
            return

        backend = self._strategic_backend
        action_dim = backend.action_dim
        input_dim = backend._net.input_dim if hasattr(backend._net, "input_dim") else 260

        n = len(reqs)
        obs_buffer, mask_buffer = self._get_strategic_batch_buffers(input_dim, action_dim)
        obs_batch = obs_buffer[:n]
        mask_batch = mask_buffer[:n]
        obs_batch.fill(0.0)
        mask_batch.fill(False)

        for i, req in enumerate(reqs):
            obs = req.get("obs")
            _copy_obs_into(obs_batch[i], obs, input_dim)

            n_actions = req.get("n_actions")
            if n_actions is not None and n_actions > 0:
                mask_batch[i, : min(n_actions, action_dim)] = True
            else:
                # All valid — should not happen in practice but safe fallback
                mask_batch[i] = True

        try:
            logits_batch, values_batch = backend.forward_batch(obs_batch, mask_batch)
        except Exception as exc:
            logger.error("InferenceServer: strategic forward_batch failed: %s", exc, exc_info=True)
            with self._stats_lock:
                self._stats["errors"] += n
            for req in reqs:
                self._send_error(req, f"forward_batch error: {exc}")
            return

        version = backend.version
        for i, req in enumerate(reqs):
            resp = {
                "kind": "result",
                "req_id": req["req_id"],
                "ok": True,
                "route": "strategic",
                "version": version,
                "logits": logits_batch[i],
                "value": float(values_batch[i]),
                "error": None,
            }
            self._send_response(req["worker_slot"], resp)

    def _get_combat_batch_buffers(self, input_dim: int, action_dim: int) -> tuple:
        """Return reusable numpy buffers for combat route."""
        obs_shape = (self.max_batch_size, input_dim)
        if self._combat_obs_buffer is None or self._combat_obs_buffer.shape != obs_shape:
            self._combat_obs_buffer = np.empty(obs_shape, dtype=np.float32)

        mask_shape = (self.max_batch_size, action_dim)
        if self._combat_mask_buffer is None or self._combat_mask_buffer.shape != mask_shape:
            self._combat_mask_buffer = np.empty(mask_shape, dtype=np.bool_)

        return self._combat_obs_buffer, self._combat_mask_buffer

    def _forward_combat(self, reqs: list) -> None:
        """Combat value evaluation using the strategic model's value head.

        Reuses the same backend as strategic inference. Combat observations
        are encoded by combat_state_encoder and fed through the full network,
        but only the value output is returned.
        """
        if self._strategic_backend is None:
            for req in reqs:
                self._send_error(req, "combat backend not available (strategic model not synced)")
            return

        backend = self._strategic_backend
        action_dim = backend.action_dim
        input_dim = backend._net.input_dim if hasattr(backend._net, "input_dim") else 260

        n = len(reqs)

        obs_buf, mask_buf = self._get_combat_batch_buffers(input_dim, action_dim)
        obs_batch = obs_buf[:n]
        mask_batch = mask_buf[:n]
        obs_batch.fill(0.0)
        mask_batch.fill(True)  # All actions "valid" — we only use the value head

        for i, req in enumerate(reqs):
            obs = req.get("obs")
            _copy_obs_into(obs_batch[i], obs, input_dim)

        try:
            _logits_batch, values_batch = backend.forward_batch(obs_batch, mask_batch)
        except Exception as exc:
            logger.error("InferenceServer: combat forward_batch failed: %s", exc, exc_info=True)
            with self._stats_lock:
                self._stats["errors"] += n
            for req in reqs:
                self._send_error(req, f"combat forward_batch error: {exc}")
            return

        version = backend.version
        for i, req in enumerate(reqs):
            resp = {
                "kind": "result",
                "req_id": req["req_id"],
                "ok": True,
                "route": "combat",
                "version": version,
                "logits": None,
                "value": float(values_batch[i]),
                "error": None,
            }
            self._send_response(req["worker_slot"], resp)

    def _send_response(self, worker_slot: int, resp: dict) -> None:
        """Deliver a response to a worker's response queue."""
        if worker_slot < 0 or worker_slot >= len(self.response_qs):
            logger.error("InferenceServer: invalid worker_slot=%d", worker_slot)
            return
        try:
            self.response_qs[worker_slot].put_nowait(resp)
        except Exception as exc:
            logger.warning(
                "InferenceServer: failed to deliver to worker %d: %s(%s)",
                worker_slot,
                type(exc).__name__,
                exc,
            )

    def _send_error(self, req: dict, msg: str) -> None:
        """Send an error response back to the requesting worker."""
        with self._stats_lock:
            self._stats["errors"] += 1
        resp = {
            "kind": "error",
            "req_id": req.get("req_id", -1),
            "ok": False,
            "route": req.get("route", "unknown"),
            "version": -1,
            "logits": None,
            "value": None,
            "error": msg,
        }
        slot = req.get("worker_slot", -1)
        if slot >= 0:
            self._send_response(slot, resp)
        else:
            logger.warning("InferenceServer: error with no valid slot — %s", msg)


# ──────────────────────────────────────────────────────────────
# InferenceClient
# ──────────────────────────────────────────────────────────────


class InferenceClient:
    """Client used in worker processes to request inference from InferenceServer.

    Workers call InferenceClient.setup_worker() once after spawn to install
    a module-level client. Then any function can call get_client() to retrieve
    it and run inference.

    Thread safety: each worker process has its own instance. Multiple threads
    within a worker process sharing a client would need external locking — but
    our workers are single-threaded, so this is fine.
    """

    def __init__(
        self,
        request_q: "mp.Queue",
        response_q: "mp.Queue",
        worker_slot: int,
        timeout_s: float = 5.0,
    ):
        self.request_q = request_q
        self.response_q = response_q
        self.worker_slot = worker_slot
        self.timeout_s = timeout_s
        self._req_counter = 0

    # ------------------------------------------------------------------
    # Class method for worker initialization
    # ------------------------------------------------------------------

    @classmethod
    def setup_worker(
        cls,
        request_q: "mp.Queue",
        response_q: "mp.Queue",
        slot_id: int,
        timeout_s: float = 5.0,
    ) -> "InferenceClient":
        """Install a module-level client for this worker process.

        Call once per worker process, immediately after spawn.

        Args:
            request_q: shared queue from InferenceServer.request_q
            response_q: this worker's queue from InferenceServer.response_qs[slot_id]
            slot_id: index identifying this worker
            timeout_s: per-request timeout (default 5s)

        Returns:
            The installed InferenceClient instance.
        """
        global _CLIENT
        _CLIENT = cls(request_q, response_q, slot_id, timeout_s)
        logger.debug("InferenceClient installed for worker slot=%d", slot_id)
        return _CLIENT

    # ------------------------------------------------------------------
    # Inference methods
    # ------------------------------------------------------------------

    def infer_strategic(
        self,
        obs: np.ndarray,
        n_actions: int,
    ) -> Optional[dict]:
        """Request strategic inference. Blocks until response or timeout.

        Args:
            obs: float32 ndarray of shape (260,) — run state observation
            n_actions: number of valid actions (mask[:n_actions] = True)

        Returns:
            Response dict with keys: ok, logits, value, version, route
            Returns None on timeout or error so caller can fall back to heuristic.
        """
        req = self._build_request("strategic", obs=obs, n_actions=n_actions)
        return self._send_and_wait(req)

    def infer_combat(
        self,
        obs: np.ndarray,
        legal_indices: np.ndarray,
    ) -> Optional[dict]:
        """Request combat inference. Placeholder for future combat net.

        Args:
            obs: float32 ndarray (combat state observation)
            legal_indices: int array of legal action indices

        Returns:
            Response dict or None on timeout/error/not-implemented.
        """
        req = self._build_request("combat", obs=obs, legal_indices=legal_indices)
        return self._send_and_wait(req)

    # ------------------------------------------------------------------
    # Internals
    # ------------------------------------------------------------------

    def _build_request(self, route: str, **kwargs) -> dict:
        self._req_counter += 1
        req = {
            "kind": "infer",
            "req_id": self._req_counter,
            "worker_slot": self.worker_slot,
            "route": route,
            "obs": kwargs.get("obs"),
            "n_actions": kwargs.get("n_actions"),
            "legal_indices": kwargs.get("legal_indices"),
        }
        return req

    def _send_and_wait(self, req: dict) -> Optional[dict]:
        """Send request and block for response up to timeout_s.

        Returns the response dict, or None if timed out or an error occurred.
        """
        # Drain any stale responses from previous timed-out requests.
        # This prevents cascading timeout storms where old responses
        # permanently desync the queue.
        drained = 0
        while True:
            try:
                self.response_q.get_nowait()
                drained += 1
            except Exception:
                break
        if drained:
            logger.debug(
                "InferenceClient slot=%d: drained %d stale responses before req_id=%d",
                self.worker_slot, drained, req["req_id"],
            )

        try:
            self.request_q.put(req, timeout=self.timeout_s)
        except Exception as exc:
            logger.warning(
                "InferenceClient slot=%d: failed to enqueue request req_id=%d: %s",
                self.worker_slot,
                req["req_id"],
                exc,
            )
            return None

        t_start = time.perf_counter()
        deadline = t_start + self.timeout_s

        while True:
            remaining = deadline - time.perf_counter()
            if remaining <= 0:
                logger.warning(
                    "InferenceClient slot=%d: timeout waiting for req_id=%d",
                    self.worker_slot,
                    req["req_id"],
                )
                return None

            try:
                resp = self.response_q.get(timeout=min(remaining, 0.1))
            except Exception:
                # queue.Empty — keep waiting until deadline
                continue

            # Validate that this is our response (stale responses can arrive
            # if a previous request timed out but eventually got served)
            if resp.get("req_id") != req["req_id"]:
                logger.debug(
                    "InferenceClient slot=%d: stale response req_id=%d (expected %d), discarding",
                    self.worker_slot,
                    resp.get("req_id", -1),
                    req["req_id"],
                )
                continue

            if not resp.get("ok", False):
                logger.warning(
                    "InferenceClient slot=%d: error response req_id=%d route=%s: %s",
                    self.worker_slot,
                    req["req_id"],
                    req["route"],
                    resp.get("error"),
                )
                return None

            return resp


# ──────────────────────────────────────────────────────────────
# Module-level accessor
# ──────────────────────────────────────────────────────────────


def get_client() -> Optional[InferenceClient]:
    """Return the module-level InferenceClient for this worker process.

    Returns None if InferenceClient.setup_worker() has not been called.
    Workers should fall back to heuristic when this returns None.
    """
    return _CLIENT
