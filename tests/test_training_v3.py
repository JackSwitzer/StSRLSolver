"""Comprehensive tests for Training v3 changes.

Covers: training_config params, reward_config, seed_pool, sweep_config,
combat_net, offline_data, iql_trainer, grpo_trainer, turn_solver adapter,
and strategic_trainer updates.
"""
from __future__ import annotations

import numpy as np
import pytest
import torch
import torch.nn.functional as F


# ===================================================================
# Config Tests
# ===================================================================


class TestTrainingConfig:
    """Verify Training v3 config params exist and have correct values."""

    def test_config_lr_base(self):
        from packages.training.training_config import LR_BASE

        assert LR_BASE == 3e-5

    def test_config_collect_games(self):
        from packages.training.training_config import TRAIN_COLLECT_GAMES

        assert TRAIN_COLLECT_GAMES == 100

    def test_config_train_steps(self):
        from packages.training.training_config import TRAIN_STEPS_PER_PHASE

        assert TRAIN_STEPS_PER_PHASE == 30

    def test_config_solver_budgets_boss(self):
        from packages.training.training_config import SOLVER_BUDGETS

        base_ms, base_nodes, cap_ms = SOLVER_BUDGETS["boss"]
        assert base_ms == 120_000.0, f"Boss base_ms should be 120s, got {base_ms}"

    def test_config_solver_budgets_elite(self):
        from packages.training.training_config import SOLVER_BUDGETS

        base_ms, base_nodes, cap_ms = SOLVER_BUDGETS["elite"]
        assert base_ms == 2_000.0, f"Elite base_ms should be 2s, got {base_ms}"

    def test_config_boss_hp_progress_scale(self):
        from packages.training.training_config import BOSS_HP_PROGRESS_SCALE

        assert BOSS_HP_PROGRESS_SCALE == 3.0

    def test_config_multi_turn_depth(self):
        from packages.training.training_config import MULTI_TURN_DEPTH

        assert MULTI_TURN_DEPTH == 5

    def test_config_abort_criteria_exist(self):
        from packages.training import training_config as cfg

        assert hasattr(cfg, "ABORT_CLIP_FRACTION")
        assert hasattr(cfg, "ABORT_VALUE_LOSS")
        assert hasattr(cfg, "ABORT_ENTROPY_MIN")
        assert cfg.ABORT_CLIP_FRACTION == 0.30
        assert cfg.ABORT_VALUE_LOSS == 5.0
        assert cfg.ABORT_ENTROPY_MIN == 0.01

    def test_config_iql_params(self):
        from packages.training import training_config as cfg

        assert hasattr(cfg, "IQL_EXPECTILE")
        assert hasattr(cfg, "IQL_DISCOUNT")
        assert hasattr(cfg, "IQL_LR")
        assert hasattr(cfg, "IQL_TEMPERATURE")
        assert hasattr(cfg, "IQL_VALUE_HIDDEN")
        assert hasattr(cfg, "IQL_Q_HIDDEN")
        # Sanity: expectile in (0.5, 1.0) for upper-expectile regression
        assert 0.5 < cfg.IQL_EXPECTILE < 1.0
        # Discount close to 1
        assert 0.9 <= cfg.IQL_DISCOUNT <= 1.0
        # LR reasonable
        assert 1e-5 < cfg.IQL_LR < 1e-2

    def test_config_grpo_params(self):
        from packages.training import training_config as cfg

        assert hasattr(cfg, "GRPO_ROLLOUTS_CARD")
        assert hasattr(cfg, "GRPO_ROLLOUTS_OTHER")
        assert hasattr(cfg, "GRPO_CLIP")
        assert hasattr(cfg, "GRPO_LR")
        # Card picks get more rollouts than other decisions
        assert cfg.GRPO_ROLLOUTS_CARD >= cfg.GRPO_ROLLOUTS_OTHER
        assert cfg.GRPO_CLIP > 0


# ===================================================================
# Reward Tests
# ===================================================================


