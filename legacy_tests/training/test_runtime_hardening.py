import json
from pathlib import Path
from types import SimpleNamespace

from packages.training.training_runner import OvernightRunner, _within_run_caps
from packages.training.turn_solver import TurnSolverAdapter
from packages.training.worker import _boss_hp_progress_reward, _claim_worker_slot


def test_within_run_caps_requires_both_limits():
    assert _within_run_caps(0, 10, 0, 20) is True
    assert _within_run_caps(10, 10, 0, 20) is False
    assert _within_run_caps(0, 10, 20, 20) is False


def test_record_game_refreshes_live_status_and_dashboard_artifacts(tmp_path: Path):
    runner = OvernightRunner({
        "run_dir": str(tmp_path),
        "workers": 1,
        "max_games": 10,
        "sweep_configs": [{"name": "runtime_test"}],
    })
    runner._current_sweep_config = {"name": "runtime_test"}
    runner._phase_name = "collecting"
    runner._collect_games_target = 4
    runner._collect_games_completed = 1
    runner._sweep_games_completed = 2
    runner._trainer = SimpleNamespace(train_steps=7, buffer=[])

    runner._record_game({
        "seed": "RUNTIME1",
        "won": False,
        "floor": 6,
        "hp": 33,
        "max_hp": 70,
        "decisions": 12,
        "duration_s": 1.2,
        "combats": [],
        "events": [],
        "deck_changes": [],
        "deck_final": [],
        "relics_final": [],
        "path_choices": [],
        "card_picks": [],
    })

    status = json.loads((tmp_path / "status.json").read_text())
    assert status["total_games"] == 1
    assert status["sweep_phase"] == "collecting"
    assert status["collect_progress"] == "1/4"
    assert status["sweep_games"] == 2
    assert status["config_name"] == "runtime_test"

    recent_episodes = json.loads((tmp_path / "recent_episodes.json").read_text())
    assert recent_episodes[0]["seed"] == "RUNTIME1"
    assert json.loads((tmp_path / "floor_curve.json").read_text()) == [6]


class _NeverReadyAsyncResult:
    def ready(self):
        return False

    def get(self, timeout=0):
        raise AssertionError("get() should not be called for an unready result")


class _DummySeedPool:
    def __init__(self):
        self.results = []

    def record_result(self, seed, result):
        self.results.append((seed, result))


class _DummyTrainer:
    def __init__(self):
        self.buffer = []

    def add_transition(self, **kwargs):
        self.buffer.append(SimpleNamespace(**kwargs))


def test_collect_batch_exits_promptly_when_shutdown_requested(monkeypatch, tmp_path: Path):
    runner = OvernightRunner({
        "run_dir": str(tmp_path),
        "workers": 1,
        "max_games": 10,
        "sweep_configs": [{"name": "runtime_test"}],
    })

    def _trigger_shutdown(_seconds: float):
        runner._shutdown_requested = True

    monkeypatch.setattr("packages.training.training_runner.time.sleep", _trigger_shutdown)
    results = runner._collect_batch(
        ["SEED1"],
        [_NeverReadyAsyncResult()],
        _DummySeedPool(),
        _DummyTrainer(),
    )

    assert results == []


def test_claim_worker_slot_reuses_dead_pid():
    slot_registry = [111, 222]
    slot = _claim_worker_slot(slot_registry, 333, pid_alive=lambda pid: pid == 222)
    assert slot == 0
    assert slot_registry == [333, 222]


def test_claim_worker_slot_returns_none_when_all_slots_live():
    slot_registry = [111, 222]
    slot = _claim_worker_slot(slot_registry, 333, pid_alive=lambda _pid: True)
    assert slot is None
    assert slot_registry == [111, 222]


def test_boss_hp_progress_reward_only_applies_to_boss_summary():
    boss_reward = _boss_hp_progress_reward({
        "room_type": "boss",
        "boss_max_hp": 300,
        "boss_dmg_dealt": 150,
    })
    assert boss_reward > 0
    assert _boss_hp_progress_reward({
        "room_type": "monster",
        "boss_max_hp": 300,
        "boss_dmg_dealt": 150,
    }) == 0.0
    assert _boss_hp_progress_reward({
        "room_type": "boss",
        "boss_max_hp": 0,
        "boss_dmg_dealt": 150,
    }) == 0.0


def test_apply_room_type_budgets_updates_multi_turn_inner_solver():
    budgets = {
        "boss": (100.0, 1000, 1000.0),
    }
    adapter = TurnSolverAdapter(solver_budgets=budgets)
    engine = SimpleNamespace(
        state=SimpleNamespace(
            enemies=[SimpleNamespace(hp=300), SimpleNamespace(hp=150)],
        )
    )

    budget_ms, budget_nodes = adapter._apply_room_type_budgets(engine, "boss")

    assert budget_ms == adapter._solver.default_time_budget_ms
    assert budget_nodes == adapter._solver.default_node_budget
    assert budget_ms == adapter._multi_turn._solver.default_time_budget_ms
    assert budget_nodes == adapter._multi_turn._solver.default_node_budget
    assert budget_ms >= 100.0
    assert budget_nodes >= 1000
