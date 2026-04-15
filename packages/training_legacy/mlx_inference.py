"""
MLX inference engine for fast Apple Silicon inference.

Converts PyTorch StrategicNet weights to MLX format for ~5-10x faster
inference on M-series chips. Training stays in PyTorch; inference uses MLX.

Usage:
    from packages.training.mlx_inference import MLXStrategicNet

    # Load from PyTorch checkpoint
    mlx_net = MLXStrategicNet.from_pytorch("logs/strategic_checkpoints/latest_strategic.pt")

    # Inference (numpy in/out)
    action_idx, value, probs = mlx_net.predict_action(obs_np, action_mask_np)

    # Batch inference
    logits, values = mlx_net.forward_batch(obs_batch_np, mask_batch_np)
"""

from __future__ import annotations

from pathlib import Path
from typing import Tuple

import numpy as np

try:
    import mlx.core as mx
    import mlx.nn as nn

    MLX_AVAILABLE = True
except ImportError:
    MLX_AVAILABLE = False


def check_mlx():
    if not MLX_AVAILABLE:
        raise RuntimeError("MLX not available. Install with: uv add mlx")


def _as_float32_array(values: np.ndarray) -> np.ndarray:
    """Return a contiguous float32 array without copying when possible."""
    return np.ascontiguousarray(np.asarray(values, dtype=np.float32))


def _as_bool_array(values: np.ndarray) -> np.ndarray:
    """Return a contiguous bool array without copying when possible."""
    return np.ascontiguousarray(np.asarray(values, dtype=np.bool_))


class MLXResidualBlock:
    """Residual block: two-layer MLP + LayerNorm with skip connection.

    fc1 -> ReLU -> fc2 -> LayerNorm -> + skip
    """

    def __init__(self, dim: int):
        self.fc1 = nn.Linear(dim, dim)
        self.fc2 = nn.Linear(dim, dim)
        self.norm = nn.LayerNorm(dim)

    def __call__(self, x):
        h = mx.maximum(self.fc1(x), 0)
        return self.norm(self.fc2(h)) + x

    def parameters(self):
        return {
            "fc1": self.fc1.parameters(),
            "fc2": self.fc2.parameters(),
            "norm": self.norm.parameters(),
        }

    def update(self, params):
        self.fc1.update(params["fc1"])
        self.fc2.update(params["fc2"])
        self.norm.update(params["norm"])


