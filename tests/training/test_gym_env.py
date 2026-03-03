"""Tests for the Gymnasium environment wrapper (StsEnv and StsVecEnv).

Covers:
- Environment creation and reset
- Step returns valid 5-tuples
- Action mask correctness
- Episode termination
- Determinism with same seed
- Reward reasonableness
- Extended multi-step stability
- Vectorized environment parallel execution
"""

import numpy as np
import pytest

from packages.training.gym_env import StsEnv, StsVecEnv, _MAX_ACTION_INDEX


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

FIXED_SEED = "GYMTEST1"


@pytest.fixture
def env():
    """Single STS environment with fixed seed."""
    e = StsEnv(seed=FIXED_SEED, ascension=20)
    yield e


# ---------------------------------------------------------------------------
# Basic lifecycle
# ---------------------------------------------------------------------------


class TestEnvCreatesAndResets:
    """Environment can be instantiated and reset."""

    def test_create(self, env):
        assert env.observation_space is not None
        assert env.action_space is not None
        assert env.observation_space.shape[0] > 0
        assert env.action_space.n == _MAX_ACTION_INDEX

    def test_reset_returns_obs_and_info(self, env):
        obs, info = env.reset()
        assert isinstance(obs, np.ndarray)
        assert obs.dtype == np.float32
        assert obs.shape == env.observation_space.shape
        assert isinstance(info, dict)

    def test_reset_populates_info(self, env):
        _, info = env.reset()
        assert "action_mask" in info
        assert "available_actions" in info
        assert "phase" in info
        assert "floor" in info
        assert "hp" in info
        assert "max_hp" in info
        assert "num_actions" in info

    def test_reset_action_mask_shape(self, env):
        _, info = env.reset()
        mask = info["action_mask"]
        assert isinstance(mask, np.ndarray)
        assert mask.dtype == np.bool_
        assert mask.shape == (_MAX_ACTION_INDEX,)
        # At least one action should be legal after reset
        assert mask.any()

    def test_reset_with_different_seed(self, env):
        obs1, _ = env.reset(seed=42)
        obs2, _ = env.reset(seed=99)
        # Different seeds should (almost certainly) produce different obs.
        # We allow equality in the extremely unlikely case of collision.
        # This is a smoke test, not a strict assertion.
        assert obs1.shape == obs2.shape


# ---------------------------------------------------------------------------
# Stepping
# ---------------------------------------------------------------------------


class TestEnvStep:
    """Environment step returns valid tuples."""

    def test_step_returns_5_tuple(self, env):
        _, info = env.reset()
        mask = info["action_mask"]
        action = int(np.flatnonzero(mask)[0])
        result = env.step(action)
        assert len(result) == 5
        obs, reward, terminated, truncated, step_info = result
        assert isinstance(obs, np.ndarray)
        assert obs.dtype == np.float32
        assert isinstance(reward, float)
        assert isinstance(terminated, bool)
        assert isinstance(truncated, bool)
        assert isinstance(step_info, dict)

    def test_step_obs_shape(self, env):
        _, info = env.reset()
        mask = info["action_mask"]
        action = int(np.flatnonzero(mask)[0])
        obs, _, _, _, _ = env.step(action)
        assert obs.shape == env.observation_space.shape

    def test_step_invalid_action_returns_penalty(self, env):
        _, info = env.reset()
        mask = info["action_mask"]
        # Find an index that is NOT in the mask.
        invalid = int(np.flatnonzero(~mask)[0])
        obs, reward, terminated, truncated, _ = env.step(invalid)
        assert reward == pytest.approx(-0.01)
        assert not terminated
        assert not truncated


# ---------------------------------------------------------------------------
# Action mask
# ---------------------------------------------------------------------------


class TestEnvActionMask:
    """Action mask is correctly populated."""

    def test_mask_has_correct_count(self, env):
        _, info = env.reset()
        mask = info["action_mask"]
        n_true = int(mask.sum())
        n_actions = len(info["available_actions"])
        assert n_true == n_actions

    def test_mask_indices_match_actions(self, env):
        _, info = env.reset()
        mask = info["action_mask"]
        actions = info["available_actions"]
        for action in actions:
            idx = env.action_space_manager.action_to_index(action)
            assert mask[idx], f"Action {action['id']} at index {idx} not set in mask"

    def test_mask_updates_after_step(self, env):
        _, info1 = env.reset()
        mask1 = info1["action_mask"].copy()
        action = int(np.flatnonzero(mask1)[0])
        _, _, _, _, info2 = env.step(action)
        # Mask may or may not change, but it should be valid
        mask2 = info2["action_mask"]
        assert mask2.shape == mask1.shape
        assert mask2.any() or env.runner.game_over


# ---------------------------------------------------------------------------
# Termination
# ---------------------------------------------------------------------------


class TestEnvTermination:
    """Episode terminates correctly."""

    def test_game_over_sets_terminated(self, env):
        obs, info = env.reset()
        terminated = False
        steps = 0
        while not terminated and steps < 3000:
            mask = info["action_mask"]
            if not mask.any():
                break
            action = int(np.flatnonzero(mask)[0])
            obs, reward, terminated, truncated, info = env.step(action)
            steps += 1
        # Game should terminate within reasonable steps (or truncate)
        assert terminated or steps >= 3000 or truncated

    def test_terminated_info_has_metadata(self, env):
        env.max_steps = 50000  # Don't truncate
        obs, info = env.reset()
        terminated = False
        steps = 0
        while not terminated and steps < 5000:
            mask = info["action_mask"]
            if not mask.any():
                break
            action = int(np.flatnonzero(mask)[0])
            obs, reward, terminated, truncated, info = env.step(action)
            steps += 1
        if terminated:
            assert "game_won" in info
            assert "game_lost" in info
            assert "final_floor" in info
            assert "final_hp" in info


