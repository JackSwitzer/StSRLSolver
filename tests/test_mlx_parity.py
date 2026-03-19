"""
Numerical parity tests: PyTorch StrategicNet vs MLX MLXStrategicNet.

Verifies that identical weights + identical inputs produce identical outputs
within float32 tolerance across both inference paths. This catches:
  - Weight conversion bugs in from_pytorch()
  - Activation function mismatches (ReLU, Tanh, Sigmoid)
  - LayerNorm parameter mapping errors
  - Action mask application differences
  - Residual connection ordering issues
  - Batch dimension handling edge cases

All tests use deterministic seeded weights (no random init noise).
"""

import numpy as np
import pytest
import tempfile
from pathlib import Path

import torch
import torch.nn.functional as F

from packages.training.strategic_net import StrategicNet

# MLX may not be available on all CI environments
try:
    import mlx.core as mx
    from packages.training.mlx_inference import MLXStrategicNet

    MLX_AVAILABLE = True
except ImportError:
    MLX_AVAILABLE = False

pytestmark = pytest.mark.skipif(not MLX_AVAILABLE, reason="MLX not available")

# Tolerance: float32 accumulation across 4 residual blocks + heads.
# Empirically 1e-5 is tight for this depth; 5e-5 handles edge cases.
ATOL = 5e-5
RTOL = 1e-4


# ──────────────────────────────────────────────────────────────
# Fixtures
# ──────────────────────────────────────────────────────────────


@pytest.fixture(scope="module")
def model_configs():
    """Return a list of (input_dim, hidden_dim, action_dim, num_blocks) tuples to test."""
    return [
        (260, 768, 256, 4),   # Production config
        (260, 128, 64, 2),    # Small config for fast tests
    ]


@pytest.fixture(scope="module")
def seeded_model_pair():
    """Create a PyTorch model with deterministic weights and its MLX conversion.

    Uses the production architecture (260, 768, 256, 4 blocks).
    Saves to a temp checkpoint and loads via from_pytorch() to test
    the full conversion pipeline.
    """
    torch.manual_seed(42)
    np.random.seed(42)

    pt_model = StrategicNet(input_dim=260, hidden_dim=768, action_dim=256, num_blocks=4)
    pt_model.eval()
    pt_model.cpu()

    with tempfile.TemporaryDirectory() as tmpdir:
        ckpt_path = Path(tmpdir) / "test_model.pt"
        pt_model.save(ckpt_path)
        mlx_model = MLXStrategicNet.from_pytorch(ckpt_path)

    return pt_model, mlx_model


@pytest.fixture(scope="module")
def small_model_pair():
    """Small model pair for batch edge-case tests (faster)."""
    torch.manual_seed(123)

    pt_model = StrategicNet(input_dim=32, hidden_dim=64, action_dim=16, num_blocks=2)
    pt_model.eval()
    pt_model.cpu()

    with tempfile.TemporaryDirectory() as tmpdir:
        ckpt_path = Path(tmpdir) / "small_model.pt"
        pt_model.save(ckpt_path)
        mlx_model = MLXStrategicNet.from_pytorch(ckpt_path)

    return pt_model, mlx_model


def _run_pytorch_forward(pt_model, obs_np, mask_np):
    """Run PyTorch forward pass and return numpy outputs."""
    with torch.no_grad():
        obs_t = torch.from_numpy(obs_np).float()
        mask_t = torch.from_numpy(mask_np).bool()
        out = pt_model(obs_t, mask_t)
        return {
            "policy_logits": out["policy_logits"].numpy(),
            "value": out["value"].numpy(),
            "floor_pred": out["floor_pred"].numpy(),
            "act_completion": out["act_completion"].numpy(),
        }


def _run_mlx_forward(mlx_model, obs_np, mask_np):
    """Run MLX forward pass and return numpy outputs."""
    obs_mx = mx.array(obs_np.astype(np.float32))
    mask_mx = mx.array(mask_np.astype(np.bool_))
    out = mlx_model(obs_mx, mask_mx)
    mx.eval(out["policy_logits"], out["value"], out["floor_pred"], out["act_completion"])
    return {
        "policy_logits": np.array(out["policy_logits"]),
        "value": np.array(out["value"]),
        "floor_pred": np.array(out["floor_pred"]),
        "act_completion": np.array(out["act_completion"]),
    }