class TestBossHpProgress:
    """Verify compute_boss_hp_progress behavior."""

    def test_boss_hp_progress_full_kill(self):
        from packages.training.reward_config import compute_boss_hp_progress
        from packages.training.training_config import BOSS_HP_PROGRESS_SCALE

        result = compute_boss_hp_progress(300.0, 300.0)
        assert result == pytest.approx(BOSS_HP_PROGRESS_SCALE)

    def test_boss_hp_progress_half_kill(self):
        from packages.training.reward_config import compute_boss_hp_progress

        result = compute_boss_hp_progress(150.0, 300.0)
        assert result == pytest.approx(1.5)

    def test_boss_hp_progress_zero_damage(self):
        from packages.training.reward_config import compute_boss_hp_progress

        result = compute_boss_hp_progress(0.0, 300.0)
        assert result == pytest.approx(0.0)

    def test_boss_hp_progress_zero_max_hp(self):
        """0 max_hp must not divide by zero; returns 0.0."""
        from packages.training.reward_config import compute_boss_hp_progress

        result = compute_boss_hp_progress(100.0, 0.0)
        assert result == 0.0


# ===================================================================
# Seed Pool Tests
# ===================================================================


class TestSeedPool:
    """Verify expanded seed pool with 50 seeds (8 eval + 42 training)."""

    def test_seed_pool_50_seeds(self):
        from packages.training.seed_pool import SeedPool

        pool = SeedPool()
        assert len(pool.play_counts) == 50

    def test_seed_pool_eval_seeds_preserved(self):
        from packages.training.seed_pool import EVAL_SEEDS, SeedPool

        assert len(EVAL_SEEDS) == 8
        pool = SeedPool()
        for s in EVAL_SEEDS:
            assert s in pool.play_counts

    def test_seed_pool_training_seeds_42(self):
        from packages.training.seed_pool import TRAINING_SEEDS

        assert len(TRAINING_SEEDS) == 42

    def test_seed_pool_eval_priority(self):
        """get_seed() should return eval seeds first (they have priority)."""
        from packages.training.seed_pool import EVAL_SEEDS, SeedPool

        pool = SeedPool()
        first_seed = pool.get_seed()
        assert first_seed in EVAL_SEEDS, (
            f"First seed should be an eval seed, got {first_seed}"
        )

    def test_seed_pool_all_eval_returned_first(self):
        """First 8 unique seeds returned should all be eval seeds."""
        from packages.training.seed_pool import EVAL_SEEDS, SeedPool

        pool = SeedPool()
        first_8 = set()
        for _ in range(8):
            s = pool.get_seed()
            first_8.add(s)
        assert first_8 == set(EVAL_SEEDS)


# ===================================================================
# Sweep Config Tests
# ===================================================================


class TestV3AblationConfigs:
    """Verify V3 ablation sweep configuration."""

    def test_v3_ablation_4_configs(self):
        from packages.training.sweep_config import V3_ABLATION_CONFIGS

        assert len(V3_ABLATION_CONFIGS) == 4

    def test_v3_ablation_names(self):
        from packages.training.sweep_config import V3_ABLATION_CONFIGS

        names = [c["name"] for c in V3_ABLATION_CONFIGS]
        assert len(set(names)) == 4, f"Names must be unique, got {names}"

    def test_v3_ablation_max_hours(self):
        from packages.training.sweep_config import V3_ABLATION_CONFIGS

        for c in V3_ABLATION_CONFIGS:
            assert c["max_hours"] == 2.0, (
                f"Config {c['name']} max_hours should be 2.0, got {c.get('max_hours')}"
            )

    def test_v3_ablation_algorithms(self):
        from packages.training.sweep_config import V3_ABLATION_CONFIGS

        algorithms = {c["algorithm"] for c in V3_ABLATION_CONFIGS}
        assert "ppo" in algorithms
        assert "iql" in algorithms
        assert "grpo" in algorithms

    def test_v3_ablation_bc_warmup_present(self):
        """At least one config should have bc_warmup=True (the hybrid)."""
        from packages.training.sweep_config import V3_ABLATION_CONFIGS

        bc_configs = [c for c in V3_ABLATION_CONFIGS if c.get("bc_warmup")]
        assert len(bc_configs) >= 1


# ===================================================================
# Combat Net Tests
# ===================================================================