class MLXStrategicNet:
    """MLX implementation of StrategicNet for fast Apple Silicon inference.

    Architecture mirrors PyTorch StrategicNet exactly:
    - Input projection: input_dim -> hidden_dim (Linear + LayerNorm + ReLU)
    - N residual blocks
    - Multi-head output: policy, value, floor, act_completion
    """

    def __init__(
        self,
        input_dim: int = 480,
        hidden_dim: int = 1024,
        action_dim: int = 512,
        num_blocks: int = 8,
    ):
        check_mlx()
        self.input_dim = input_dim
        self.hidden_dim = hidden_dim
        self.action_dim = action_dim
        self.num_blocks = num_blocks

        # Input projection
        self.input_linear = nn.Linear(input_dim, hidden_dim)
        self.input_norm = nn.LayerNorm(hidden_dim)

        # Residual trunk
        self.blocks = [MLXResidualBlock(hidden_dim) for _ in range(num_blocks)]

        # Policy head
        self.policy_1 = nn.Linear(hidden_dim, 256)
        self.policy_2 = nn.Linear(256, action_dim)

        # Value head
        self.value_1 = nn.Linear(hidden_dim, 64)
        self.value_2 = nn.Linear(64, 1)

        # Floor prediction head
        self.floor_1 = nn.Linear(hidden_dim, 64)
        self.floor_2 = nn.Linear(64, 1)

        # Act completion head
        self.act_1 = nn.Linear(hidden_dim, 64)
        self.act_2 = nn.Linear(64, 3)

    def __call__(self, x, action_mask=None):
        """Forward pass.

        Args:
            x: [batch, input_dim] input tensor
            action_mask: [batch, action_dim] bool tensor (True=valid)

        Returns:
            dict with policy_logits, value, floor_pred, act_completion
        """
        # Input projection
        h = mx.maximum(self.input_norm(self.input_linear(x)), 0)

        # Trunk
        for block in self.blocks:
            h = block(h)

        # Policy
        logits = self.policy_2(mx.maximum(self.policy_1(h), 0))
        if action_mask is not None:
            logits = mx.where(action_mask, logits, mx.array(-1e8))

        # Value (no tanh — rewards can exceed [-1, 1])
        value = self.value_2(mx.maximum(self.value_1(h), 0)).squeeze(-1)

        # Floor
        floor_pred = self.floor_2(mx.maximum(self.floor_1(h), 0)).squeeze(-1)

        # Act completion (sigmoid)
        act_completion = mx.sigmoid(self.act_2(mx.maximum(self.act_1(h), 0)))

        return {
            "policy_logits": logits,
            "value": value,
            "floor_pred": floor_pred,
            "act_completion": act_completion,
        }

    def predict_action(
        self,
        obs_np: np.ndarray,
        action_mask_np: np.ndarray,
        temperature: float = 0.0,
    ) -> Tuple[int, float, np.ndarray]:
        """Predict action from numpy observation.

        Returns:
            (action_index, value, policy_probs)
        """
        obs = mx.array(obs_np[np.newaxis].astype(np.float32))
        mask = mx.array(action_mask_np[np.newaxis].astype(np.bool_))

        out = self(obs, mask)
        logits = out["policy_logits"][0]

        if temperature > 0:
            scaled = logits / temperature
            probs = mx.softmax(scaled)
            # Sample from distribution
            action = int(mx.random.categorical(mx.log(probs + 1e-8)).item())
        else:
            action = int(mx.argmax(logits).item())

        probs = np.array(mx.softmax(logits))
        val = float(out["value"][0].item())

        return action, val, probs

    def forward_batch(
        self,
        obs_batch: np.ndarray,
        mask_batch: np.ndarray,
    ) -> Tuple[np.ndarray, np.ndarray]:
        """Batch forward pass for parallel workers.

        Args:
            obs_batch: [N, input_dim] numpy array
            mask_batch: [N, action_dim] numpy bool array

        Returns:
            (logits [N, action_dim], values [N]) as numpy arrays
        """
        obs = mx.array(obs_batch.astype(np.float32))
        mask = mx.array(mask_batch.astype(np.bool_))

        out = self(obs, mask)
        mx.eval(out["policy_logits"], out["value"])

        return np.array(out["policy_logits"]), np.array(out["value"])

    @classmethod
    def from_pytorch(cls, path: str | Path, **kwargs) -> "MLXStrategicNet":
        """Load from a PyTorch StrategicNet checkpoint.

        Converts PyTorch state_dict to MLX arrays.
        """
        check_mlx()
        import torch

        checkpoint = torch.load(path, map_location="cpu", weights_only=True)
        config = checkpoint["config"]
        state_dict = checkpoint["model_state_dict"]

        net = cls(
            input_dim=config["input_dim"],
            hidden_dim=config["hidden_dim"],
            action_dim=config["action_dim"],
            num_blocks=config["num_blocks"],
        )

        # Convert PyTorch state dict to MLX weights
        _load_linear(net.input_linear, state_dict, "input_proj.0")
        _load_layernorm(net.input_norm, state_dict, "input_proj.1")

        for i, block in enumerate(net.blocks):
            _load_linear(block.fc1, state_dict, f"trunk.{i}.fc1")
            _load_linear(block.fc2, state_dict, f"trunk.{i}.fc2")
            _load_layernorm(block.norm, state_dict, f"trunk.{i}.norm")

        _load_linear(net.policy_1, state_dict, "policy_head.0")
        _load_linear(net.policy_2, state_dict, "policy_head.2")

        _load_linear(net.value_1, state_dict, "value_head.0")
        _load_linear(net.value_2, state_dict, "value_head.2")

        _load_linear(net.floor_1, state_dict, "floor_head.0")
        _load_linear(net.floor_2, state_dict, "floor_head.2")

        _load_linear(net.act_1, state_dict, "act_head.0")
        _load_linear(net.act_2, state_dict, "act_head.2")

        return net

    def save_mlx(self, path: str | Path) -> None:
        """Save MLX weights to disk."""
        check_mlx()
        import json

        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)

        weights = _collect_weights(self)
        mx.savez(str(path.with_suffix(".npz")), **weights)

        config = {
            "input_dim": self.input_dim,
            "hidden_dim": self.hidden_dim,
            "action_dim": self.action_dim,
            "num_blocks": self.num_blocks,
        }
        path.with_suffix(".json").write_text(json.dumps(config))

    @classmethod
    def load_mlx(cls, path: str | Path) -> "MLXStrategicNet":
        """Load from MLX-native format (npz + json config)."""
        check_mlx()
        import json

        path = Path(path)
        config = json.loads(path.with_suffix(".json").read_text())
        net = cls(**config)

        data = mx.load(str(path.with_suffix(".npz")))
        _apply_weights(net, data)

        return net


def _torch_to_mx(tensor) -> "mx.array":
    """Convert PyTorch tensor to MLX array."""
    return mx.array(tensor.detach().cpu().numpy())


def _load_linear(mlx_linear, state_dict: dict, prefix: str):
    """Load a PyTorch Linear layer's weights into an MLX Linear."""
    mlx_linear.weight = _torch_to_mx(state_dict[f"{prefix}.weight"])
    if f"{prefix}.bias" in state_dict:
        mlx_linear.bias = _torch_to_mx(state_dict[f"{prefix}.bias"])