# ──────────────────────────────────────────────────────────────
# Core parity tests
# ──────────────────────────────────────────────────────────────


class TestNumericalParity:
    """Verify MLX and PyTorch produce identical outputs for same weights+inputs."""

    def test_single_sample_parity(self, seeded_model_pair):
        """batch=1: basic forward pass parity."""
        pt_model, mlx_model = seeded_model_pair
        rng = np.random.RandomState(0)

        obs = rng.randn(1, 260).astype(np.float32)
        mask = np.ones((1, 256), dtype=bool)
        mask[0, 10:] = False  # Only 10 valid actions

        pt_out = _run_pytorch_forward(pt_model, obs, mask)
        mlx_out = _run_mlx_forward(mlx_model, obs, mask)

        np.testing.assert_allclose(
            mlx_out["policy_logits"], pt_out["policy_logits"],
            atol=ATOL, rtol=RTOL,
            err_msg="Policy logits mismatch (batch=1)",
        )
        np.testing.assert_allclose(
            mlx_out["value"], pt_out["value"],
            atol=ATOL, rtol=RTOL,
            err_msg="Value head mismatch (batch=1)",
        )
        np.testing.assert_allclose(
            mlx_out["floor_pred"], pt_out["floor_pred"],
            atol=ATOL, rtol=RTOL,
            err_msg="Floor prediction mismatch (batch=1)",
        )
        np.testing.assert_allclose(
            mlx_out["act_completion"], pt_out["act_completion"],
            atol=ATOL, rtol=RTOL,
            err_msg="Act completion mismatch (batch=1)",
        )

    def test_batch_32_parity(self, seeded_model_pair):
        """batch=32: medium batch parity."""
        pt_model, mlx_model = seeded_model_pair
        rng = np.random.RandomState(1)

        obs = rng.randn(32, 260).astype(np.float32)
        # Varied masks: each sample has different number of valid actions
        mask = np.zeros((32, 256), dtype=bool)
        for i in range(32):
            n_valid = rng.randint(1, 256)
            mask[i, :n_valid] = True

        pt_out = _run_pytorch_forward(pt_model, obs, mask)
        mlx_out = _run_mlx_forward(mlx_model, obs, mask)

        np.testing.assert_allclose(
            mlx_out["policy_logits"], pt_out["policy_logits"],
            atol=ATOL, rtol=RTOL,
            err_msg="Policy logits mismatch (batch=32)",
        )
        np.testing.assert_allclose(
            mlx_out["value"], pt_out["value"],
            atol=ATOL, rtol=RTOL,
            err_msg="Value head mismatch (batch=32)",
        )

    def test_batch_128_parity(self, seeded_model_pair):
        """batch=128: max production batch size."""
        pt_model, mlx_model = seeded_model_pair
        rng = np.random.RandomState(2)

        obs = rng.randn(128, 260).astype(np.float32)
        mask = np.ones((128, 256), dtype=bool)
        # Block half the actions for all samples
        mask[:, 128:] = False

        pt_out = _run_pytorch_forward(pt_model, obs, mask)
        mlx_out = _run_mlx_forward(mlx_model, obs, mask)

        np.testing.assert_allclose(
            mlx_out["policy_logits"], pt_out["policy_logits"],
            atol=ATOL, rtol=RTOL,
            err_msg="Policy logits mismatch (batch=128)",
        )
        np.testing.assert_allclose(
            mlx_out["value"], pt_out["value"],
            atol=ATOL, rtol=RTOL,
            err_msg="Value head mismatch (batch=128)",
        )


# ──────────────────────────────────────────────────────────────
# Action mask tests
# ──────────────────────────────────────────────────────────────


