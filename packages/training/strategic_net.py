"""
Strategic decision model for the two-model RL architecture.

Residual trunk with LayerNorm (default: 8x1024, ~18M params).
Multi-head output for different decision types + auxiliary predictions.

The strategic model handles all non-combat decisions: path selection,
card picks, rest sites, shop, events. Combat is handled by the
separate combat solver (MCTS + combat net).
"""

from __future__ import annotations

from pathlib import Path
from typing import Dict, Optional, Tuple

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F


def _get_device() -> torch.device:
    """Get best available device (MPS > CUDA > CPU)."""
    if torch.backends.mps.is_available():
        return torch.device("mps")
    if torch.cuda.is_available():
        return torch.device("cuda")
    return torch.device("cpu")


class PopArtLayer(nn.Module):
    """PopArt normalization for value targets.

    Maintains running mean/std of value targets via exponential moving average.
    Normalizes targets for training, denormalizes outputs for inference.

    Reference: https://arxiv.org/abs/1602.07714
    """

    def __init__(self, beta: float = 0.0003):
        super().__init__()
        self.beta = beta
        self.register_buffer("mu", torch.zeros(1))
        self.register_buffer("sigma", torch.ones(1))
        self.register_buffer("count", torch.zeros(1))

    def update(self, targets: torch.Tensor) -> None:
        """Update running statistics with a batch of targets."""
        with torch.no_grad():
            batch_mean = targets.mean()
            batch_var = targets.var()
            batch_std = (batch_var + 1e-8).sqrt()

            self.count += 1
            beta = max(self.beta, 1.0 / self.count.item())
            self.mu = (1 - beta) * self.mu + beta * batch_mean
            self.sigma = (1 - beta) * self.sigma + beta * batch_std

    def normalize(self, targets: torch.Tensor) -> torch.Tensor:
        """Normalize targets: y_norm = (y - mu) / sigma."""
        return (targets - self.mu) / (self.sigma + 1e-8)

    def denormalize(self, values: torch.Tensor) -> torch.Tensor:
        """Denormalize model output: v_denorm = v * sigma + mu."""
        return values * self.sigma + self.mu


class ResidualBlock(nn.Module):
    """Residual block: two-layer MLP + LayerNorm with skip connection.

    fc1 -> ReLU -> fc2 -> LayerNorm -> + skip
    """

    def __init__(self, dim: int):
        super().__init__()
        self.fc1 = nn.Linear(dim, dim)
        self.fc2 = nn.Linear(dim, dim)
        self.norm = nn.LayerNorm(dim)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        h = F.relu(self.fc1(x))
        return self.norm(self.fc2(h)) + x


