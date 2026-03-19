"""Tests for shared memory inference pipeline.

Validates the zero-copy shared memory buffers used for Apple Silicon
unified memory inference. Tests cover buffer creation, read/write cycles,
concurrent workers, cleanup, the SharedMemoryClient interface, and the
server-side poll loop.
"""

from __future__ import annotations

import multiprocessing as mp
import threading
import time

import numpy as np
import pytest

from packages.training.shared_inference import (
    SharedInferenceBuffers,
    SharedMemoryClient,
    SharedMemoryServerLoop,
)


# ---------------------------------------------------------------------------
# Buffer lifecycle
# ---------------------------------------------------------------------------


class TestSharedInferenceBuffers:
    def test_shared_buffer_creation(self):
        """Shared buffers can be created with correct shapes."""
        bufs = SharedInferenceBuffers(
            max_workers=4, input_dim=480, action_dim=512, session_id="test_create"
        )
        try:
            assert bufs.obs_buffer.shape == (4, 480)
            assert bufs.logits_buffer.shape == (4, 512)
            assert bufs.values_buffer.shape == (4,)
            assert bufs.request_flags.shape == (4,)
            assert bufs.response_flags.shape == (4,)
            assert bufs.n_actions.shape == (4,)
        finally:
            bufs.cleanup()

    def test_buffers_initialized_to_zero(self):
        """All buffers are zeroed on creation."""
        bufs = SharedInferenceBuffers(
            max_workers=2, input_dim=10, action_dim=8, session_id="test_zero"
        )
        try:
            np.testing.assert_array_equal(bufs.obs_buffer, 0)
            np.testing.assert_array_equal(bufs.logits_buffer, 0)
            np.testing.assert_array_equal(bufs.values_buffer, 0)
            np.testing.assert_array_equal(bufs.request_flags, 0)
            np.testing.assert_array_equal(bufs.response_flags, 0)
            np.testing.assert_array_equal(bufs.n_actions, 0)
        finally:
            bufs.cleanup()

    def test_write_read_cycle(self):
        """Worker writes obs, server reads, writes result, worker reads."""
        bufs = SharedInferenceBuffers(
            max_workers=2, input_dim=480, action_dim=512, session_id="test_rw"
        )
        try:
            # Simulate worker write
            obs = np.random.randn(480).astype(np.float32)
            bufs.obs_buffer[0] = obs
            bufs.n_actions[0] = 5
            bufs.request_flags[0] = 1

            # Simulate server read
            assert bufs.request_flags[0] == 1
            np.testing.assert_array_equal(bufs.obs_buffer[0], obs)

            # Simulate server write result
            logits = np.random.randn(512).astype(np.float32)
            bufs.logits_buffer[0] = logits
            bufs.values_buffer[0] = 0.5
            bufs.response_flags[0] = 1
            bufs.request_flags[0] = 0

            # Simulate worker read
            assert bufs.response_flags[0] == 1
            np.testing.assert_array_equal(bufs.logits_buffer[0], logits)
            assert bufs.values_buffer[0] == pytest.approx(0.5)
        finally:
            bufs.cleanup()

    def test_concurrent_workers(self):
        """Multiple workers can write simultaneously."""
        bufs = SharedInferenceBuffers(
            max_workers=4, input_dim=480, action_dim=512, session_id="test_conc"
        )
        try:
            for i in range(4):
                bufs.obs_buffer[i] = np.ones(480, dtype=np.float32) * i
                bufs.request_flags[i] = 1

            # All 4 requests visible
            pending = np.where(bufs.request_flags == 1)[0]
            assert len(pending) == 4

            # Verify each slot has distinct data
            for i in range(4):
                np.testing.assert_allclose(bufs.obs_buffer[i], float(i))
        finally:
            bufs.cleanup()

    def test_shm_names_roundtrip(self):
        """shm_names dict has all fields needed for worker attachment."""
        bufs = SharedInferenceBuffers(
            max_workers=4, input_dim=480, action_dim=512, session_id="test_names"
        )
        try:
            info = bufs.shm_names
            assert info["session_id"] == "test_names"
            assert info["max_workers"] == 4
            assert info["input_dim"] == 480
            assert info["action_dim"] == 512
        finally:
            bufs.cleanup()

    def test_attach_existing(self):
        """Worker can attach to existing shared memory (create=False)."""
        owner = SharedInferenceBuffers(
            max_workers=2, input_dim=10, action_dim=8, session_id="test_attach"
        )
        try:
            # Write data from owner
            owner.obs_buffer[0, :5] = [1, 2, 3, 4, 5]
            owner.request_flags[0] = 1

            # Attach from "worker" side
            worker = SharedInferenceBuffers(
                max_workers=2, input_dim=10, action_dim=8,
                create=False, session_id="test_attach",
            )
            try:
                # Worker sees owner's data
                assert worker.request_flags[0] == 1
                np.testing.assert_array_equal(worker.obs_buffer[0, :5], [1, 2, 3, 4, 5])

                # Worker writes response, owner sees it
                worker.response_flags[0] = 1
                assert owner.response_flags[0] == 1
            finally:
                worker.close()
        finally:
            owner.cleanup()

    def test_cleanup_idempotent(self):
        """Calling cleanup() twice does not raise."""
        bufs = SharedInferenceBuffers(
            max_workers=2, input_dim=10, action_dim=8, session_id="test_idempotent"
        )
        bufs.cleanup()
        bufs.cleanup()  # Should not raise