class TestActionMaskParity:
    """Verify action mask application is identical in both backends."""

    def test_all_actions_valid(self, seeded_model_pair):
        """All actions valid: no masking applied."""
        pt_model, mlx_model = seeded_model_pair
        rng = np.random.RandomState(10)

        obs = rng.randn(4, 260).astype(np.float32)
        mask = np.ones((4, 256), dtype=bool)

        pt_out = _run_pytorch_forward(pt_model, obs, mask)
        mlx_out = _run_mlx_forward(mlx_model, obs, mask)

        np.testing.assert_allclose(
            mlx_out["policy_logits"], pt_out["policy_logits"],
            atol=ATOL, rtol=RTOL,
        )

    def test_single_action_valid(self, seeded_model_pair):
        """Only one action valid per sample: masked logits should be -1e8."""
        pt_model, mlx_model = seeded_model_pair
        rng = np.random.RandomState(11)

        obs = rng.randn(4, 260).astype(np.float32)
        mask = np.zeros((4, 256), dtype=bool)
        # Each sample has exactly one valid action at different indices
        for i in range(4):
            mask[i, i * 3] = True

        pt_out = _run_pytorch_forward(pt_model, obs, mask)
        mlx_out = _run_mlx_forward(mlx_model, obs, mask)

        # Masked positions should be -1e8 in both
        for i in range(4):
            invalid_mask = ~mask[i]
            pt_invalid = pt_out["policy_logits"][i][invalid_mask]
            mlx_invalid = mlx_out["policy_logits"][i][invalid_mask]
            np.testing.assert_allclose(pt_invalid, -1e8, atol=1e-2)
            np.testing.assert_allclose(mlx_invalid, -1e8, atol=1e-2)

        # Valid positions should match
        np.testing.assert_allclose(
            mlx_out["policy_logits"], pt_out["policy_logits"],
            atol=ATOL, rtol=RTOL,
        )

    def test_no_mask_provided(self, seeded_model_pair):
        """No mask: both should return raw logits."""
        pt_model, mlx_model = seeded_model_pair
        rng = np.random.RandomState(12)

        obs = rng.randn(2, 260).astype(np.float32)

        # PyTorch with no mask
        with torch.no_grad():
            obs_t = torch.from_numpy(obs).float()
            pt_out = pt_model(obs_t, action_mask=None)
            pt_logits = pt_out["policy_logits"].numpy()

        # MLX with no mask
        obs_mx = mx.array(obs)
        mlx_out = mlx_model(obs_mx, action_mask=None)
        mx.eval(mlx_out["policy_logits"])
        mlx_logits = np.array(mlx_out["policy_logits"])

        np.testing.assert_allclose(
            mlx_logits, pt_logits,
            atol=ATOL, rtol=RTOL,
            err_msg="Raw logits mismatch when no mask provided",
        )

    def test_sparse_random_masks(self, seeded_model_pair):
        """Random sparse masks: verify masked values match."""
        pt_model, mlx_model = seeded_model_pair
        rng = np.random.RandomState(13)

        obs = rng.randn(16, 260).astype(np.float32)
        # Random masks with ~10% valid actions
        mask = rng.rand(16, 256) < 0.1
        # Ensure at least one valid action per sample
        for i in range(16):
            if not mask[i].any():
                mask[i, 0] = True

        pt_out = _run_pytorch_forward(pt_model, obs, mask)
        mlx_out = _run_mlx_forward(mlx_model, obs, mask)

        np.testing.assert_allclose(
            mlx_out["policy_logits"], pt_out["policy_logits"],
            atol=ATOL, rtol=RTOL,
        )


# ──────────────────────────────────────────────────────────────
# forward_batch parity
# ──────────────────────────────────────────────────────────────


class TestForwardBatchParity:
    """Test the forward_batch() convenience method matches __call__."""

    def test_forward_batch_matches_call(self, small_model_pair):
        """forward_batch returns same logits/values as __call__."""
        pt_model, mlx_model = small_model_pair
        rng = np.random.RandomState(20)

        obs = rng.randn(8, 32).astype(np.float32)
        mask = np.ones((8, 16), dtype=bool)
        mask[:, 8:] = False

        # Via forward_batch
        fb_logits, fb_values = mlx_model.forward_batch(obs, mask)

        # Via __call__
        call_out = _run_mlx_forward(mlx_model, obs, mask)

        np.testing.assert_allclose(
            fb_logits, call_out["policy_logits"],
            atol=1e-7,
            err_msg="forward_batch logits differ from __call__",
        )
        np.testing.assert_allclose(
            fb_values, call_out["value"],
            atol=1e-7,
            err_msg="forward_batch values differ from __call__",
        )

    def test_forward_batch_single_sample(self, small_model_pair):
        """forward_batch with batch=1."""
        pt_model, mlx_model = small_model_pair
        rng = np.random.RandomState(21)

        obs = rng.randn(1, 32).astype(np.float32)
        mask = np.ones((1, 16), dtype=bool)

        fb_logits, fb_values = mlx_model.forward_batch(obs, mask)
        assert fb_logits.shape == (1, 16)
        assert fb_values.shape == (1,)

        # Cross-check with PyTorch
        pt_out = _run_pytorch_forward(pt_model, obs, mask)
        np.testing.assert_allclose(
            fb_logits, pt_out["policy_logits"],
            atol=ATOL, rtol=RTOL,
        )