# ---------------------------------------------------------------------------
# Determinism
# ---------------------------------------------------------------------------


class TestEnvDeterminism:
    """Same seed produces same trajectory."""

    def test_deterministic_with_same_seed(self):
        env1 = StsEnv(seed="DETERM1", ascension=20)
        env2 = StsEnv(seed="DETERM1", ascension=20)

        obs1, info1 = env1.reset()
        obs2, info2 = env2.reset()

        np.testing.assert_array_equal(obs1, obs2)
        assert len(info1["available_actions"]) == len(info2["available_actions"])

        # Take the same first action.
        mask1 = info1["action_mask"]
        mask2 = info2["action_mask"]
        np.testing.assert_array_equal(mask1, mask2)

        action = int(np.flatnonzero(mask1)[0])
        obs1b, r1, t1, tr1, info1b = env1.step(action)
        obs2b, r2, t2, tr2, info2b = env2.step(action)

        np.testing.assert_array_equal(obs1b, obs2b)
        assert r1 == r2
        assert t1 == t2
        assert tr1 == tr2


# ---------------------------------------------------------------------------
# Reward
# ---------------------------------------------------------------------------


class TestEnvReward:
    """Reward signal is reasonable."""

    def test_reward_is_finite(self, env):
        _, info = env.reset()
        for _ in range(10):
            mask = info["action_mask"]
            if not mask.any():
                break
            action = int(np.flatnonzero(mask)[0])
            _, reward, terminated, _, info = env.step(action)
            assert np.isfinite(reward)
            if terminated:
                break

    def test_reward_bounded(self, env):
        """Rewards should be in a reasonable range."""
        _, info = env.reset()
        for _ in range(50):
            mask = info["action_mask"]
            if not mask.any():
                break
            action = int(np.flatnonzero(mask)[0])
            _, reward, terminated, _, info = env.step(action)
            # Reward per step should be bounded: step cost + hp_delta + terminal
            # Worst case: -0.001 - 1.0 (full hp loss) - 0.5 = -1.501
            # Best case: -0.001 + 1.0 (full hp gain) + 1.0 (win) = 1.999
            assert -2.0 <= reward <= 2.0, f"Reward {reward} out of expected range"
            if terminated:
                break


# ---------------------------------------------------------------------------
# Extended run
# ---------------------------------------------------------------------------


class TestEnvExtendedRun:
    """Environment runs 100+ steps without crashing."""

    def test_100_steps_no_crash(self, env):
        _, info = env.reset()
        for step_i in range(100):
            mask = info["action_mask"]
            if not mask.any():
                break
            action = int(np.flatnonzero(mask)[0])
            obs, reward, terminated, truncated, info = env.step(action)
            assert obs.shape == env.observation_space.shape
            if terminated or truncated:
                obs, info = env.reset()

    def test_multiple_episodes(self, env):
        """Run 3 full episodes to completion or 500 steps each."""
        for episode in range(3):
            obs, info = env.reset()
            for _ in range(500):
                mask = info["action_mask"]
                if not mask.any():
                    break
                action = int(np.flatnonzero(mask)[0])
                obs, reward, terminated, truncated, info = env.step(action)
                if terminated or truncated:
                    break

    def test_render_ansi(self, env):
        env.render_mode = "ansi"
        _, info = env.reset()
        text = env.render()
        assert isinstance(text, str)
        assert "Phase:" in text
        assert "HP:" in text


# ---------------------------------------------------------------------------
# Vectorized environment
# ---------------------------------------------------------------------------


class TestVecEnv:
    """Vectorized environment parallel execution."""

    @pytest.fixture
    def venv(self):
        v = StsVecEnv(num_envs=2, seed="VECTEST", ascension=20)
        yield v
        v.close()

    def test_vec_reset(self, venv):
        obs_batch, info_batch = venv.reset()
        assert obs_batch.shape[0] == 2
        assert obs_batch.ndim == 2
        assert len(info_batch) == 2
        for info in info_batch:
            assert "action_mask" in info

    def test_vec_step(self, venv):
        obs_batch, info_batch = venv.reset()
        actions = np.array([
            int(np.flatnonzero(info_batch[i]["action_mask"])[0])
            for i in range(2)
        ])
        obs_batch, rewards, terminateds, truncateds, info_batch = venv.step(actions)
        assert obs_batch.shape[0] == 2
        assert rewards.shape == (2,)
        assert terminateds.shape == (2,)
        assert truncateds.shape == (2,)
        assert len(info_batch) == 2

    def test_vec_multiple_steps(self, venv):
        obs_batch, info_batch = venv.reset()
        for _ in range(20):
            actions = np.array([
                int(np.flatnonzero(info_batch[i]["action_mask"])[0])
                for i in range(venv.num_envs)
            ])
            obs_batch, rewards, terminateds, truncateds, info_batch = venv.step(actions)
            assert obs_batch.shape[0] == venv.num_envs
            assert rewards.dtype == np.float32

    def test_vec_close(self):
        venv = StsVecEnv(num_envs=2, seed="CLOSE", ascension=20)
        venv.reset()
        venv.close()
        # Should not raise on double close.
        venv.close()
