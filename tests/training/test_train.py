"""Tests for the training pipeline: PolicyValueNet, EpisodeLogger, and Trainer.

Covers:
- PolicyValueNet forward pass shapes
- Action masking (invalid actions get -inf)
- Save/load roundtrip
- EpisodeLogger collection and serialization
- Trainer episode collection and single training step
"""

import json
import tempfile
from pathlib import Path

import numpy as np
import pytest

from packages.training.policy_net import PolicyValueNet, _softmax
from packages.training.episode_logger import EpisodeLog, EpisodeStep
from packages.training.train import Trainer, RolloutBuffer
from packages.training.gym_env import StsEnv, _MAX_ACTION_INDEX


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

OBS_DIM = 1186
ACTION_DIM = 2048
FIXED_SEED = "TRAINTEST1"


@pytest.fixture
def policy():
    """Small policy network for testing."""
    return PolicyValueNet(obs_dim=OBS_DIM, action_dim=ACTION_DIM, hidden_dim=64, num_layers=2)


@pytest.fixture
def obs():
    """Random observation vector."""
    return np.random.randn(OBS_DIM).astype(np.float32)


@pytest.fixture
def action_mask():
    """Random action mask with some valid actions."""
    mask = np.zeros(ACTION_DIM, dtype=np.bool_)
    # Enable 10 random actions
    indices = np.random.choice(ACTION_DIM, size=10, replace=False)
    mask[indices] = True
    return mask


@pytest.fixture
def env():
    """StsEnv for integration tests."""
    return StsEnv(seed=FIXED_SEED, ascension=20)


# ---------------------------------------------------------------------------
# PolicyValueNet tests
# ---------------------------------------------------------------------------


class TestPolicyNetForwardShapes:
    """Network forward pass returns correctly shaped outputs."""

    def test_forward_returns_logits_and_value(self, policy, obs):
        logits, value = policy.forward(obs)
        assert isinstance(logits, np.ndarray)
        assert isinstance(value, float)

    def test_logits_shape(self, policy, obs):
        logits, _ = policy.forward(obs)
        assert logits.shape == (ACTION_DIM,)

    def test_logits_dtype(self, policy, obs):
        logits, _ = policy.forward(obs)
        assert logits.dtype == np.float32

    def test_value_bounded(self, policy, obs):
        _, value = policy.forward(obs)
        assert -1.0 <= value <= 1.0

    def test_forward_batch(self, policy):
        batch = np.random.randn(4, OBS_DIM).astype(np.float32)
        logits, values = policy.forward_batch(batch)
        assert logits.shape == (4, ACTION_DIM)
        assert values.shape == (4,)

    def test_param_count_positive(self, policy):
        assert policy.param_count > 0

    def test_different_obs_different_output(self, policy):
        obs1 = np.random.randn(OBS_DIM).astype(np.float32)
        obs2 = np.random.randn(OBS_DIM).astype(np.float32)
        logits1, v1 = policy.forward(obs1)
        logits2, v2 = policy.forward(obs2)
        # Very unlikely to be exactly equal
        assert not np.allclose(logits1, logits2)


class TestPolicyNetActionMasking:
    """Action masking correctly prevents invalid actions."""

    def test_predict_action_returns_valid_index(self, policy, obs, action_mask):
        action = policy.predict_action(obs, action_mask)
        assert isinstance(action, int)
        assert 0 <= action < ACTION_DIM
        assert action_mask[action], "Predicted action should be in the mask"

    def test_predict_action_respects_mask(self, policy, obs):
        # Only action 5 is valid
        mask = np.zeros(ACTION_DIM, dtype=np.bool_)
        mask[5] = True
        action = policy.predict_action(obs, mask, temperature=0.0)
        assert action == 5

    def test_predict_action_greedy(self, policy, obs, action_mask):
        # Greedy should be deterministic
        a1 = policy.predict_action(obs, action_mask, temperature=0.0)
        a2 = policy.predict_action(obs, action_mask, temperature=0.0)
        assert a1 == a2

    def test_predict_action_stochastic(self, policy, obs, action_mask):
        # With temperature=1, should sample. Run many times and check diversity.
        actions = set()
        for _ in range(100):
            a = policy.predict_action(obs, action_mask, temperature=1.0)
            assert action_mask[a]
            actions.add(a)
        # With 10 valid actions and 100 samples, should see more than 1
        assert len(actions) > 1

    def test_all_masked_returns_valid(self, policy, obs):
        # Edge case: no valid actions (should not crash)
        mask = np.zeros(ACTION_DIM, dtype=np.bool_)
        action = policy.predict_action(obs, mask)
        assert isinstance(action, int)