class TestCombatNet:
    """CombatNet forward/backward, save/load, and training."""

    @pytest.fixture
    def small_combat_net(self):
        torch.manual_seed(42)
        from packages.training.combat_net import CombatNet
        return CombatNet(input_dim=32, hidden_dim=32, num_layers=1)

    def test_combat_net_forward(self, small_combat_net):
        """Forward pass produces scalar in [0, 1]."""
        x = torch.randn(1, 32)
        out = small_combat_net(x)
        assert out.shape == (1,)
        assert 0.0 <= out.item() <= 1.0

    def test_combat_net_batch(self, small_combat_net):
        """Batch inference works and produces correct shape."""
        x = torch.randn(16, 32)
        out = small_combat_net(x)
        assert out.shape == (16,)
        assert (out >= 0).all() and (out <= 1).all()

    def test_combat_net_save_load(self, small_combat_net, tmp_path):
        """Round-trip save/load preserves weights."""
        path = tmp_path / "combat_net.pt"
        small_combat_net.save(path)

        from packages.training.combat_net import CombatNet
        loaded = CombatNet.load(path, device=torch.device("cpu"))

        x = torch.randn(4, 32)
        torch.manual_seed(42)
        orig_out = small_combat_net(x)
        loaded_out = loaded(x)
        assert torch.allclose(orig_out, loaded_out, atol=1e-6)

    def test_combat_net_train_smoke(self):
        """train_combat_net on fake data converges (loss decreases)."""
        torch.manual_seed(42)
        np.random.seed(42)
        from packages.training.combat_net import CombatNet, train_combat_net

        input_dim = 32
        # Create fake data: half wins, half losses with distinguishable features
        games_data = []
        for i in range(200):
            obs = np.random.randn(input_dim).astype(np.float32)
            won = i < 100
            # Make wins have positive features, losses negative
            if won:
                obs[:16] += 2.0
            else:
                obs[:16] -= 2.0
            games_data.append({"combat_obs": obs, "won": won})

        model = CombatNet(input_dim=input_dim, hidden_dim=32, num_layers=1)
        trained, metrics = train_combat_net(
            games_data, model=model, epochs=20, batch_size=32,
            lr=1e-3, device=torch.device("cpu"),
        )
        assert metrics["accuracy"] > 60.0, (
            f"Expected >60% accuracy on separable data, got {metrics['accuracy']:.1f}%"
        )
        assert metrics["samples"] == 200

    def test_combat_net_predict_single(self, small_combat_net):
        """predict() returns a float in [0, 1]."""
        obs = np.random.randn(32).astype(np.float32)
        p = small_combat_net.predict(obs)
        assert isinstance(p, float)
        assert 0.0 <= p <= 1.0

    def test_combat_net_predict_batch(self, small_combat_net):
        """predict_batch() returns correct shape numpy array."""
        obs = np.random.randn(8, 32).astype(np.float32)
        out = small_combat_net.predict_batch(obs)
        assert isinstance(out, np.ndarray)
        assert out.shape == (8,)


# ===================================================================
# Offline Data Tests
# ===================================================================