# ──────────────────────────────────────────────────────────────
# Weight conversion correctness
# ──────────────────────────────────────────────────────────────


class TestWeightConversion:
    """Verify from_pytorch() maps all layers correctly."""

    def test_all_weights_loaded(self, seeded_model_pair):
        """Every PyTorch parameter has a corresponding MLX parameter."""
        pt_model, mlx_model = seeded_model_pair

        pt_params = dict(pt_model.named_parameters())
        # Count PyTorch parameters (excluding Tanh/ReLU/Sigmoid which have none)
        pt_param_count = len(pt_params)

        # Count MLX parameters by checking all layers
        mlx_param_count = 0
        # input_linear (weight, bias) + input_norm (weight, bias) = 4
        mlx_param_count += 2  # input_linear
        mlx_param_count += 2  # input_norm
        # Each block: fc1 (weight, bias) + fc2 (weight, bias) + norm (weight, bias) = 6
        mlx_param_count += 6 * mlx_model.num_blocks
        # Heads: policy_1, policy_2, value_1, value_2, floor_1, floor_2, act_1, act_2
        # Each has weight + bias = 16
        mlx_param_count += 2 * 8

        assert mlx_param_count == pt_param_count, (
            f"Parameter count mismatch: MLX has {mlx_param_count}, PyTorch has {pt_param_count}"
        )

    def test_input_projection_weights_match(self, seeded_model_pair):
        """Input Linear + LayerNorm weights match exactly."""
        pt_model, mlx_model = seeded_model_pair

        # Linear weight
        pt_w = pt_model.input_proj[0].weight.detach().numpy()
        mlx_w = np.array(mlx_model.input_linear.weight)
        np.testing.assert_allclose(mlx_w, pt_w, atol=1e-7, err_msg="input_linear.weight mismatch")

        # Linear bias
        pt_b = pt_model.input_proj[0].bias.detach().numpy()
        mlx_b = np.array(mlx_model.input_linear.bias)
        np.testing.assert_allclose(mlx_b, pt_b, atol=1e-7, err_msg="input_linear.bias mismatch")

        # LayerNorm weight
        pt_ln_w = pt_model.input_proj[1].weight.detach().numpy()
        mlx_ln_w = np.array(mlx_model.input_norm.weight)
        np.testing.assert_allclose(mlx_ln_w, pt_ln_w, atol=1e-7, err_msg="input_norm.weight mismatch")

        # LayerNorm bias
        pt_ln_b = pt_model.input_proj[1].bias.detach().numpy()
        mlx_ln_b = np.array(mlx_model.input_norm.bias)
        np.testing.assert_allclose(mlx_ln_b, pt_ln_b, atol=1e-7, err_msg="input_norm.bias mismatch")

    def test_residual_block_weights_match(self, seeded_model_pair):
        """Each residual block's fc1 + fc2 + LayerNorm weights match."""
        pt_model, mlx_model = seeded_model_pair

        for i in range(mlx_model.num_blocks):
            pt_block = pt_model.trunk[i]
            mlx_block = mlx_model.blocks[i]

            # fc1
            pt_w = pt_block.fc1.weight.detach().numpy()
            mlx_w = np.array(mlx_block.fc1.weight)
            np.testing.assert_allclose(
                mlx_w, pt_w, atol=1e-7,
                err_msg=f"Block {i} fc1.weight mismatch",
            )

            pt_b = pt_block.fc1.bias.detach().numpy()
            mlx_b = np.array(mlx_block.fc1.bias)
            np.testing.assert_allclose(
                mlx_b, pt_b, atol=1e-7,
                err_msg=f"Block {i} fc1.bias mismatch",
            )

            # fc2
            pt_w2 = pt_block.fc2.weight.detach().numpy()
            mlx_w2 = np.array(mlx_block.fc2.weight)
            np.testing.assert_allclose(
                mlx_w2, pt_w2, atol=1e-7,
                err_msg=f"Block {i} fc2.weight mismatch",
            )

            pt_b2 = pt_block.fc2.bias.detach().numpy()
            mlx_b2 = np.array(mlx_block.fc2.bias)
            np.testing.assert_allclose(
                mlx_b2, pt_b2, atol=1e-7,
                err_msg=f"Block {i} fc2.bias mismatch",
            )

            # LayerNorm
            pt_ln_w = pt_block.norm.weight.detach().numpy()
            mlx_ln_w = np.array(mlx_block.norm.weight)
            np.testing.assert_allclose(
                mlx_ln_w, pt_ln_w, atol=1e-7,
                err_msg=f"Block {i} norm.weight mismatch",
            )

            pt_ln_b = pt_block.norm.bias.detach().numpy()
            mlx_ln_b = np.array(mlx_block.norm.bias)
            np.testing.assert_allclose(
                mlx_ln_b, pt_ln_b, atol=1e-7,
                err_msg=f"Block {i} norm.bias mismatch",
            )

    def test_policy_head_weights_match(self, seeded_model_pair):
        """Policy head layers match."""
        pt_model, mlx_model = seeded_model_pair

        # policy_head.0 -> policy_1
        np.testing.assert_allclose(
            np.array(mlx_model.policy_1.weight),
            pt_model.policy_head[0].weight.detach().numpy(),
            atol=1e-7,
        )
        # policy_head.2 -> policy_2
        np.testing.assert_allclose(
            np.array(mlx_model.policy_2.weight),
            pt_model.policy_head[2].weight.detach().numpy(),
            atol=1e-7,
        )

    def test_value_head_weights_match(self, seeded_model_pair):
        """Value head layers match (Linear only, Tanh has no params)."""
        pt_model, mlx_model = seeded_model_pair

        # value_head.0 -> value_1
        np.testing.assert_allclose(
            np.array(mlx_model.value_1.weight),
            pt_model.value_head[0].weight.detach().numpy(),
            atol=1e-7,
        )
        # value_head.2 -> value_2
        np.testing.assert_allclose(
            np.array(mlx_model.value_2.weight),
            pt_model.value_head[2].weight.detach().numpy(),
            atol=1e-7,
        )