class StrategicNet(nn.Module):
    """Strategic decision model for non-combat phases.

    Architecture:
        - Input projection: input_dim -> hidden_dim
        - N residual blocks of Linear(hidden_dim) + LayerNorm + ReLU
        - Multi-head output:
            - decision: policy logits over action space
            - value: scalar value estimate
            - floor_pred: predicted final floor
            - act_completion: P(clear act 1/2/3)

    Outputs dict:
        "policy_logits": [batch, action_dim]
        "value": [batch]
        "floor_pred": [batch]
        "act_completion": [batch, 3]
    """

    def __init__(
        self,
        input_dim: int = 480,
        hidden_dim: int = 1024,
        action_dim: int = 512,
        num_blocks: int = 8,
    ):
        super().__init__()
        self.input_dim = input_dim
        self.hidden_dim = hidden_dim
        self.action_dim = action_dim
        self.num_blocks = num_blocks

        # Input projection
        self.input_proj = nn.Sequential(
            nn.Linear(input_dim, hidden_dim),
            nn.LayerNorm(hidden_dim),
            nn.ReLU(),
        )

        # Residual trunk
        self.trunk = nn.Sequential(*[ResidualBlock(hidden_dim) for _ in range(num_blocks)])

        # --- Output heads ---

        # Policy: hidden_dim -> 256 -> action_dim
        self.policy_head = nn.Sequential(
            nn.Linear(hidden_dim, 256),
            nn.ReLU(),
            nn.Linear(256, action_dim),
        )

        # Value: hidden_dim -> 64 -> 1 (no Tanh — rewards can exceed [-1, 1])
        self.value_head = nn.Sequential(
            nn.Linear(hidden_dim, 64),
            nn.ReLU(),
            nn.Linear(64, 1),
        )

        # Floor prediction: hidden_dim -> 64 -> 1
        self.floor_head = nn.Sequential(
            nn.Linear(hidden_dim, 64),
            nn.ReLU(),
            nn.Linear(64, 1),
        )

        # Act completion P(clear act 1/2/3): hidden_dim -> 64 -> 3
        self.act_head = nn.Sequential(
            nn.Linear(hidden_dim, 64),
            nn.ReLU(),
            nn.Linear(64, 3),
            nn.Sigmoid(),
        )

        self._init_weights()

    def _init_weights(self):
        """Xavier initialization with small policy head init."""
        for m in self.modules():
            if isinstance(m, nn.Linear):
                nn.init.xavier_uniform_(m.weight)
                if m.bias is not None:
                    nn.init.zeros_(m.bias)
        # Small init for policy head (near-uniform initial policy)
        nn.init.xavier_uniform_(self.policy_head[-1].weight, gain=0.01)

    def forward(
        self,
        x: torch.Tensor,
        action_mask: Optional[torch.Tensor] = None,
    ) -> Dict[str, torch.Tensor]:
        """Forward pass.

        Args:
            x: [batch, input_dim] observation tensor
            action_mask: [batch, action_dim] bool tensor (True=valid)

        Returns:
            Dict with policy_logits, value, floor_pred, act_completion
        """
        h = self.input_proj(x)
        h = self.trunk(h)

        # Policy
        logits = self.policy_head(h)
        if action_mask is not None:
            logits = logits.masked_fill(~action_mask, -1e8)

        return {
            "policy_logits": logits,
            "value": self.value_head(h).squeeze(-1),
            "floor_pred": self.floor_head(h).squeeze(-1),
            "act_completion": self.act_head(h),
        }

    def predict_action(
        self,
        obs_np: np.ndarray,
        action_mask_np: np.ndarray,
        temperature: float = 0.0,
    ) -> Tuple[int, float, np.ndarray]:
        """Predict action from numpy observation (inference mode).

        Returns:
            (action_index, value, policy_probs)
        """
        device = next(self.parameters()).device

        with torch.no_grad():
            obs = torch.from_numpy(obs_np).float().unsqueeze(0).to(device)
            mask = torch.from_numpy(action_mask_np).bool().unsqueeze(0).to(device)

            out = self.forward(obs, mask)
            logits = out["policy_logits"]

            if temperature > 0:
                probs = F.softmax(logits / temperature, dim=-1)
                action = torch.multinomial(probs, 1).item()
            else:
                action = logits.argmax(dim=-1).item()

            probs = F.softmax(logits, dim=-1).squeeze(0).cpu().numpy()
            val = out["value"].item()

        return action, val, probs

    def param_count(self) -> int:
        """Total trainable parameters."""
        return sum(p.numel() for p in self.parameters() if p.requires_grad)

    def save(self, path: str | Path, extra: dict | None = None) -> None:
        """Save model weights + config + optional training state. Atomic write."""
        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)
        data = {
            "model_state_dict": self.state_dict(),
            "config": {
                "input_dim": self.input_dim,
                "hidden_dim": self.hidden_dim,
                "action_dim": self.action_dim,
                "num_blocks": self.num_blocks,
            },
        }
        if extra:
            data.update(extra)
        # Atomic write: save to tmp, then rename (rename is atomic on most FS)
        tmp_path = path.with_suffix(".pt.tmp")
        torch.save(data, tmp_path)
        tmp_path.rename(path)

    @classmethod
    def load(cls, path: str | Path, device: Optional[torch.device] = None) -> "StrategicNet":
        """Load model from checkpoint."""
        if device is None:
            device = _get_device()
        checkpoint = torch.load(path, map_location=device, weights_only=True)
        config = checkpoint["config"]
        model = cls(**config)
        model.load_state_dict(checkpoint["model_state_dict"])
        model.to(device)
        model.eval()
        return model


if __name__ == "__main__":
    model = StrategicNet(input_dim=254)
    print(f"StrategicNet parameter count: {model.param_count():,}")
    print(f"  Target: ~3M")
    x = torch.randn(1, 254)
    out = model(x)
    for k, v in out.items():
        print(f"  {k}: {v.shape}")
