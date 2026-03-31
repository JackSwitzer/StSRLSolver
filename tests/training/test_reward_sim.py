"""Tests for reward simulation and offline evaluation."""

from __future__ import annotations

import tempfile
from pathlib import Path

import numpy as np
import pytest

from packages.training.reward_sim import (
    ALL_REWARD_CONFIGS,
    REWARD_CONFIG_A,
    REWARD_CONFIG_B,
    REWARD_CONFIG_C,
    REWARD_CONFIG_D,
    compare_configs,
    rescore_trajectory,
)
from packages.training.data_utils import TrajectoryData


def _make_trajectory(floor: int, T: int = 50) -> TrajectoryData:
    """Create a synthetic trajectory for testing."""
    return TrajectoryData(
        obs=np.random.randn(T, 480).astype(np.float32),
        masks=np.ones((T, 512), dtype=np.bool_),
        actions=np.zeros(T, dtype=np.int32),
        rewards=np.random.randn(T).astype(np.float32) * 0.1,
        dones=np.zeros(T, dtype=np.bool_),
        values=np.zeros(T, dtype=np.float32),
        log_probs=np.zeros(T, dtype=np.float32),
        final_floors=np.full(T, floor / 55.0, dtype=np.float32),
        cleared_act1=np.zeros(T, dtype=np.float32),
        floor=floor,
        source_path=Path(f"/tmp/traj_F{floor:02d}_test.npz"),
    )


class TestRescoreTrajectory:
    def test_different_configs_produce_different_returns(self):
        """Rescoring the same trajectory with different configs should yield different returns."""
        traj = _make_trajectory(floor=16, T=100)
        results = [rescore_trajectory(traj, cfg) for cfg in ALL_REWARD_CONFIGS]
        returns = [r["total_return"] for r in results]
        # At least some configs should differ
        assert len(set(returns)) > 1, f"All configs produced same return: {returns}"

    def test_config_b_higher_hp_reward_at_boss(self):
        """Config B (split milestones) should give higher reward for high HP at boss.

        Config B has death_penalty_scale=-3.0 (harsher death) and PBRS hp=0.80
        (heavier HP weighting), making HP preservation more rewarded.
        """
        traj = _make_trajectory(floor=16, T=100)
        result_a = rescore_trajectory(traj, REWARD_CONFIG_A)
        result_b = rescore_trajectory(traj, REWARD_CONFIG_B)
        # B has higher PBRS hp weight (0.80 vs 0.30), so HP-at-boss should be more rewarded
        assert result_b["hp_at_boss"] >= result_a["hp_at_boss"], (
            f"Config B hp_at_boss ({result_b['hp_at_boss']}) should be >= "
            f"Config A ({result_a['hp_at_boss']})"
        )

    def test_config_c_lower_reward_at_f16_with_zero_hp(self):
        """Config C with no floor milestones should give lower reward for reaching F16."""
        traj = _make_trajectory(floor=16, T=100)
        result_a = rescore_trajectory(traj, REWARD_CONFIG_A)
        result_c = rescore_trajectory(traj, REWARD_CONFIG_C)
        # Config C has no floor milestones, so floor_reward should be 0
        assert result_c["floor_reward"] == 0.0, (
            f"Config C should have 0 floor_reward, got {result_c['floor_reward']}"
        )
        assert result_a["floor_reward"] > 0, (
            f"Config A should have positive floor_reward, got {result_a['floor_reward']}"
        )

    def test_config_d_no_floor_milestones(self):
        """Config D should have zero floor milestone reward."""
        traj = _make_trajectory(floor=16, T=100)
        result_d = rescore_trajectory(traj, REWARD_CONFIG_D)
        assert result_d["floor_reward"] == 0.0

    def test_empty_trajectory(self):
        """Empty trajectories should not crash."""
        traj = _make_trajectory(floor=0, T=0)
        traj.obs = np.zeros((0, 480), dtype=np.float32)
        traj.rewards = np.zeros(0, dtype=np.float32)
        result = rescore_trajectory(traj, REWARD_CONFIG_A)
        assert result["total_return"] == 0.0

    def test_high_floor_trajectory(self):
        """Winning trajectory (F55) should have positive win_reward in all configs."""
        traj = _make_trajectory(floor=55, T=200)
        for cfg in ALL_REWARD_CONFIGS:
            result = rescore_trajectory(traj, cfg)
            assert result["total_return"] > 0, (
                f"Config {cfg['name']} should give positive return for win, "
                f"got {result['total_return']}"
            )

    def test_hp_gradient(self):
        """HP gradient (70 vs 10 HP) should be positive for configs with HP bonuses."""
        from packages.training.reward_sim import _hp_gradient
        for cfg in ALL_REWARD_CONFIGS:
            grad = _hp_gradient(cfg, 70, 10)
            # All configs should have non-negative gradient (more HP = more reward)
            assert grad >= 0, f"Config {cfg['name']} has negative HP gradient: {grad}"


class TestCompareConfigs:
    def test_compare_with_synthetic_data(self):
        """compare_configs should work with a temp directory of synthetic data."""
        with tempfile.TemporaryDirectory() as tmpdir:
            tmppath = Path(tmpdir)
            # Create a few fake .npz files
            for i, floor in enumerate([6, 10, 16, 16, 17]):
                T = 50
                np.savez(
                    tmppath / f"traj_F{floor:02d}_test_{i}.npz",
                    obs=np.random.randn(T, 480).astype(np.float32),
                    masks=np.ones((T, 512), dtype=np.bool_),
                    actions=np.zeros(T, dtype=np.int32),
                    rewards=np.random.randn(T).astype(np.float32) * 0.1,
                    dones=np.zeros(T, dtype=np.bool_),
                )

            results = compare_configs(
                configs=ALL_REWARD_CONFIGS,
                data_dirs=[tmppath],
            )

            assert results["num_trajectories"] == 5
            assert len(results["configs"]) == 4
            for name, stats in results["configs"].items():
                assert "mean_return" in stats
                assert "std_return" in stats
                assert "histogram" in stats

    def test_empty_directory(self):
        """compare_configs should handle empty directories gracefully."""
        with tempfile.TemporaryDirectory() as tmpdir:
            results = compare_configs(data_dirs=[Path(tmpdir)])
            assert "error" in results

    def test_floor_reward_curve(self):
        """Floor-reward curve should have entries for each floor in the data."""
        with tempfile.TemporaryDirectory() as tmpdir:
            tmppath = Path(tmpdir)
            for floor in [5, 10, 16]:
                T = 30
                np.savez(
                    tmppath / f"traj_F{floor:02d}_curve.npz",
                    obs=np.random.randn(T, 480).astype(np.float32),
                    masks=np.ones((T, 512), dtype=np.bool_),
                    actions=np.zeros(T, dtype=np.int32),
                    rewards=np.random.randn(T).astype(np.float32) * 0.1,
                    dones=np.zeros(T, dtype=np.bool_),
                )

            results = compare_configs(data_dirs=[tmppath])
            for name, stats in results["configs"].items():
                curve = stats["floor_reward_curve"]
                assert len(curve) > 0, f"Config {name} has empty floor_reward_curve"