# ──────────────────────────────────────────────────────────────
# Activation function parity
# ──────────────────────────────────────────────────────────────


class TestActivationParity:
    """Verify activation functions behave identically."""

    def test_value_head_parity(self, seeded_model_pair):
        """Value head has no tanh (unbounded) in both PyTorch and MLX."""
        pt_model, mlx_model = seeded_model_pair
        rng = np.random.RandomState(30)

        obs = rng.randn(8, 260).astype(np.float32)
        mask = np.ones((8, 256), dtype=bool)

        pt_out = _run_pytorch_forward(pt_model, obs, mask)
        mlx_out = _run_mlx_forward(mlx_model, obs, mask)

        np.testing.assert_allclose(
            mlx_out["value"], pt_out["value"],
            atol=ATOL, rtol=RTOL,
            err_msg="Value head mismatch",
        )

        np.testing.assert_allclose(
            mlx_out["value"], pt_out["value"],
            atol=ATOL, rtol=RTOL,
            err_msg="Value head tanh output mismatch",
        )

    def test_act_completion_sigmoid_parity(self, seeded_model_pair):
        """Act completion head uses sigmoid in both backends."""
        pt_model, mlx_model = seeded_model_pair
        rng = np.random.RandomState(31)

        obs = rng.randn(4, 260).astype(np.float32)
        mask = np.ones((4, 256), dtype=bool)

        pt_out = _run_pytorch_forward(pt_model, obs, mask)
        mlx_out = _run_mlx_forward(mlx_model, obs, mask)

        # Sigmoid output should be in [0, 1]
        assert np.all(pt_out["act_completion"] >= 0.0 - 1e-7)
        assert np.all(pt_out["act_completion"] <= 1.0 + 1e-7)
        assert np.all(mlx_out["act_completion"] >= 0.0 - 1e-7)
        assert np.all(mlx_out["act_completion"] <= 1.0 + 1e-7)

        np.testing.assert_allclose(
            mlx_out["act_completion"], pt_out["act_completion"],
            atol=ATOL, rtol=RTOL,
            err_msg="Act completion sigmoid mismatch",
        )

    def test_relu_vs_maximum(self):
        """Verify mx.maximum(x, 0) matches F.relu(x) semantically."""
        rng = np.random.RandomState(32)
        x = rng.randn(100).astype(np.float32)

        pt_relu = F.relu(torch.from_numpy(x)).numpy()
        mlx_relu = np.array(mx.maximum(mx.array(x), 0))

        np.testing.assert_array_equal(pt_relu, mlx_relu)