class TestOfflineDataset:
    """OfflineDataset construction, sampling, and next-state logic."""

    def _make_dataset(self, n=100, obs_dim=16, action_dim=8):
        np.random.seed(42)
        from packages.training.offline_data import OfflineDataset

        states = np.random.randn(n, obs_dim).astype(np.float32)
        actions = np.random.randint(0, action_dim, size=n).astype(np.int32)
        rewards = np.random.randn(n).astype(np.float32)
        # next_states: shift states by 1, zeros for last
        next_states = np.zeros_like(states)
        dones = np.zeros(n, dtype=np.float32)
        dones[-1] = 1.0  # last step is terminal
        for i in range(n - 1):
            if dones[i] == 0:
                next_states[i] = states[i + 1]
        action_masks = np.ones((n, action_dim), dtype=np.bool_)
        return OfflineDataset(
            states=states, actions=actions, rewards=rewards,
            next_states=next_states, dones=dones, action_masks=action_masks,
        )

    def test_offline_dataset_from_arrays(self):
        ds = self._make_dataset()
        assert len(ds) == 100

    def test_offline_dataset_sample_batch(self):
        ds = self._make_dataset(n=100, obs_dim=16, action_dim=8)
        batch = ds.sample_batch(32)
        assert batch.states.shape == (32, 16)
        assert batch.actions.shape == (32,)
        assert batch.rewards.shape == (32,)
        assert batch.next_states.shape == (32, 16)
        assert batch.dones.shape == (32,)
        assert batch.action_masks.shape == (32, 8)

    def test_offline_dataset_next_state(self):
        """s' = obs[i+1] for non-terminal, zeros for terminal."""
        from packages.training.offline_data import OfflineDataset

        states = np.array([[1, 0], [2, 0], [3, 0]], dtype=np.float32)
        dones = np.array([0, 1, 0], dtype=np.float32)
        next_states = np.zeros_like(states)
        # step 0: not done, s' = states[1]
        next_states[0] = states[1]
        # step 1: done, s' = zeros
        # step 2: not done but last step, s' = zeros

        ds = OfflineDataset(
            states=states,
            actions=np.array([0, 1, 0], dtype=np.int32),
            rewards=np.zeros(3, dtype=np.float32),
            next_states=next_states,
            dones=dones,
            action_masks=np.ones((3, 2), dtype=np.bool_),
        )
        item0 = ds[0]
        np.testing.assert_array_equal(item0["next_state"], [2.0, 0.0])
        item1 = ds[1]
        np.testing.assert_array_equal(item1["next_state"], [0.0, 0.0])

    def test_load_trajectories_from_npz(self, tmp_path):
        """Save fake .npz, load it back via load_trajectories."""
        from packages.training.offline_data import load_trajectories

        obs_dim = 16
        action_dim = 8
        T = 50
        np.random.seed(42)

        obs = np.random.randn(T, obs_dim).astype(np.float32)
        masks = np.ones((T, action_dim), dtype=np.bool_)
        actions = np.random.randint(0, action_dim, size=T).astype(np.int32)
        rewards = np.random.randn(T).astype(np.float32)
        dones = np.zeros(T, dtype=np.float32)
        dones[-1] = 1.0

        np.savez(
            tmp_path / "traj_F25_seed123.npz",
            obs=obs, masks=masks, actions=actions,
            rewards=rewards, dones=dones,
        )

        ds = load_trajectories([tmp_path], max_transitions=100)
        assert len(ds) == T
        np.testing.assert_array_equal(ds.states, obs)

    def test_load_trajectories_max_transitions(self, tmp_path):
        """max_transitions caps loaded data."""
        from packages.training.offline_data import load_trajectories

        obs = np.random.randn(200, 8).astype(np.float32)
        masks = np.ones((200, 4), dtype=np.bool_)
        actions = np.zeros(200, dtype=np.int32)
        rewards = np.zeros(200, dtype=np.float32)
        dones = np.zeros(200, dtype=np.float32)

        np.savez(
            tmp_path / "traj_F50_seed1.npz",
            obs=obs, masks=masks, actions=actions,
            rewards=rewards, dones=dones,
        )
        ds = load_trajectories([tmp_path], max_transitions=50)
        assert len(ds) == 50

    def test_load_trajectories_empty_dir(self, tmp_path):
        """Loading from empty directory returns empty dataset."""
        from packages.training.offline_data import load_trajectories

        ds = load_trajectories([tmp_path / "nonexistent"], max_transitions=100)
        assert len(ds) == 0


# ===================================================================
# IQL Trainer Tests
# ===================================================================