class TestPolicyNetSaveLoad:
    """Save and load roundtrip preserves weights."""

    def test_save_load_roundtrip(self, policy, obs):
        with tempfile.TemporaryDirectory() as tmpdir:
            path = str(Path(tmpdir) / "test_policy.npz")
            policy.save(path)

            # Get output before
            logits_before, value_before = policy.forward(obs)

            # Load into new network
            loaded = PolicyValueNet(
                obs_dim=OBS_DIM, action_dim=ACTION_DIM,
                hidden_dim=64, num_layers=2,
            )
            loaded.load(path)

            logits_after, value_after = loaded.forward(obs)

            np.testing.assert_array_almost_equal(logits_before, logits_after)
            assert abs(value_before - value_after) < 1e-6

    def test_save_creates_directories(self, policy):
        with tempfile.TemporaryDirectory() as tmpdir:
            path = str(Path(tmpdir) / "deep" / "nested" / "policy.npz")
            policy.save(path)
            assert Path(path).exists()

    def test_load_with_npz_extension(self, policy, obs):
        with tempfile.TemporaryDirectory() as tmpdir:
            path = str(Path(tmpdir) / "policy.npz")
            policy.save(path)

            loaded = PolicyValueNet(
                obs_dim=OBS_DIM, action_dim=ACTION_DIM,
                hidden_dim=64, num_layers=2,
            )
            # Load without extension should still work
            loaded.load(path.replace(".npz", ""))

            logits_orig, _ = policy.forward(obs)
            logits_loaded, _ = loaded.forward(obs)
            np.testing.assert_array_almost_equal(logits_orig, logits_loaded)

    def test_loaded_params_match(self, policy):
        with tempfile.TemporaryDirectory() as tmpdir:
            path = str(Path(tmpdir) / "policy.npz")
            policy.save(path)

            loaded = PolicyValueNet(
                obs_dim=OBS_DIM, action_dim=ACTION_DIM,
                hidden_dim=64, num_layers=2,
            )
            loaded.load(path)

            assert loaded.obs_dim == policy.obs_dim
            assert loaded.action_dim == policy.action_dim
            assert loaded.hidden_dim == policy.hidden_dim
            assert loaded.num_layers == policy.num_layers
            assert loaded.param_count == policy.param_count


# ---------------------------------------------------------------------------
# EpisodeLogger tests
# ---------------------------------------------------------------------------


class TestEpisodeLoggerCollectsEpisode:
    """EpisodeLog correctly collects episodes from the environment."""

    def test_from_env_rollout_returns_log(self, env, policy):
        log = EpisodeLog.from_env_rollout(env, policy, max_steps=50)
        assert isinstance(log, EpisodeLog)
        assert log.seed is not None
        assert log.ascension == 20
        assert log.character == "Watcher"
        assert isinstance(log.total_reward, float)
        assert len(log.steps) > 0
        assert len(log.hp_history) > 0

    def test_from_env_rollout_steps_have_data(self, env, policy):
        log = EpisodeLog.from_env_rollout(env, policy, max_steps=20)
        for step in log.steps:
            assert isinstance(step.action_index, int)
            assert isinstance(step.reward, float)
            assert isinstance(step.phase, str)
            assert isinstance(step.floor, int)

    def test_from_env_rollout_no_obs_when_disabled(self, env, policy):
        log = EpisodeLog.from_env_rollout(env, policy, max_steps=10, record_obs=False)
        for step in log.steps:
            assert step.observation == []

    def test_from_env_rollout_total_reward_sums(self, env, policy):
        log = EpisodeLog.from_env_rollout(env, policy, max_steps=20)
        computed_total = sum(s.reward for s in log.steps)
        assert abs(log.total_reward - computed_total) < 1e-6

    def test_duration_positive(self, env, policy):
        log = EpisodeLog.from_env_rollout(env, policy, max_steps=10)
        assert log.duration_seconds >= 0.0

    def test_timestamp_populated(self, env, policy):
        log = EpisodeLog.from_env_rollout(env, policy, max_steps=10)
        assert log.timestamp != ""