# ---------------------------------------------------------------------------
# SharedMemoryClient
# ---------------------------------------------------------------------------


class TestSharedMemoryClient:
    def test_client_infer_strategic(self):
        """Client writes obs, gets logits/value after simulated server response."""
        bufs = SharedInferenceBuffers(
            max_workers=2, input_dim=480, action_dim=512, session_id="test_client"
        )
        try:
            client = SharedMemoryClient(buffers=bufs, slot=0, timeout_s=2.0)

            # Server thread: wait for request, write response
            def fake_server():
                deadline = time.monotonic() + 2.0
                while bufs.request_flags[0] != 1:
                    if time.monotonic() > deadline:
                        return
                    time.sleep(0.001)
                # Write dummy logits/value
                bufs.logits_buffer[0] = np.arange(512, dtype=np.float32) * 0.01
                bufs.values_buffer[0] = 0.42
                bufs.response_flags[0] = 1
                bufs.request_flags[0] = 0

            server_thread = threading.Thread(target=fake_server)
            server_thread.start()

            obs = np.random.randn(480).astype(np.float32)
            result = client.infer_strategic(obs, n_actions=10)

            server_thread.join(timeout=3.0)

            assert result is not None
            assert result["ok"] is True
            assert result["route"] == "strategic"
            assert result["value"] == pytest.approx(0.42)
            assert result["logits"].shape == (512,)
        finally:
            bufs.cleanup()

    def test_client_timeout(self):
        """Client returns None when server does not respond."""
        bufs = SharedInferenceBuffers(
            max_workers=2, input_dim=10, action_dim=8, session_id="test_timeout"
        )
        try:
            client = SharedMemoryClient(buffers=bufs, slot=0, timeout_s=0.1)

            obs = np.zeros(10, dtype=np.float32)
            result = client.infer_strategic(obs, n_actions=3)

            assert result is None
            # Request flag should be cleared on timeout
            assert bufs.request_flags[0] == 0
        finally:
            bufs.cleanup()

    def test_client_obs_padding(self):
        """Short observations are zero-padded in the buffer."""
        bufs = SharedInferenceBuffers(
            max_workers=2, input_dim=20, action_dim=8, session_id="test_pad"
        )
        try:
            client = SharedMemoryClient(buffers=bufs, slot=0, timeout_s=0.05)

            # Send short obs
            obs = np.array([1.0, 2.0, 3.0], dtype=np.float32)
            # Will timeout (no server), but we can check the buffer was written correctly
            client.infer_strategic(obs, n_actions=2)

            np.testing.assert_array_equal(bufs.obs_buffer[0, :3], [1.0, 2.0, 3.0])
            np.testing.assert_array_equal(bufs.obs_buffer[0, 3:], 0.0)
        finally:
            bufs.cleanup()

    def test_client_combat_route(self):
        """infer_combat returns result with route='combat' and logits=None."""
        bufs = SharedInferenceBuffers(
            max_workers=2, input_dim=10, action_dim=8, session_id="test_combat"
        )
        try:
            client = SharedMemoryClient(buffers=bufs, slot=0, timeout_s=1.0)

            def fake_server():
                deadline = time.monotonic() + 1.0
                while bufs.request_flags[0] != 1:
                    if time.monotonic() > deadline:
                        return
                    time.sleep(0.001)
                bufs.logits_buffer[0] = np.zeros(8, dtype=np.float32)
                bufs.values_buffer[0] = 0.7
                bufs.response_flags[0] = 1
                bufs.request_flags[0] = 0

            t = threading.Thread(target=fake_server)
            t.start()

            result = client.infer_combat(
                np.zeros(10, dtype=np.float32),
                legal_indices=np.array([0, 1, 2]),
            )
            t.join(timeout=2.0)

            assert result is not None
            assert result["route"] == "combat"
            assert result["logits"] is None
            assert result["value"] == pytest.approx(0.7)
        finally:
            bufs.cleanup()

    def test_worker_slot_compatibility(self):
        """SharedMemoryClient has worker_slot attribute for compatibility."""
        bufs = SharedInferenceBuffers(
            max_workers=2, input_dim=10, action_dim=8, session_id="test_compat"
        )
        try:
            client = SharedMemoryClient(buffers=bufs, slot=1)
            assert client.worker_slot == 1
            assert client.slot == 1
        finally:
            bufs.cleanup()