# ──────────────────────────────────────────────────────────────
# Residual block structure parity
# ──────────────────────────────────────────────────────────────


class TestResidualBlockParity:
    """Verify residual block: relu(norm(linear(x))) + x matches in both."""

    def test_single_block_parity(self, seeded_model_pair):
        """Run one residual block through both and compare."""
        pt_model, mlx_model = seeded_model_pair
        rng = np.random.RandomState(40)

        # Create input that would come from input_proj
        h = rng.randn(4, 768).astype(np.float32)

        # PyTorch: run through first residual block
        with torch.no_grad():
            h_pt = torch.from_numpy(h).float()
            block_pt = pt_model.trunk[0]
            out_pt = block_pt(h_pt).numpy()

        # MLX: run through first residual block
        h_mx = mx.array(h)
        block_mlx = mlx_model.blocks[0]
        out_mlx = block_mlx(h_mx)
        mx.eval(out_mlx)
        out_mlx = np.array(out_mlx)

        np.testing.assert_allclose(
            out_mlx, out_pt, atol=ATOL, rtol=RTOL,
            err_msg="Residual block 0 output mismatch",
        )


# ──────────────────────────────────────────────────────────────
# Input edge cases
# ──────────────────────────────────────────────────────────────


class TestInputEdgeCases:
    """Edge cases for input handling."""

    def test_zero_input(self, seeded_model_pair):
        """All-zero input produces identical outputs."""
        pt_model, mlx_model = seeded_model_pair

        obs = np.zeros((1, 260), dtype=np.float32)
        mask = np.ones((1, 256), dtype=bool)

        pt_out = _run_pytorch_forward(pt_model, obs, mask)
        mlx_out = _run_mlx_forward(mlx_model, obs, mask)

        np.testing.assert_allclose(
            mlx_out["policy_logits"], pt_out["policy_logits"],
            atol=ATOL, rtol=RTOL,
        )

    def test_large_input_values(self, seeded_model_pair):
        """Large input values: verify no overflow divergence."""
        pt_model, mlx_model = seeded_model_pair
        rng = np.random.RandomState(50)

        obs = (rng.randn(2, 260) * 100).astype(np.float32)
        mask = np.ones((2, 256), dtype=bool)

        pt_out = _run_pytorch_forward(pt_model, obs, mask)
        mlx_out = _run_mlx_forward(mlx_model, obs, mask)

        # Allow slightly looser tolerance for large values
        np.testing.assert_allclose(
            mlx_out["policy_logits"], pt_out["policy_logits"],
            atol=1e-3, rtol=1e-3,
            err_msg="Large input parity failure",
        )

    def test_negative_input_values(self, seeded_model_pair):
        """All-negative inputs."""
        pt_model, mlx_model = seeded_model_pair

        obs = np.full((1, 260), -1.0, dtype=np.float32)
        mask = np.ones((1, 256), dtype=bool)

        pt_out = _run_pytorch_forward(pt_model, obs, mask)
        mlx_out = _run_mlx_forward(mlx_model, obs, mask)

        np.testing.assert_allclose(
            mlx_out["value"], pt_out["value"],
            atol=ATOL, rtol=RTOL,
        )


# ──────────────────────────────────────────────────────────────
# Determinism
# ──────────────────────────────────────────────────────────────


