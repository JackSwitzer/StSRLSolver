"""Tests for PopArt value normalization and value head pretraining."""

from __future__ import annotations

from pathlib import Path

import numpy as np
import pytest
import torch

from packages.training.strategic_net import PopArtLayer, StrategicNet
from packages.training.strategic_trainer import StrategicTrainer, StrategicTransition


class TestPopArtLayer:
    """PopArt normalization layer tests."""

    def test_initial_state(self):
        layer = PopArtLayer(beta=0.01)
        assert layer.mu.item() == 0.0
        assert layer.sigma.item() == 1.0
        assert layer.count.item() == 0.0

    def test_normalize_identity_at_init(self):
        layer = PopArtLayer()
        x = torch.tensor([5.0, 10.0, 15.0])
        normed = layer.normalize(x)
        # mu=0, sigma=1 at init, so normalize is identity
        torch.testing.assert_close(normed, x, atol=1e-6, rtol=1e-6)

    def test_denormalize_identity_at_init(self):
        layer = PopArtLayer()
        x = torch.tensor([1.0, 2.0, 3.0])
        denormed = layer.denormalize(x)
        torch.testing.assert_close(denormed, x, atol=1e-6, rtol=1e-6)

    def test_normalize_denormalize_roundtrip(self):
        layer = PopArtLayer(beta=0.5)
        targets = torch.tensor([10.0, 20.0, 30.0, 40.0, 50.0])
        # Warm up stats
        for _ in range(20):
            layer.update(targets)
        normed = layer.normalize(targets)
        recovered = layer.denormalize(normed)
        torch.testing.assert_close(recovered, targets, atol=1e-4, rtol=1e-4)

    def test_variance_near_one_after_warmup(self):
        """After sufficient updates, normalized targets should have variance ~1."""
        layer = PopArtLayer(beta=0.01)
        torch.manual_seed(42)
        # Feed batches with mean=20, std=10
        for _ in range(500):
            batch = torch.randn(64) * 10 + 20
            layer.update(batch)

        # Now normalize a test batch
        test = torch.randn(1000) * 10 + 20
        normed = layer.normalize(test)
        var = normed.var().item()
        # Variance should be close to 1 (within tolerance)
        assert 0.5 < var < 2.0, f"Normalized variance {var} not near 1.0"

    def test_update_shifts_statistics(self):
        layer = PopArtLayer(beta=1.0)  # Instant update for testing
        batch = torch.tensor([100.0, 100.0, 100.0])
        layer.update(batch)
        assert layer.mu.item() == pytest.approx(100.0, abs=1.0)


class TestValueNormMethods:
    """Test all 3 normalization methods work in the trainer."""

    @pytest.fixture
    def model(self):
        return StrategicNet(input_dim=32, hidden_dim=32, action_dim=8, num_blocks=1)

    def _make_buffer(self, trainer: StrategicTrainer, n: int = 300):
        """Fill trainer buffer with synthetic transitions."""
        for i in range(n):
            obs = np.random.randn(32).astype(np.float32)
            mask = np.ones(8, dtype=bool)
            trainer.buffer.append(StrategicTransition(
                obs=obs,
                action_mask=mask,
                action=0,
                reward=float(np.random.randn()),
                done=(i % 20 == 19),
                value=float(np.random.randn()),
                log_prob=-1.0,
                episode_id=i // 20,
                final_floor=float(np.random.rand()),
                cleared_act1=1.0 if i > 10 else 0.0,
                cleared_act2=0.0,
                cleared_act3=0.0,
            ))

    def test_popart_method(self, model, monkeypatch):
        monkeypatch.setattr("packages.training.training_config.VALUE_NORM_METHOD", "popart")
        monkeypatch.setattr("packages.training.training_config.POPART_BETA", 0.01)
        trainer = StrategicTrainer(model, batch_size=64, ppo_epochs=1)
        trainer._value_norm_method = "popart"
        self._make_buffer(trainer)
        metrics = trainer.train_batch()
        assert metrics["value_loss"] > 0
        assert metrics["total_loss"] > 0

    def test_clip_method(self, model, monkeypatch):
        monkeypatch.setattr("packages.training.training_config.VALUE_NORM_METHOD", "clip")
        trainer = StrategicTrainer(model, batch_size=64, ppo_epochs=1)
        trainer._value_norm_method = "clip"
        self._make_buffer(trainer)
        metrics = trainer.train_batch()
        assert metrics["value_loss"] > 0

    def test_none_method(self, model, monkeypatch):
        monkeypatch.setattr("packages.training.training_config.VALUE_NORM_METHOD", "none")
        trainer = StrategicTrainer(model, batch_size=64, ppo_epochs=1)
        trainer._value_norm_method = "none"
        self._make_buffer(trainer)
        metrics = trainer.train_batch()
        assert metrics["value_loss"] > 0


class TestValueLossWithNormalization:
    """Value loss should be lower with normalization on high-variance returns."""

    def test_popart_reduces_value_loss(self):
        """With PopArt on high-variance data, loss should be < 1.0 after training."""
        torch.manual_seed(42)
        model = StrategicNet(input_dim=32, hidden_dim=64, action_dim=8, num_blocks=1)
        popart = PopArtLayer(beta=0.01)

        # Synthetic high-variance targets (like real returns: 0 to 50)
        N = 500
        obs = torch.randn(N, 32)
        targets = torch.rand(N) * 50  # Range [0, 50]

        optimizer = torch.optim.Adam(model.value_head.parameters(), lr=1e-3)

        for epoch in range(30):
            indices = torch.randperm(N)
            for start in range(0, N, 64):
                idx = indices[start:start + 64]
                out = model(obs[idx])
                popart.update(targets[idx])
                norm_t = popart.normalize(targets[idx])
                loss = torch.nn.functional.mse_loss(out["value"], norm_t)
                optimizer.zero_grad()
                loss.backward()
                optimizer.step()

        # Final loss should be reasonable (< 1.0)
        with torch.no_grad():
            out = model(obs[:64])
            popart.update(targets[:64])
            norm_t = popart.normalize(targets[:64])
            final_loss = torch.nn.functional.mse_loss(out["value"], norm_t).item()
        assert final_loss < 1.0, f"Value loss {final_loss} should be < 1.0 with PopArt"


class TestPretrainScript:
    """Test that the pretrain script components work."""

    def test_load_trajectories(self, tmp_path):
        """Test trajectory loading function."""
        # Create fake trajectory files
        d = tmp_path / "trajs"
        d.mkdir()
        T = 10
        np.savez_compressed(
            d / "traj_F14_seed123.npz",
            obs=np.random.randn(T, 32).astype(np.float32),
            masks=np.ones((T, 8), dtype=np.bool_),
            actions=np.zeros(T, dtype=np.int32),
            rewards=np.random.randn(T).astype(np.float32),
            dones=np.zeros(T, dtype=np.bool_),
            values=np.random.randn(T).astype(np.float32),
            log_probs=np.random.randn(T).astype(np.float32),
            final_floors=np.full(T, 14.0 / 55.0, dtype=np.float32),
        )

        # Import and test
        import sys
        sys.path.insert(0, str(Path(__file__).resolve().parent.parent.parent))
        from scripts.pretrain_value_head import load_trajectories

        obs, floors, won = load_trajectories(d, input_dim=32)
        assert len(obs) == T
        assert floors[0] == pytest.approx(14.0 / 55.0, abs=0.01)
        assert won[0] == 0.0  # Floor 14 is not a win