class TestEpisodeLoggerSaveLoad:
    """Episode save/load roundtrip preserves data."""

    def test_save_load_roundtrip(self, env, policy):
        with tempfile.TemporaryDirectory() as tmpdir:
            path = str(Path(tmpdir) / "episodes.jsonl")

            log = EpisodeLog.from_env_rollout(env, policy, max_steps=10)
            log.save(path)

            loaded = EpisodeLog.load(path)
            assert len(loaded) == 1
            loaded_log = loaded[0]

            assert loaded_log.seed == log.seed
            assert loaded_log.ascension == log.ascension
            assert loaded_log.character == log.character
            assert loaded_log.won == log.won
            assert abs(loaded_log.total_reward - log.total_reward) < 1e-6
            assert loaded_log.floors_reached == log.floors_reached
            assert len(loaded_log.steps) == len(log.steps)

    def test_save_multiple_episodes(self, env, policy):
        with tempfile.TemporaryDirectory() as tmpdir:
            path = str(Path(tmpdir) / "multi.jsonl")

            for _ in range(3):
                log = EpisodeLog.from_env_rollout(env, policy, max_steps=10)
                log.save(path)

            loaded = EpisodeLog.load(path)
            assert len(loaded) == 3

    def test_load_nonexistent_returns_empty(self):
        loaded = EpisodeLog.load("/nonexistent/path/file.jsonl")
        assert loaded == []

    def test_save_compact(self, env, policy):
        with tempfile.TemporaryDirectory() as tmpdir:
            path = str(Path(tmpdir) / "compact.jsonl")

            log = EpisodeLog.from_env_rollout(env, policy, max_steps=10)
            log.save_compact(path)

            # Compact saves only metadata (1 line)
            with open(path) as f:
                lines = f.readlines()
            assert len(lines) == 1

            d = json.loads(lines[0])
            assert d["seed"] == log.seed
            assert d["num_steps"] == len(log.steps)

    def test_step_serialization(self):
        step = EpisodeStep(
            observation=[1.0, 2.0, 3.0],
            action_index=42,
            action_id="play_card|Strike",
            reward=-0.5,
            phase="COMBAT",
            floor=3,
        )
        d = step.to_dict()
        restored = EpisodeStep.from_dict(d)
        assert restored.action_index == 42
        assert restored.action_id == "play_card|Strike"
        assert restored.reward == -0.5


# ---------------------------------------------------------------------------
# Trainer tests
# ---------------------------------------------------------------------------


class TestTrainerCollectEpisodes:
    """Trainer can collect episodes without crashing."""

    def test_collect_single_episode(self):
        trainer = Trainer(
            obs_dim=OBS_DIM,
            hidden_dim=64,
            num_layers=2,
            max_steps_per_episode=50,
            seed=FIXED_SEED,
        )
        buffers, logs = trainer.collect_episodes(n_episodes=1)
        assert len(buffers) == 1
        assert len(logs) == 1
        assert len(buffers[0]) > 0
        assert len(logs[0].steps) > 0

    def test_rollout_buffer_contents(self):
        trainer = Trainer(
            obs_dim=OBS_DIM,
            hidden_dim=64,
            num_layers=2,
            max_steps_per_episode=20,
            seed=FIXED_SEED,
        )
        buffers, _ = trainer.collect_episodes(n_episodes=1)
        buf = buffers[0]

        arrays = buf.to_arrays()
        assert arrays["observations"].shape[0] == len(buf)
        assert arrays["observations"].shape[1] == OBS_DIM
        assert arrays["actions"].shape == (len(buf),)
        assert arrays["rewards"].shape == (len(buf),)
        assert arrays["action_masks"].shape[0] == len(buf)