# ---------------------------------------------------------------------------
# SharedMemoryServerLoop
# ---------------------------------------------------------------------------


class TestSharedMemoryServerLoop:
    def _make_fake_backend(self, action_dim):
        """Create a fake backend that returns deterministic results."""

        class FakeBackend:
            def __init__(self, ad):
                self.action_dim = ad

            def forward_batch(self, obs_batch, mask_batch):
                n = obs_batch.shape[0]
                logits = np.ones((n, self.action_dim), dtype=np.float32) * 0.1
                values = np.full(n, 0.99, dtype=np.float32)
                return logits, values

        return FakeBackend(action_dim)

    def test_poll_no_requests(self):
        """poll_and_dispatch returns False when no requests pending."""
        bufs = SharedInferenceBuffers(
            max_workers=2, input_dim=10, action_dim=8, session_id="test_poll_empty"
        )
        try:
            backend = self._make_fake_backend(8)
            loop = SharedMemoryServerLoop(
                buffers=bufs, get_backend=lambda: backend, batch_timeout_ms=0.0,
            )
            assert loop.poll_and_dispatch() is False
        finally:
            bufs.cleanup()

    def test_poll_single_request(self):
        """Server processes a single pending request."""
        bufs = SharedInferenceBuffers(
            max_workers=2, input_dim=10, action_dim=8, session_id="test_poll_one"
        )
        try:
            backend = self._make_fake_backend(8)
            loop = SharedMemoryServerLoop(
                buffers=bufs, get_backend=lambda: backend, batch_timeout_ms=0.0,
            )

            # Simulate worker request
            bufs.obs_buffer[0] = np.ones(10, dtype=np.float32)
            bufs.n_actions[0] = 3
            bufs.request_flags[0] = 1

            assert loop.poll_and_dispatch() is True
            assert loop.total_requests == 1
            assert loop.total_batches == 1

            # Response should be written
            assert bufs.response_flags[0] == 1
            assert bufs.request_flags[0] == 0
            assert bufs.values_buffer[0] == pytest.approx(0.99)
            np.testing.assert_allclose(bufs.logits_buffer[0], 0.1)
        finally:
            bufs.cleanup()

    def test_poll_batch_multiple(self):
        """Server batches multiple pending requests into one forward pass."""
        bufs = SharedInferenceBuffers(
            max_workers=4, input_dim=10, action_dim=8, session_id="test_poll_batch"
        )
        try:
            backend = self._make_fake_backend(8)
            loop = SharedMemoryServerLoop(
                buffers=bufs, get_backend=lambda: backend, batch_timeout_ms=0.0,
            )

            # 3 workers submit requests
            for i in range(3):
                bufs.obs_buffer[i] = np.ones(10, dtype=np.float32) * (i + 1)
                bufs.n_actions[i] = 5
                bufs.request_flags[i] = 1

            assert loop.poll_and_dispatch() is True
            assert loop.total_requests == 3
            assert loop.total_batches == 1

            # All 3 should have responses
            for i in range(3):
                assert bufs.response_flags[i] == 1
                assert bufs.request_flags[i] == 0

            # Slot 3 should be untouched
            assert bufs.response_flags[3] == 0
        finally:
            bufs.cleanup()

    def test_poll_no_backend(self):
        """Server clears requests when no backend is loaded."""
        bufs = SharedInferenceBuffers(
            max_workers=2, input_dim=10, action_dim=8, session_id="test_poll_noback"
        )
        try:
            loop = SharedMemoryServerLoop(
                buffers=bufs, get_backend=lambda: None, batch_timeout_ms=0.0,
            )

            bufs.request_flags[0] = 1
            assert loop.poll_and_dispatch() is False
            # Request flag cleared to prevent spin-lock
            assert bufs.request_flags[0] == 0
        finally:
            bufs.cleanup()

    def test_end_to_end_client_server(self):
        """Full round-trip: client submits, server loop processes, client reads."""
        bufs = SharedInferenceBuffers(
            max_workers=4, input_dim=480, action_dim=512, session_id="test_e2e"
        )
        try:
            backend = self._make_fake_backend(512)
            loop = SharedMemoryServerLoop(
                buffers=bufs, get_backend=lambda: backend, batch_timeout_ms=0.0,
            )
            client = SharedMemoryClient(buffers=bufs, slot=1, timeout_s=2.0)

            # Server runs in background thread
            stop_event = threading.Event()

            def server_thread():
                while not stop_event.is_set():
                    loop.poll_and_dispatch()
                    time.sleep(0.0005)

            t = threading.Thread(target=server_thread, daemon=True)
            t.start()

            try:
                obs = np.random.randn(480).astype(np.float32)
                result = client.infer_strategic(obs, n_actions=10)

                assert result is not None
                assert result["ok"] is True
                assert result["value"] == pytest.approx(0.99)
                assert result["logits"].shape == (512,)
            finally:
                stop_event.set()
                t.join(timeout=2.0)
        finally:
            bufs.cleanup()

    def test_multiple_sequential_requests(self):
        """Client can make multiple sequential requests on the same slot."""
        bufs = SharedInferenceBuffers(
            max_workers=2, input_dim=10, action_dim=8, session_id="test_seq"
        )
        try:
            backend = self._make_fake_backend(8)
            loop = SharedMemoryServerLoop(
                buffers=bufs, get_backend=lambda: backend, batch_timeout_ms=0.0,
            )
            client = SharedMemoryClient(buffers=bufs, slot=0, timeout_s=2.0)

            stop_event = threading.Event()

            def server_thread():
                while not stop_event.is_set():
                    loop.poll_and_dispatch()
                    time.sleep(0.0005)

            t = threading.Thread(target=server_thread, daemon=True)
            t.start()

            try:
                for i in range(5):
                    obs = np.ones(10, dtype=np.float32) * i
                    result = client.infer_strategic(obs, n_actions=3)
                    assert result is not None, f"Request {i} returned None"
                    assert result["ok"] is True

                assert loop.total_requests == 5
            finally:
                stop_event.set()
                t.join(timeout=2.0)
        finally:
            bufs.cleanup()