def _load_layernorm(mlx_ln, state_dict: dict, prefix: str):
    """Load a PyTorch LayerNorm's weights into an MLX LayerNorm."""
    mlx_ln.weight = _torch_to_mx(state_dict[f"{prefix}.weight"])
    mlx_ln.bias = _torch_to_mx(state_dict[f"{prefix}.bias"])


def _collect_weights(net: MLXStrategicNet) -> dict:
    """Collect all weights as a flat dict for saving."""
    weights = {}

    def _add(prefix, layer):
        if hasattr(layer, "weight"):
            weights[f"{prefix}.weight"] = layer.weight
        if hasattr(layer, "bias") and layer.bias is not None:
            weights[f"{prefix}.bias"] = layer.bias

    _add("input_linear", net.input_linear)
    _add("input_norm", net.input_norm)

    for i, block in enumerate(net.blocks):
        _add(f"block.{i}.fc1", block.fc1)
        _add(f"block.{i}.fc2", block.fc2)
        _add(f"block.{i}.norm", block.norm)

    _add("policy_1", net.policy_1)
    _add("policy_2", net.policy_2)
    _add("value_1", net.value_1)
    _add("value_2", net.value_2)
    _add("floor_1", net.floor_1)
    _add("floor_2", net.floor_2)
    _add("act_1", net.act_1)
    _add("act_2", net.act_2)

    return weights


def _apply_weights(net: MLXStrategicNet, data: dict):
    """Apply loaded weights to the network."""
    def _set(prefix, layer):
        if f"{prefix}.weight" in data:
            layer.weight = data[f"{prefix}.weight"]
        if f"{prefix}.bias" in data:
            layer.bias = data[f"{prefix}.bias"]

    _set("input_linear", net.input_linear)
    _set("input_norm", net.input_norm)

    for i, block in enumerate(net.blocks):
        _set(f"block.{i}.fc1", block.fc1)
        _set(f"block.{i}.fc2", block.fc2)
        _set(f"block.{i}.norm", block.norm)

    _set("policy_1", net.policy_1)
    _set("policy_2", net.policy_2)
    _set("value_1", net.value_1)
    _set("value_2", net.value_2)
    _set("floor_1", net.floor_1)
    _set("floor_2", net.floor_2)
    _set("act_1", net.act_1)
    _set("act_2", net.act_2)


def benchmark(
    input_dim: int = 254,
    action_dim: int = 512,
    batch_sizes: list = [1, 8, 32, 128],
    n_iters: int = 100,
):
    """Benchmark MLX vs PyTorch inference throughput."""
    import time

    check_mlx()
    import torch

    print("=== MLX vs PyTorch Inference Benchmark ===")
    print(f"Model: StrategicNet (input={input_dim}, hidden=1024, action={action_dim})")
    print()

    # Create random weights (no checkpoint needed)
    from .strategic_net import StrategicNet, _get_device

    pt_model = StrategicNet(input_dim=input_dim, action_dim=action_dim)
    pt_device = _get_device()
    pt_model.to(pt_device)
    pt_model.eval()

    # Save and reload as MLX
    tmp_path = Path("/tmp/bench_strategic.pt")
    pt_model.save(tmp_path)
    mlx_model = MLXStrategicNet.from_pytorch(tmp_path)

    for bs in batch_sizes:
        obs_np = np.random.randn(bs, input_dim).astype(np.float32)
        mask_np = np.random.rand(bs, action_dim) > 0.5

        # PyTorch MPS
        obs_t = torch.from_numpy(obs_np).to(pt_device)
        mask_t = torch.from_numpy(mask_np).bool().to(pt_device)

        # Warmup
        for _ in range(5):
            with torch.no_grad():
                pt_model(obs_t, mask_t)
        if pt_device.type == "mps":
            torch.mps.synchronize()

        t0 = time.perf_counter()
        for _ in range(n_iters):
            with torch.no_grad():
                pt_model(obs_t, mask_t)
        if pt_device.type == "mps":
            torch.mps.synchronize()
        pt_ms = (time.perf_counter() - t0) / n_iters * 1000

        # MLX
        obs_mx = mx.array(obs_np)
        mask_mx = mx.array(mask_np)

        # Warmup
        for _ in range(5):
            out = mlx_model(obs_mx, mask_mx)
            mx.eval(out["policy_logits"])

        t0 = time.perf_counter()
        for _ in range(n_iters):
            out = mlx_model(obs_mx, mask_mx)
            mx.eval(out["policy_logits"])
        mlx_ms = (time.perf_counter() - t0) / n_iters * 1000

        speedup = pt_ms / mlx_ms if mlx_ms > 0 else 0
        print(f"Batch {bs:>4d}: PyTorch {pt_device.type:>3s} = {pt_ms:.2f}ms | MLX = {mlx_ms:.2f}ms | Speedup: {speedup:.1f}x")

    tmp_path.unlink(missing_ok=True)


if __name__ == "__main__":
    benchmark()
