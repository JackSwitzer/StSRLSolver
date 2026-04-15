"""Tests for auxiliary heads in StrategicNet and StrategicTrainer."""
import numpy as np
import pytest
import torch

from packages.training.strategic_net import StrategicNet
from packages.training.strategic_trainer import StrategicTrainer, StrategicTransition
from packages.training.training_config import AUX_HEADS


class TestAuxiliaryHeads:
    """Test new auxiliary heads produce valid outputs."""

    def test_all_new_heads_present_in_output(self):
        """Forward pass includes all enabled aux heads."""
        model = StrategicNet(input_dim=480, hidden_dim=64, action_dim=16, num_blocks=2)
        x = torch.randn(4, 480)
        out = model(x)

        for head_name in ("deck_quality", "combat_horizon", "win_loss", "boss_ready"):
            if AUX_HEADS.get(head_name, 0) > 0:
                assert head_name in out, f"{head_name} missing from output"

    def test_head_shapes(self):
        """Each aux head produces [batch] shape."""
        model = StrategicNet(input_dim=480, hidden_dim=64, action_dim=16, num_blocks=2)
        batch_size = 8
        x = torch.randn(batch_size, 480)
        out = model(x)

        for head_name in ("deck_quality", "combat_horizon", "win_loss", "boss_ready"):
            if head_name in out:
                assert out[head_name].shape == (batch_size,), \
                    f"{head_name} shape {out[head_name].shape} != ({batch_size},)"

    def test_no_nan_in_outputs(self):
        """No NaN values in any head output."""
        model = StrategicNet(input_dim=480, hidden_dim=64, action_dim=16, num_blocks=2)
        x = torch.randn(4, 480)
        out = model(x)

        for key, val in out.items():
            assert not torch.isnan(val).any(), f"NaN in {key}"

    def test_sigmoid_heads_bounded(self):
        """win_loss and boss_ready outputs are in [0, 1]."""
        model = StrategicNet(input_dim=480, hidden_dim=64, action_dim=16, num_blocks=2)
        x = torch.randn(16, 480)
        out = model(x)

        for head_name in ("win_loss", "boss_ready"):
            if head_name in out:
                assert (out[head_name] >= 0).all(), f"{head_name} has values < 0"
                assert (out[head_name] <= 1).all(), f"{head_name} has values > 1"

    def test_disabled_heads_not_in_output(self):
        """Heads with weight=0 should not appear in output."""
        model = StrategicNet(
            input_dim=480, hidden_dim=64, action_dim=16, num_blocks=2,
            enabled_aux_heads={"floor_pred", "act_completion"},
        )
        x = torch.randn(4, 480)
        out = model(x)

        for head_name in ("deck_quality", "combat_horizon", "win_loss", "boss_ready"):
            assert head_name not in out, f"{head_name} should be disabled"

    def test_gradients_flow_through_new_heads(self):
        """Gradients flow from new head losses back through trunk."""
        model = StrategicNet(input_dim=480, hidden_dim=64, action_dim=16, num_blocks=2)
        x = torch.randn(4, 480)
        out = model(x)

        # Sum all new head outputs and backprop
        loss = torch.tensor(0.0)
        for head_name in ("deck_quality", "combat_horizon", "win_loss", "boss_ready"):
            if head_name in out:
                loss = loss + out[head_name].sum()

        loss.backward()

        # Check trunk has gradients
        trunk_grad = model.trunk[0].fc1.weight.grad
        assert trunk_grad is not None, "No gradient in trunk"
        assert (trunk_grad != 0).any(), "Trunk gradients are all zero"

    def test_param_count_increases_with_heads(self):
        """More enabled heads = more parameters."""
        model_none = StrategicNet(
            input_dim=480, hidden_dim=64, action_dim=16, num_blocks=2,
            enabled_aux_heads={"floor_pred", "act_completion"},
        )
        model_all = StrategicNet(
            input_dim=480, hidden_dim=64, action_dim=16, num_blocks=2,
        )
        assert model_all.param_count() > model_none.param_count()


class TestTrainerAuxLoss:
    """Test that trainer loss computation includes new head losses."""

    def _make_trainer(self):
        model = StrategicNet(input_dim=32, hidden_dim=32, action_dim=8, num_blocks=1)
        trainer = StrategicTrainer(
            model=model, lr=1e-3, batch_size=4, ppo_epochs=1, warmup_steps=0,
        )
        return trainer

    def _fill_buffer(self, trainer, n=8):
        for i in range(n):
            t = StrategicTransition(
                obs=np.random.randn(32).astype(np.float32),
                action_mask=np.ones(8, dtype=bool),
                action=np.random.randint(0, 8),
                reward=np.random.randn(),
                done=(i == n - 1),
                value=np.random.randn(),
                log_prob=-1.0,
                episode_id=0,
                final_floor=0.3,
                cleared_act1=1.0,
                cleared_act2=0.0,
                cleared_act3=0.0,
                deck_quality=0.3,
                combat_horizon=5.0,
                won_run=0.0,
                boss_ready=0.0,
            )
            trainer.buffer.append(t)

    def test_train_batch_includes_new_metrics(self):
        """train_batch returns metrics for new aux heads."""
        trainer = self._make_trainer()
        self._fill_buffer(trainer)
        metrics = trainer.train_batch()

        for key in ("deck_quality_loss", "combat_horizon_loss", "win_loss_loss", "boss_ready_loss"):
            assert key in metrics, f"Missing metric: {key}"

    def test_train_batch_no_crash(self):
        """Full training step completes without error."""
        trainer = self._make_trainer()
        self._fill_buffer(trainer, n=16)
        metrics = trainer.train_batch()
        assert metrics["total_loss"] > 0

    def test_aux_loss_nonzero(self):
        """Combined aux loss should be nonzero with active heads."""
        trainer = self._make_trainer()
        self._fill_buffer(trainer, n=16)
        metrics = trainer.train_batch()
        assert metrics["aux_loss"] > 0


class TestSaveLoadWithAuxHeads:
    """Test save/load round-trip with new aux heads."""

    def test_save_load_roundtrip(self, tmp_path):
        model = StrategicNet(input_dim=32, hidden_dim=32, action_dim=8, num_blocks=1)
        path = tmp_path / "test_model.pt"
        model.save(path)

        loaded = StrategicNet.load(path, device=torch.device("cpu"))
        assert loaded._enabled_aux == model._enabled_aux

        # Forward pass should work
        x = torch.randn(2, 32)
        out_orig = model(x)
        out_loaded = loaded(x)
        for key in out_orig:
            assert key in out_loaded


class TestAuxHeadsConfig:
    """Test AUX_HEADS config is properly structured."""

    def test_config_has_all_heads(self):
        expected = {"floor_pred", "act_completion", "deck_quality", "combat_horizon", "win_loss", "boss_ready"}
        assert set(AUX_HEADS.keys()) == expected

    def test_config_weights_positive(self):
        for name, weight in AUX_HEADS.items():
            assert isinstance(weight, (int, float)), f"{name} weight is not numeric"
            assert weight >= 0, f"{name} weight is negative"
