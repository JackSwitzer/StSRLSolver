"""
Strategic decision model for the two-model RL architecture.

4x384 trunk with residual connections + LayerNorm.
Multi-head output for different decision types + auxiliary predictions.
Target ~3M parameters.

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


class ResidualBlock(nn.Module):
    """Residual block: Linear + LayerNorm + ReLU with skip connection."""

    def __init__(self, dim: int):
        super().__init__()
        self.linear = nn.Linear(dim, dim)
        self.norm = nn.LayerNorm(dim)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return F.relu(self.norm(self.linear(x))) + x


class StrategicNet(nn.Module):
    """Strategic decision model for non-combat phases.

    Architecture:
        - Input projection: input_dim -> hidden_dim
        - 4 residual blocks of Linear(hidden_dim) + LayerNorm + ReLU
        - Multi-head output:
            - decision: policy logits over action space
            - value: P(win) estimate [-1, 1]
            - floor_pred: predicted final floor
            - combat_cost: predicted HP loss next combat
            - act_completion: P(clear act 1/2/3)

    Outputs dict:
        "policy_logits": [batch, action_dim]
        "value": [batch]
        "floor_pred": [batch]
        "combat_cost": [batch]
        "act_completion": [batch, 3]
    """

    def __init__(
        self,
        input_dim: int = 260,
        hidden_dim: int = 768,
        action_dim: int = 256,
        num_blocks: int = 4,
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

        # Policy: 384 -> 256 -> action_dim
        self.policy_head = nn.Sequential(
            nn.Linear(hidden_dim, 256),
            nn.ReLU(),
            nn.Linear(256, action_dim),
        )

        # Value: 384 -> 64 -> 1 -> Tanh
        self.value_head = nn.Sequential(
            nn.Linear(hidden_dim, 64),
            nn.ReLU(),
            nn.Linear(64, 1),
            nn.Tanh(),
        )

        # Floor prediction: 384 -> 64 -> 1
        self.floor_head = nn.Sequential(
            nn.Linear(hidden_dim, 64),
            nn.ReLU(),
            nn.Linear(64, 1),
        )

        # Combat cost (predicted HP loss): 384 -> 64 -> 1
        self.combat_cost_head = nn.Sequential(
            nn.Linear(hidden_dim, 64),
            nn.ReLU(),
            nn.Linear(64, 1),
        )

        # Act completion P(clear act 1/2/3): 384 -> 64 -> 3
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
            Dict with policy_logits, value, floor_pred, combat_cost, act_completion
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
            "combat_cost": self.combat_cost_head(h).squeeze(-1),
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

    def save(self, path: str | Path) -> None:
        """Save model weights + config."""
        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)
        torch.save(
            {
                "model_state_dict": self.state_dict(),
                "config": {
                    "input_dim": self.input_dim,
                    "hidden_dim": self.hidden_dim,
                    "action_dim": self.action_dim,
                    "num_blocks": self.num_blocks,
                },
            },
            path,
        )

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