class TestIQLTrainer:
    """IQL trainer initialization and training steps."""

    @pytest.fixture
    def iql_setup(self):
        """Create small IQL trainer with fake data."""
        torch.manual_seed(42)
        np.random.seed(42)

        from packages.training.iql_trainer import IQLTrainer
        from packages.training.strategic_net import StrategicNet

        obs_dim = 32
        action_dim = 8

        model = StrategicNet(
            input_dim=obs_dim, hidden_dim=32,
            action_dim=action_dim, num_blocks=1,
        ).to(torch.device("cpu"))

        trainer = IQLTrainer(
            policy=model,
            input_dim=obs_dim,
            action_dim=action_dim,
            lr=1e-3,
            value_hidden=32,
            q_hidden=32,
        )
        return trainer, obs_dim, action_dim

    def _make_batch(self, obs_dim, action_dim, batch_size=32):
        from packages.training.offline_data import OfflineBatch

        return OfflineBatch(
            states=torch.randn(batch_size, obs_dim),
            actions=torch.randint(0, action_dim, (batch_size,)),
            rewards=torch.randn(batch_size),
            next_states=torch.randn(batch_size, obs_dim),
            dones=torch.zeros(batch_size),
            action_masks=torch.ones(batch_size, action_dim, dtype=torch.bool),
        )

    def test_iql_trainer_init(self, iql_setup):
        trainer, _, _ = iql_setup
        assert trainer.train_steps == 0
        assert trainer.v_net is not None
        assert trainer.q_net is not None
        assert trainer.q_target is not None

    def test_iql_train_step(self, iql_setup):
        trainer, obs_dim, action_dim = iql_setup
        batch = self._make_batch(obs_dim, action_dim)
        metrics = trainer.train_step(batch)
        assert isinstance(metrics, dict)
        assert trainer.train_steps == 1

    def test_iql_train_step_metrics(self, iql_setup):
        trainer, obs_dim, action_dim = iql_setup
        batch = self._make_batch(obs_dim, action_dim)
        metrics = trainer.train_step(batch)
        assert "q_loss" in metrics
        assert "v_loss" in metrics
        assert "policy_loss" in metrics
        assert "advantage_mean" in metrics
        assert "weight_mean" in metrics

    def test_iql_expectile_loss(self):
        """Expectile loss computation: higher weight on positive diffs."""
        from packages.training.iql_trainer import _expectile_loss

        diff = torch.tensor([1.0, -1.0, 0.5, -0.5])
        expectile = 0.7

        loss = _expectile_loss(diff, expectile)
        assert loss.item() > 0

        # Compare: positive diff weighted by 0.7, negative by 0.3
        weight = torch.where(diff > 0, 0.7, 0.3)
        expected = (weight * diff.pow(2)).mean()
        assert torch.allclose(loss, expected, atol=1e-6)

    def test_iql_train_offline_smoke(self, iql_setup):
        """train_offline on a small dataset runs without error."""
        trainer, obs_dim, action_dim = iql_setup

        from packages.training.offline_data import OfflineDataset
        n = 64
        ds = OfflineDataset(
            states=np.random.randn(n, obs_dim).astype(np.float32),
            actions=np.random.randint(0, action_dim, size=n).astype(np.int32),
            rewards=np.random.randn(n).astype(np.float32),
            next_states=np.random.randn(n, obs_dim).astype(np.float32),
            dones=np.zeros(n, dtype=np.float32),
            action_masks=np.ones((n, action_dim), dtype=np.bool_),
        )
        metrics = trainer.train_offline(ds, epochs=2, batch_size=16, steps_per_epoch=2)
        assert metrics["steps"] > 0
        assert "v_loss" in metrics


# ===================================================================
# GRPO Trainer Tests
# ===================================================================


