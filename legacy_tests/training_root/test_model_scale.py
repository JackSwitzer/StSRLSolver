"""Tests for model scaling -- verify 1024 hidden, 8 blocks works end-to-end."""

import numpy as np
import pytest


class TestModelScale:
    def test_strategic_net_1024(self):
        """StrategicNet with 1024 hidden, 8 blocks creates and forward passes."""
        import torch

        from packages.training.strategic_net import StrategicNet

        model = StrategicNet(input_dim=480, action_dim=512, hidden_dim=1024, num_blocks=8)
        params = sum(p.numel() for p in model.parameters())
        assert params > 10_000_000  # Should be ~18M

        # Forward pass
        x = torch.randn(4, 480)
        mask = torch.ones(4, 512, dtype=torch.bool)
        out = model(x, mask)
        assert out["policy_logits"].shape == (4, 512)
        assert out["value"].shape == (4,)

    def test_mlx_backend_1024(self):
        """MLX backend loads 1024-dim model from PyTorch state dict."""
        try:
            import mlx.core as mx  # noqa: F401

            from packages.training.inference_server import MLXStrategicBackend
        except ImportError:
            pytest.skip("MLX not available")

        import torch

        from packages.training.strategic_net import StrategicNet

        model = StrategicNet(input_dim=480, action_dim=512, hidden_dim=1024, num_blocks=8)
        state_dict = {k: v.detach().cpu() for k, v in model.state_dict().items()}
        config = {
            "input_dim": 480,
            "hidden_dim": 1024,
            "action_dim": 512,
            "num_blocks": 8,
        }

        backend = MLXStrategicBackend.from_state_dict(state_dict, config, version=1)

        obs = np.random.randn(4, 480).astype(np.float32)
        mask = np.ones((4, 512), dtype=np.bool_)
        logits, values = backend.forward_batch(obs, mask)
        assert logits.shape == (4, 512)
        assert values.shape == (4,)

    def test_param_count_matches_config(self):
        """Model param count matches what training_config specifies."""
        from packages.training.strategic_net import StrategicNet
        from packages.training.training_config import MODEL_HIDDEN_DIM, MODEL_NUM_BLOCKS

        model = StrategicNet(
            input_dim=480,
            action_dim=512,
            hidden_dim=MODEL_HIDDEN_DIM,
            num_blocks=MODEL_NUM_BLOCKS,
        )
        params = sum(p.numel() for p in model.parameters())
        # Should be ~18M for 1024 hidden, 8 blocks
        assert params > 15_000_000

    def test_pytorch_mlx_output_parity(self):
        """PyTorch and MLX models produce matching outputs from same weights."""
        try:
            import mlx.core as mx  # noqa: F401

            from packages.training.inference_server import MLXStrategicBackend
        except ImportError:
            pytest.skip("MLX not available")

        import torch

        from packages.training.strategic_net import StrategicNet

        torch.manual_seed(42)
        model = StrategicNet(input_dim=480, action_dim=512, hidden_dim=1024, num_blocks=8)
        model.eval()

        state_dict = {k: v.detach().cpu() for k, v in model.state_dict().items()}
        config = {
            "input_dim": 480,
            "hidden_dim": 1024,
            "action_dim": 512,
            "num_blocks": 8,
        }
        backend = MLXStrategicBackend.from_state_dict(state_dict, config, version=1)

        np.random.seed(42)
        obs = np.random.randn(2, 480).astype(np.float32)
        mask = np.ones((2, 512), dtype=np.bool_)

        # PyTorch forward
        with torch.no_grad():
            pt_out = model(torch.from_numpy(obs), torch.from_numpy(mask))
            pt_logits = pt_out["policy_logits"].numpy()
            pt_values = pt_out["value"].numpy()

        # MLX forward
        mlx_logits, mlx_values = backend.forward_batch(obs, mask)

        np.testing.assert_allclose(pt_logits, mlx_logits, atol=1e-4, rtol=1e-4)
        np.testing.assert_allclose(pt_values, mlx_values, atol=1e-4, rtol=1e-4)
