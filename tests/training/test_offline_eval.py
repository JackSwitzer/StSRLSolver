"""Tests for packages.training.offline_eval."""
from __future__ import annotations
import inspect
from pathlib import Path
from unittest.mock import patch
from packages.training.offline_eval import evaluate_model, run_ab_test, _floor_distribution

class TestEvaluateModelSignature:
    def test_no_reward_config_param(self):
        sig = inspect.signature(evaluate_model)
        assert "reward_config" not in sig.parameters
    def test_has_required_params(self):
        sig = inspect.signature(evaluate_model)
        for p in ("model_path", "num_games", "workers", "ascension"):
            assert p in sig.parameters
    def test_label_param_exists(self):
        sig = inspect.signature(evaluate_model)
        assert "label" in sig.parameters

class TestFloorDistribution:
    def test_basic_buckets(self):
        dist = _floor_distribution([1, 5, 6, 10, 11, 15, 16, 17, 20])
        assert dist == {"F1-5": 2, "F6-10": 2, "F11-15": 2, "F16": 1, "F17+": 2}
    def test_empty(self):
        assert _floor_distribution([]) == {"F1-5": 0, "F6-10": 0, "F11-15": 0, "F16": 0, "F17+": 0}

def _fake(model_path, num_games=50, workers=4, label="eval", **kw):
    return {"config_name": label, "model_path": model_path, "games_played": num_games,
            "avg_floor": 10.0, "median_floor": 10.0, "max_floor": 16, "win_rate": 0.0,
            "wins": 0, "avg_hp": 40.0, "avg_hp_at_boss": None,
            "floor_distribution": {}, "elapsed_s": 1.0, "games_per_min": 3000.0}

class TestRunAbTest:
    @patch("packages.training.offline_eval.evaluate_model", side_effect=_fake)
    def test_result_structure(self, m, tmp_path):
        r = run_ab_test(model_path="/f.pt", config_names=["A_baseline", "B_split_milestones"],
                        num_games=10, workers=1, output_path=tmp_path / "r.json")
        assert "configs" in r and "A_baseline" in r["configs"] and (tmp_path / "r.json").exists()
    @patch("packages.training.offline_eval.evaluate_model", side_effect=_fake)
    def test_default_configs(self, m, tmp_path):
        r = run_ab_test(model_path="/f.pt", num_games=5, output_path=tmp_path / "r.json")
        assert len(r["configs"]) == 3
    def test_no_matching_configs(self, tmp_path):
        r = run_ab_test(model_path="/f.pt", config_names=["Z"], output_path=tmp_path / "r.json")
        assert r == {"error": "no_matching_configs"}

class TestErrorHandling:
    @patch("multiprocessing.Pool")
    def test_missing_model_returns_error(self, mock_pool):
        mock_pool.side_effect = RuntimeError("fail")
        r = evaluate_model(model_path="/bad.pt", num_games=1, workers=1, label="test")
        assert "error" in r and r["games_played"] == 0