class TestGRPOTrainer:
    """GRPO trainer initialization and batch training."""

    @pytest.fixture
    def grpo_setup(self):
        torch.manual_seed(42)
        np.random.seed(42)

        from packages.training.grpo_trainer import GRPOTrainer
        from packages.training.strategic_net import StrategicNet

        obs_dim = 32
        action_dim = 8

        model = StrategicNet(
            input_dim=obs_dim, hidden_dim=32,
            action_dim=action_dim, num_blocks=1,
        ).to(torch.device("cpu"))

        trainer = GRPOTrainer(
            model=model, lr=1e-3, clip=0.2,
            rollouts_card=3, rollouts_other=2,
        )
        return trainer, obs_dim, action_dim

    def test_grpo_trainer_init(self, grpo_setup):
        trainer, _, _ = grpo_setup
        assert trainer.train_steps == 0
        assert trainer.rollouts_card == 3
        assert trainer.rollouts_other == 2

    def test_grpo_train_batch_smoke(self, grpo_setup):
        """train_batch on fake groups returns metrics."""
        trainer, obs_dim, action_dim = grpo_setup
        from packages.training.grpo_trainer import GroupResult, GroupSample

        # Build 3 fake groups, each with 3 samples
        groups = []
        for g in range(3):
            samples = []
            for k in range(3):
                samples.append(GroupSample(
                    action_idx=k % action_dim,
                    obs=np.random.randn(obs_dim).astype(np.float32),
                    action_mask=np.ones(action_dim, dtype=np.bool_),
                    log_prob=float(np.log(1.0 / action_dim)),
                    total_return=float(np.random.randn()),
                ))
            group = GroupResult(samples=samples, phase_type="card_pick")
            group.compute_advantages()
            groups.append(group)

        metrics = trainer.train_batch(groups)
        assert "policy_loss" in metrics
        assert "entropy" in metrics
        assert metrics["groups"] == 3
        assert metrics["samples"] == 9
        assert trainer.train_steps == 1

    def test_grpo_train_batch_empty(self, grpo_setup):
        """Empty group list returns zero loss."""
        trainer, _, _ = grpo_setup
        metrics = trainer.train_batch([])
        assert metrics["policy_loss"] == 0.0
        assert metrics["groups"] == 0

    def test_grpo_group_advantages(self):
        """Group advantage computation normalizes returns."""
        from packages.training.grpo_trainer import GroupResult, GroupSample

        samples = [
            GroupSample(action_idx=0, obs=np.zeros(4, dtype=np.float32),
                        action_mask=np.ones(4, dtype=np.bool_),
                        log_prob=-1.0, total_return=10.0),
            GroupSample(action_idx=1, obs=np.zeros(4, dtype=np.float32),
                        action_mask=np.ones(4, dtype=np.bool_),
                        log_prob=-1.0, total_return=0.0),
        ]
        group = GroupResult(samples=samples, phase_type="card_pick")
        adv = group.compute_advantages()
        # Return 10 is above mean(5), so advantage[0] > 0
        assert adv[0] > 0
        assert adv[1] < 0
        # Normalized: mean ~0
        assert abs(adv.mean()) < 1e-5


# ===================================================================
# Turn Solver Adapter Tests
# ===================================================================


class TestTurnSolverAdapter:
    """TurnSolverAdapter constructor params for v3 wiring."""

    def test_adapter_solver_budgets_wired(self):
        from packages.training.turn_solver import TurnSolverAdapter

        budgets = {
            "monster": (50.0, 5_000, 300_000),
            "elite": (2_000.0, 50_000, 600_000),
            "boss": (30_000.0, 200_000, 600_000),
        }
        adapter = TurnSolverAdapter(solver_budgets=budgets)
        assert adapter._solver_budgets == budgets

    def test_adapter_multi_turn_depth_5(self):
        """Default multi_turn_depth should be 5 when passed explicitly."""
        from packages.training.turn_solver import TurnSolverAdapter

        adapter = TurnSolverAdapter(multi_turn_depth=5)
        # The MultiTurnSolver inside stores max_depth
        assert adapter._multi_turn.max_depth == 5

    def test_adapter_multi_turn_budget_30s(self):
        """Default multi_turn_budget_ms should be 30_000 when passed explicitly."""
        from packages.training.turn_solver import TurnSolverAdapter

        adapter = TurnSolverAdapter(multi_turn_budget_ms=30_000.0)
        assert adapter._multi_turn.time_budget_ms == 30_000.0

    def test_adapter_default_params(self):
        """Adapter with defaults constructs without error."""
        from packages.training.turn_solver import TurnSolverAdapter

        adapter = TurnSolverAdapter()
        assert adapter._solver is not None
        assert adapter._multi_turn is not None

    def test_adapter_reset(self):
        """reset() clears cached plan."""
        from packages.training.turn_solver import TurnSolverAdapter

        adapter = TurnSolverAdapter()
        adapter._cached_plan = ["fake"]
        adapter._cached_plan_index = 5
        adapter.reset()
        assert adapter._cached_plan is None
        assert adapter._cached_plan_index == 0