class TestTrainerOneStep:
    """Trainer can perform one training step without crashing."""

    def test_one_training_step(self):
        trainer = Trainer(
            obs_dim=OBS_DIM,
            hidden_dim=64,
            num_layers=2,
            max_steps_per_episode=30,
            seed=FIXED_SEED,
            learning_rate=1e-3,
        )
        buffers, logs = trainer.collect_episodes(n_episodes=1)
        metrics = trainer.train_step(buffers)

        assert "policy_loss" in metrics
        assert "value_loss" in metrics
        assert "entropy" in metrics
        assert np.isfinite(metrics["policy_loss"])
        assert np.isfinite(metrics["value_loss"])
        assert np.isfinite(metrics["entropy"])

    def test_empty_buffer_no_crash(self):
        trainer = Trainer(
            obs_dim=OBS_DIM,
            hidden_dim=64,
            num_layers=2,
            max_steps_per_episode=30,
            seed=FIXED_SEED,
        )
        empty_buf = RolloutBuffer()
        metrics = trainer.train_step([empty_buf])
        assert metrics["policy_loss"] == 0.0


# ---------------------------------------------------------------------------
# RolloutBuffer tests
# ---------------------------------------------------------------------------


class TestRolloutBuffer:
    """RolloutBuffer computes returns and advantages correctly."""

    def test_returns_and_advantages_shape(self):
        buf = RolloutBuffer()
        for i in range(10):
            buf.add(
                obs=np.zeros(OBS_DIM, dtype=np.float32),
                action=0,
                reward=1.0,
                value=0.5,
                action_mask=np.ones(ACTION_DIM, dtype=np.bool_),
                done=(i == 9),
            )
        returns, advantages = buf.compute_returns_and_advantages()
        assert returns.shape == (10,)
        assert advantages.shape == (10,)

    def test_returns_positive_for_positive_rewards(self):
        buf = RolloutBuffer()
        for i in range(5):
            buf.add(
                obs=np.zeros(OBS_DIM, dtype=np.float32),
                action=0,
                reward=1.0,
                value=0.0,
                action_mask=np.ones(ACTION_DIM, dtype=np.bool_),
                done=(i == 4),
            )
        returns, _ = buf.compute_returns_and_advantages()
        assert all(r > 0 for r in returns)

    def test_terminal_resets_gae(self):
        buf = RolloutBuffer()
        # Episode 1: reward = 1
        buf.add(np.zeros(OBS_DIM, dtype=np.float32), 0, 1.0, 0.0,
                np.ones(ACTION_DIM, dtype=np.bool_), True)
        # Episode 2: reward = -1
        buf.add(np.zeros(OBS_DIM, dtype=np.float32), 0, -1.0, 0.0,
                np.ones(ACTION_DIM, dtype=np.bool_), True)

        returns, advantages = buf.compute_returns_and_advantages()
        # First episode positive, second negative
        assert returns[0] > 0
        assert returns[1] < 0


# ---------------------------------------------------------------------------
# Softmax helper test
# ---------------------------------------------------------------------------


class TestSoftmax:
    """Softmax utility function."""

    def test_sums_to_one(self):
        x = np.array([1.0, 2.0, 3.0])
        s = _softmax(x)
        assert abs(s.sum() - 1.0) < 1e-6

    def test_numerically_stable(self):
        x = np.array([1000.0, 1001.0, 1002.0])
        s = _softmax(x)
        assert np.all(np.isfinite(s))
        assert abs(s.sum() - 1.0) < 1e-6

    def test_negative_inputs(self):
        x = np.array([-100.0, -200.0, -50.0])
        s = _softmax(x)
        assert np.all(s >= 0)
        assert abs(s.sum() - 1.0) < 1e-6