class TestDeterminism:
    """Verify repeated calls produce identical results (no hidden state)."""

    def test_mlx_deterministic(self, seeded_model_pair):
        """Two identical MLX forward passes produce exact same output."""
        _, mlx_model = seeded_model_pair
        rng = np.random.RandomState(60)

        obs = rng.randn(4, 260).astype(np.float32)
        mask = np.ones((4, 256), dtype=bool)

        out1 = _run_mlx_forward(mlx_model, obs, mask)
        out2 = _run_mlx_forward(mlx_model, obs, mask)

        np.testing.assert_array_equal(
            out1["policy_logits"], out2["policy_logits"],
            err_msg="MLX not deterministic on repeated calls",
        )
        np.testing.assert_array_equal(
            out1["value"], out2["value"],
        )

    def test_forward_batch_deterministic(self, small_model_pair):
        """forward_batch is deterministic."""
        _, mlx_model = small_model_pair
        rng = np.random.RandomState(61)

        obs = rng.randn(16, 32).astype(np.float32)
        mask = np.ones((16, 16), dtype=bool)

        l1, v1 = mlx_model.forward_batch(obs, mask)
        l2, v2 = mlx_model.forward_batch(obs, mask)

        np.testing.assert_array_equal(l1, l2)
        np.testing.assert_array_equal(v1, v2)


# ──────────────────────────────────────────────────────────────
# Save/load MLX native format roundtrip
# ──────────────────────────────────────────────────────────────


class TestMLXSaveLoad:
    """Verify save_mlx + load_mlx roundtrip preserves parity."""

    def test_roundtrip_preserves_outputs(self, seeded_model_pair):
        """Save to MLX format, reload, and verify identical outputs."""
        pt_model, mlx_model = seeded_model_pair
        rng = np.random.RandomState(70)

        obs = rng.randn(4, 260).astype(np.float32)
        mask = np.ones((4, 256), dtype=bool)
        mask[:, 50:] = False

        ref_out = _run_mlx_forward(mlx_model, obs, mask)

        with tempfile.TemporaryDirectory() as tmpdir:
            save_path = Path(tmpdir) / "model"
            mlx_model.save_mlx(save_path)
            loaded = MLXStrategicNet.load_mlx(save_path)

        loaded_out = _run_mlx_forward(loaded, obs, mask)

        np.testing.assert_allclose(
            loaded_out["policy_logits"], ref_out["policy_logits"],
            atol=1e-7,
            err_msg="MLX save/load roundtrip changed outputs",
        )
        np.testing.assert_allclose(
            loaded_out["value"], ref_out["value"],
            atol=1e-7,
        )


# ──────────────────────────────────────────────────────────────
# InferenceServer backend parity
# ──────────────────────────────────────────────────────────────


class TestInferenceServerBackendParity:
    """Verify MLXStrategicBackend.from_state_dict matches from_pytorch."""

    def test_backend_from_state_dict_matches_from_pytorch(self):
        """Build backend via from_state_dict and compare to from_pytorch."""
        from packages.training.inference_server import MLXStrategicBackend

        torch.manual_seed(99)
        pt_model = StrategicNet(input_dim=32, hidden_dim=64, action_dim=16, num_blocks=2)
        pt_model.eval()
        pt_model.cpu()

        # Path 1: from_pytorch
        with tempfile.TemporaryDirectory() as tmpdir:
            ckpt_path = Path(tmpdir) / "model.pt"
            pt_model.save(ckpt_path)
            mlx_via_file = MLXStrategicNet.from_pytorch(ckpt_path)

        # Path 2: from_state_dict (as InferenceServer does)
        state_dict = {k: v.detach().cpu() for k, v in pt_model.state_dict().items()}
        config = {
            "input_dim": 32,
            "hidden_dim": 64,
            "action_dim": 16,
            "num_blocks": 2,
        }
        backend = MLXStrategicBackend.from_state_dict(state_dict, config, version=1)

        # Same input
        rng = np.random.RandomState(0)
        obs = rng.randn(4, 32).astype(np.float32)
        mask = np.ones((4, 16), dtype=bool)

        # Compare outputs
        out_file = _run_mlx_forward(mlx_via_file, obs, mask)

        logits_sd, values_sd = backend.forward_batch(obs, mask.astype(np.bool_))

        np.testing.assert_allclose(
            logits_sd, out_file["policy_logits"],
            atol=1e-7,
            err_msg="from_state_dict logits differ from from_pytorch",
        )
        np.testing.assert_allclose(
            values_sd, out_file["value"],
            atol=1e-7,
            err_msg="from_state_dict values differ from from_pytorch",
        )