# ===================================================================
# Strategic Trainer Tests
# ===================================================================


class TestStrategicTrainer:
    """Strategic trainer: BC pretrain max_transitions, return normalization."""

    @pytest.fixture
    def trainer_setup(self):
        torch.manual_seed(42)
        from packages.training.strategic_net import StrategicNet
        from packages.training.strategic_trainer import StrategicTrainer

        obs_dim = 32
        action_dim = 8
        model = StrategicNet(
            input_dim=obs_dim, hidden_dim=32,
            action_dim=action_dim, num_blocks=1,
        ).to(torch.device("cpu"))
        trainer = StrategicTrainer(
            model=model, lr=1e-3, batch_size=16,
            ppo_epochs=2, warmup_steps=2,
        )
        return trainer, obs_dim, action_dim

    def test_bc_pretrain_max_48k(self):
        """Default max_transitions for bc_pretrain is 48000."""
        import inspect
        from packages.training.strategic_trainer import StrategicTrainer

        sig = inspect.signature(StrategicTrainer.bc_pretrain)
        default = sig.parameters["max_transitions"].default
        assert default == 48000

    def test_return_normalization(self, trainer_setup):
        """Returns are normalized in train_batch (training doesn't crash
        and produces reasonable metrics on data with large returns)."""
        trainer, obs_dim, action_dim = trainer_setup
        np.random.seed(42)

        # Add transitions with large, variable returns to stress normalization
        for i in range(64):
            obs = np.random.randn(obs_dim).astype(np.float32)
            mask = np.zeros(action_dim, dtype=np.bool_)
            mask[:3] = True
            trainer.add_transition(
                obs=obs,
                action_mask=mask,
                action=i % 3,
                reward=float(np.random.randn() * 100),  # Large rewards
                done=(i == 63),
                value=float(np.random.randn() * 50),
                log_prob=float(np.log(1.0 / 3)),
                episode_id=0,
            )

        metrics = trainer.train_batch()
        assert "value_loss" in metrics
        assert "policy_loss" in metrics
        # Key check: value_loss should be finite (normalization prevents NaN)
        assert np.isfinite(metrics["value_loss"])
        assert np.isfinite(metrics["policy_loss"])
        assert metrics["num_transitions"] == 64

    def test_train_batch_insufficient_data(self, trainer_setup):
        """train_batch with fewer transitions than batch_size returns early."""
        trainer, obs_dim, action_dim = trainer_setup
        # Add only 5 transitions (batch_size=16)
        for i in range(5):
            obs = np.random.randn(obs_dim).astype(np.float32)
            mask = np.ones(action_dim, dtype=np.bool_)
            trainer.add_transition(
                obs=obs, action_mask=mask, action=0,
                reward=1.0, done=False, value=0.5, log_prob=-1.0,
            )
        metrics = trainer.train_batch()
        assert metrics["num_transitions"] == 5
        assert metrics["policy_loss"] == 0  # No training happened

    def test_gae_computation(self, trainer_setup):
        """GAE produces non-zero advantages for non-trivial trajectories."""
        trainer, obs_dim, action_dim = trainer_setup
        np.random.seed(42)

        for i in range(20):
            obs = np.random.randn(obs_dim).astype(np.float32)
            mask = np.ones(action_dim, dtype=np.bool_)
            trainer.add_transition(
                obs=obs, action_mask=mask, action=i % action_dim,
                reward=float(i) * 0.1,  # Increasing rewards
                done=(i == 19),
                value=0.5,
                log_prob=-1.0,
                episode_id=0,
            )
        advantages, returns = trainer.compute_gae()
        assert len(advantages) == 20
        assert len(returns) == 20
        # Non-trivial: not all zeros
        assert np.any(advantages != 0)


# ===================================================================
# Worker Config Wiring Tests
# ===================================================================