# ---------------------------------------------------------------------------
# Cross-process test
# ---------------------------------------------------------------------------


def _worker_process(shm_info, slot, obs_value, result_queue):
    """Run in a child process: attach to shared memory and make a request."""
    bufs = SharedInferenceBuffers(
        max_workers=shm_info["max_workers"],
        input_dim=shm_info["input_dim"],
        action_dim=shm_info["action_dim"],
        create=False,
        session_id=shm_info["session_id"],
    )
    client = SharedMemoryClient(buffers=bufs, slot=slot, timeout_s=3.0)
    obs = np.ones(shm_info["input_dim"], dtype=np.float32) * obs_value
    result = client.infer_strategic(obs, n_actions=5)
    if result is not None:
        result_queue.put({"slot": slot, "value": result["value"], "ok": result["ok"]})
    else:
        result_queue.put({"slot": slot, "value": None, "ok": False})
    bufs.close()


class TestCrossProcess:
    def test_cross_process_inference(self):
        """Workers in child processes communicate with server via shared memory."""
        bufs = SharedInferenceBuffers(
            max_workers=4, input_dim=10, action_dim=8, session_id="test_xproc"
        )
        try:

            class FakeBackend:
                action_dim = 8
                def forward_batch(self, obs_batch, mask_batch):
                    n = obs_batch.shape[0]
                    # Encode the mean of obs into value so we can verify
                    values = obs_batch.mean(axis=1)
                    logits = np.zeros((n, 8), dtype=np.float32)
                    return logits, values

            backend = FakeBackend()
            loop = SharedMemoryServerLoop(
                buffers=bufs, get_backend=lambda: backend, batch_timeout_ms=1.0,
            )

            # Server thread in main process
            stop_event = threading.Event()

            def server_fn():
                while not stop_event.is_set():
                    loop.poll_and_dispatch()
                    time.sleep(0.0005)

            server_t = threading.Thread(target=server_fn, daemon=True)
            server_t.start()

            # Spawn 2 child processes
            ctx = mp.get_context("spawn")
            result_q = ctx.Queue()
            info = bufs.shm_names

            workers = []
            for slot_id, obs_val in [(0, 1.0), (1, 2.0)]:
                p = ctx.Process(
                    target=_worker_process,
                    args=(info, slot_id, obs_val, result_q),
                )
                p.start()
                workers.append(p)

            # Collect results
            results = {}
            for _ in range(2):
                try:
                    r = result_q.get(timeout=10.0)
                    results[r["slot"]] = r
                except Exception:
                    pass

            for p in workers:
                p.join(timeout=5.0)

            stop_event.set()
            server_t.join(timeout=2.0)

            # Verify results
            assert len(results) == 2, f"Expected 2 results, got {len(results)}: {results}"
            assert results[0]["ok"] is True
            assert results[0]["value"] == pytest.approx(1.0)  # mean of all-1.0 obs
            assert results[1]["ok"] is True
            assert results[1]["value"] == pytest.approx(2.0)  # mean of all-2.0 obs
        finally:
            bufs.cleanup()
