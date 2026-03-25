from __future__ import annotations

from pathlib import Path
from types import SimpleNamespace

import numpy as np
import pytest

import scripts.v3_train_concurrent as v3
from packages.training import training_config as cfg


def _write_traj(path: Path, value: float) -> None:
    obs = np.full((1, 480), value, dtype=np.float32)
    masks = np.ones((1, 256), dtype=np.bool_)
    actions = np.array([0], dtype=np.int32)
    floors = np.array([float(value)], dtype=np.float32)
    np.savez(
        path,
        obs=obs,
        masks=masks,
        actions=actions,
        final_floors=floors,
    )


def test_runtime_config_uses_shared_constants():
    runtime = v3.build_runtime_config()

    assert runtime.worker_count == cfg.TRAIN_WORKERS
    assert runtime.collection_batch_size == cfg.TRAIN_GAMES_PER_BATCH
    assert runtime.inference_batch_size == cfg.TRAIN_MAX_BATCH_INFERENCE
    assert runtime.inference_batch_timeout_ms == cfg.INFERENCE_BATCH_TIMEOUT_MS
    assert runtime.temperature == cfg.TEMPERATURE
    assert runtime.mcts_enabled is cfg.MCTS_COMBAT_ENABLED
    assert runtime.mcts_card_sims == cfg.MCTS_BUDGETS["card_pick"]
    assert runtime.solver_time_budget_ms == cfg.SOLVER_BUDGETS["monster"][0]
    assert runtime.training_batch_size == cfg.TRAIN_BATCH_SIZE
    assert runtime.training_epochs == cfg.TRAIN_PPO_EPOCHS


def test_build_play_one_game_args_uses_shared_runtime_and_explore_multiplier():
    runtime = v3.build_runtime_config()

    base_args = v3.build_play_one_game_args("seed-1", 12, runtime, explore_game=False)
    explore_args = v3.build_play_one_game_args("seed-2", 34, runtime, explore_game=True)

    assert base_args == (
        "seed-1",
        runtime.ascension,
        runtime.temperature,
        12,
        runtime.solver_time_budget_ms,
        runtime.strategic_search,
        runtime.mcts_enabled,
        runtime.mcts_card_sims,
    )
    assert explore_args[0] == "seed-2"
    assert explore_args[1] == runtime.ascension
    assert explore_args[2] == pytest.approx(runtime.temperature * runtime.explore_temp_multiplier)
    assert explore_args[3] == 34


def test_shared_state_snapshot_includes_real_gpu_and_inference_stats(monkeypatch):
    monkeypatch.setattr(v3, "_read_gpu_percent", lambda: 73)

    state = v3.SharedState()
    state.update_inference_stats(
        {
            "avg_batch_size": 7.5,
            "avg_queue_wait_ms": 1.25,
            "avg_forward_ms": 0.5,
            "total_batches": 3,
        }
    )

    snap = state.snapshot()

    assert snap["gpu_percent"] == 73
    assert snap["inference"]["avg_batch_size"] == 7.5
    assert snap["inference"]["avg_queue_wait_ms"] == 1.25
    assert snap["inference"]["avg_forward_ms"] == 0.5
    assert snap["inference"]["total_batches"] == 3


def test_trajectory_cache_only_loads_new_files(tmp_path, monkeypatch):
    first = tmp_path / "traj_001.npz"
    second = tmp_path / "traj_002.npz"
    _write_traj(first, 1.0)

    calls: list[str] = []
    real_loader = v3._load_trajectory_file

    def counting_loader(path: Path):
        calls.append(path.name)
        return real_loader(path)

    monkeypatch.setattr(v3, "_load_trajectory_file", counting_loader)

    cache = v3.TrajectoryCache(search_dirs=[tmp_path], max_transitions=10)

    data1 = cache.update()
    assert data1 is not None
    assert data1["n_files"] == 1
    assert data1["obs"].shape == (1, 480)
    assert calls == ["traj_001.npz"]

    _write_traj(second, 2.0)
    data2 = cache.update()
    assert data2 is not None
    assert data2["n_files"] == 2
    assert data2["obs"].shape == (2, 480)
    assert calls == ["traj_001.npz", "traj_002.npz"]


def test_collection_thread_passes_slot_registry_to_worker_pool(monkeypatch, tmp_path):
    captured: dict[str, object] = {}

    class DummyModel:
        def __init__(self, *args, **kwargs):
            pass

        def to(self, device):
            return self

        def load_state_dict(self, state_dict):
            captured["loaded_state_dict"] = state_dict

    class DummyServer:
        def __init__(self, *args, **kwargs):
            self.request_q = object()
            self.response_qs = [object()]
            self.slot_q = object()
            self.slot_registry = object()
            self.slot_registry_lock = object()
            self.shm_info = {"session_id": "dummy"}

        def sync_strategic_from_pytorch(self, model, version=0):
            captured["sync_version"] = version

        def start(self):
            captured["server_started"] = True

        def get_stats(self):
            return {"total_batches": 0}

        def stop(self):
            captured["server_stopped"] = True

    class DummyPool:
        def __init__(self, **kwargs):
            captured["pool_kwargs"] = kwargs

        def terminate(self):
            captured["pool_terminated"] = True

        def join(self):
            captured["pool_joined"] = True

    class DummyContext:
        def Pool(self, **kwargs):
            return DummyPool(**kwargs)

    monkeypatch.setattr(v3, "load_checkpoint_into_model", lambda *args, **kwargs: False)
    monkeypatch.setattr(v3.mp, "get_context", lambda _method: DummyContext())
    monkeypatch.setattr(v3.torch.backends.mps, "is_available", lambda: False)
    monkeypatch.setattr(v3, "_shutdown", SimpleNamespace(is_set=lambda: True, wait=lambda *_args, **_kwargs: None))

    import packages.training.inference_server as inference_server_mod
    import packages.training.seed_pool as seed_pool_mod
    import packages.training.strategic_net as strategic_net_mod

    monkeypatch.setattr(inference_server_mod, "InferenceServer", DummyServer)
    monkeypatch.setattr(seed_pool_mod, "SeedPool", lambda: SimpleNamespace())
    monkeypatch.setattr(strategic_net_mod, "StrategicNet", DummyModel)

    shared = v3.SharedState()
    runtime = v3.build_runtime_config()
    v3.collection_thread(shared, tmp_path / "traj", tmp_path / "combat", runtime)

    pool_kwargs = captured["pool_kwargs"]
    initargs = pool_kwargs["initargs"]
    server_request_q = initargs[0]
    server_response_qs = initargs[1]
    server_slot_q = initargs[2]
    server_shm = initargs[3]
    server_slot_registry = initargs[4]
    server_slot_registry_lock = initargs[5]

    assert pool_kwargs["processes"] == runtime.worker_count
    assert len(initargs) == 6
    assert server_request_q is not None
    assert len(server_response_qs) == 1
    assert server_slot_q is not None
    assert server_shm == {"session_id": "dummy"}
    assert server_slot_registry is not None
    assert server_slot_registry_lock is not None
    assert captured["server_started"] is True
    assert captured["pool_terminated"] is True
    assert captured["pool_joined"] is True
    assert captured["server_stopped"] is True