class TestWorkerConfigWiring:
    """Verify worker.py imports the correct v3 config values."""

    def test_worker_imports_solver_budgets(self):
        from packages.training.worker import SOLVER_BUDGETS
        assert "boss" in SOLVER_BUDGETS
        assert "elite" in SOLVER_BUDGETS
        assert "monster" in SOLVER_BUDGETS

    def test_worker_imports_multi_turn_depth(self):
        from packages.training.worker import MULTI_TURN_DEPTH
        assert MULTI_TURN_DEPTH == 5

    def test_worker_imports_multi_turn_budget(self):
        from packages.training.worker import MULTI_TURN_BUDGET_MS
        assert MULTI_TURN_BUDGET_MS == 30_000.0

    def test_worker_imports_adaptive_budget_configs(self):
        from packages.training.worker import (
            MCTS_FLOOR_MULTIPLIERS, MCTS_PHASE_MULTIPLIERS, MCTS_ADAPTIVE_CAP,
        )
        assert MCTS_FLOOR_MULTIPLIERS[0] == 10.0  # Neow: 1 min deep planning
        assert MCTS_FLOOR_MULTIPLIERS[16] == 5.0   # Boss floor
        assert MCTS_PHASE_MULTIPLIERS["card_pick"] == 2.0
        assert MCTS_ADAPTIVE_CAP == 5000


class TestAdaptiveSearchBudget:
    """Verify adaptive search budget logic computes correct multipliers."""

    def test_neow_card_pick_gets_20x(self):
        """Floor 0 (10x) * card_pick (2x) = 20x budget."""
        from packages.training.training_config import (
            MCTS_FLOOR_MULTIPLIERS, MCTS_PHASE_MULTIPLIERS, MCTS_BUDGETS,
        )
        floor_mult = MCTS_FLOOR_MULTIPLIERS.get(0, 1.0)
        phase_mult = MCTS_PHASE_MULTIPLIERS.get("card_pick", 1.0)
        base = MCTS_BUDGETS["card_pick"]  # 200
        effective = int(base * floor_mult * phase_mult)
        assert effective == 4000  # 200 * 10 * 2

    def test_boss_floor_card_pick_gets_10x(self):
        """Floor 16 (5x) * card_pick (2x) = 10x budget."""
        from packages.training.training_config import (
            MCTS_FLOOR_MULTIPLIERS, MCTS_PHASE_MULTIPLIERS, MCTS_BUDGETS,
        )
        floor_mult = MCTS_FLOOR_MULTIPLIERS.get(16, 1.0)
        phase_mult = MCTS_PHASE_MULTIPLIERS.get("card_pick", 1.0)
        base = MCTS_BUDGETS["card_pick"]
        effective = int(base * floor_mult * phase_mult)
        assert effective == 2000  # 200 * 5 * 2

    def test_mid_run_path_is_normal(self):
        """Floor 7 (no multiplier) * path (1x) = 1x budget."""
        from packages.training.training_config import (
            MCTS_FLOOR_MULTIPLIERS, MCTS_PHASE_MULTIPLIERS,
        )
        floor_mult = MCTS_FLOOR_MULTIPLIERS.get(7, 1.0)
        phase_mult = MCTS_PHASE_MULTIPLIERS.get("path", 1.0)
        assert floor_mult * phase_mult == 1.0

    def test_forced_path_skipped(self):
        """If MCTS_SKIP_FORCED, n_actions=1 skips search entirely."""
        from packages.training.training_config import MCTS_SKIP_FORCED
        assert MCTS_SKIP_FORCED is True

    def test_pre_boss_rest_gets_boost(self):
        """Floor 15 (3x) * rest (1.5x) = 4.5x budget."""
        from packages.training.training_config import (
            MCTS_FLOOR_MULTIPLIERS, MCTS_PHASE_MULTIPLIERS,
        )
        floor_mult = MCTS_FLOOR_MULTIPLIERS.get(15, 1.0)
        phase_mult = MCTS_PHASE_MULTIPLIERS.get("rest", 1.0)
        assert floor_mult * phase_mult == pytest.approx(4.5)

    def test_low_impact_other_gets_half(self):
        """'other' phase type gets 0.5x multiplier."""
        from packages.training.training_config import MCTS_PHASE_MULTIPLIERS
        assert MCTS_PHASE_MULTIPLIERS["other"] == 0.5
