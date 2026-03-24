"""Combat value head: predicts fight win probability from combat state.

Small network trained on MCTS rollout data. Used by TurnSolver
for leaf evaluation instead of hand-tuned _score_terminal.
"""

from __future__ import annotations

import logging
from pathlib import Path
from typing import Dict, List, Optional, Tuple

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F

from .state_encoders import CombatStateEncoder
from .strategic_net import _get_device

logger = logging.getLogger(__name__)


class CombatNet(nn.Module):
    """Predicts fight win probability from combat state.

    Architecture:
        - Input projection: input_dim -> hidden_dim
        - 2-3 hidden layers with ReLU + LayerNorm
        - Output: scalar [0, 1] (sigmoid, win probability)
    """

    def __init__(
        self,
        input_dim: int = 298,
        hidden_dim: int = 256,
        num_layers: int = 3,
    ):
        super().__init__()
        self.input_dim = input_dim
        self.hidden_dim = hidden_dim
        self.num_layers = num_layers

        layers = [
            nn.Linear(input_dim, hidden_dim),
            nn.LayerNorm(hidden_dim),
            nn.ReLU(),
        ]
        for _ in range(num_layers - 1):
            layers.extend([
                nn.Linear(hidden_dim, hidden_dim),
                nn.LayerNorm(hidden_dim),
                nn.ReLU(),
            ])
        layers.append(nn.Linear(hidden_dim, 1))

        self.net = nn.Sequential(*layers)
        self._init_weights()

    def _init_weights(self):
        """Xavier initialization with small output init."""
        for m in self.modules():
            if isinstance(m, nn.Linear):
                nn.init.xavier_uniform_(m.weight)
                if m.bias is not None:
                    nn.init.zeros_(m.bias)
        # Small init for output layer (start near 0.5 after sigmoid)
        nn.init.xavier_uniform_(self.net[-1].weight, gain=0.01)

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """Forward pass.

        Args:
            x: [batch, input_dim] combat state encoding

        Returns:
            [batch] win probability in [0, 1]
        """
        return torch.sigmoid(self.net(x).squeeze(-1))

    def predict(self, obs_np: np.ndarray) -> float:
        """Predict win probability from numpy observation (inference mode).

        Args:
            obs_np: [input_dim] combat state vector

        Returns:
            Win probability scalar in [0, 1]
        """
        device = next(self.parameters()).device
        with torch.no_grad():
            obs = torch.from_numpy(obs_np).float().unsqueeze(0).to(device)
            return self.forward(obs).item()

    def predict_batch(self, obs_batch: np.ndarray) -> np.ndarray:
        """Predict win probabilities for a batch.

        Args:
            obs_batch: [batch, input_dim] combat state vectors

        Returns:
            [batch] win probabilities
        """
        device = next(self.parameters()).device
        with torch.no_grad():
            obs = torch.from_numpy(obs_batch).float().to(device)
            return self.forward(obs).cpu().numpy()

    def param_count(self) -> int:
        """Total trainable parameters."""
        return sum(p.numel() for p in self.parameters() if p.requires_grad)

    def save(self, path: str | Path, extra: dict | None = None) -> None:
        """Save model weights + config. Atomic write."""
        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)
        data = {
            "model_state_dict": self.state_dict(),
            "config": {
                "input_dim": self.input_dim,
                "hidden_dim": self.hidden_dim,
                "num_layers": self.num_layers,
            },
        }
        if extra:
            data.update(extra)
        tmp_path = path.with_suffix(".pt.tmp")
        torch.save(data, tmp_path)
        tmp_path.rename(path)

    @classmethod
    def load(cls, path: str | Path, device: Optional[torch.device] = None) -> "CombatNet":
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


def train_combat_net(
    games_data: List[Dict],
    model: Optional[CombatNet] = None,
    epochs: int = 10,
    batch_size: int = 256,
    lr: float = 1e-3,
    device: Optional[torch.device] = None,
) -> Tuple[CombatNet, Dict[str, float]]:
    """Train combat net on combat position + outcome pairs.

    Args:
        games_data: List of dicts with:
            - "combat_obs": np.ndarray [input_dim] from CombatStateEncoder
            - "won": bool (True if player won this fight)
        model: Existing model to continue training, or None to create new
        epochs: Training epochs
        batch_size: Mini-batch size
        lr: Learning rate
        device: Torch device

    Returns:
        (trained_model, metrics_dict)
    """
    if device is None:
        device = _get_device()

    if not games_data:
        logger.warning("train_combat_net: no training data provided")
        if model is None:
            model = CombatNet()
            model.to(device)
        return model, {"loss": 0.0, "accuracy": 0.0, "samples": 0}

    # Build tensors
    obs_list = [d["combat_obs"] for d in games_data]
    labels = [float(d["won"]) for d in games_data]

    obs_t = torch.from_numpy(np.stack(obs_list)).float().to(device)
    label_t = torch.tensor(labels, dtype=torch.float32).to(device)

    if model is None:
        input_dim = obs_t.shape[1]
        model = CombatNet(input_dim=input_dim)
    model.to(device)
    model.train()

    optimizer = torch.optim.Adam(model.parameters(), lr=lr, eps=1e-5)
    n = len(obs_list)

    total_loss = 0.0
    total_correct = 0
    total_samples = 0

    for epoch in range(epochs):
        indices = torch.randperm(n)
        for start in range(0, n, batch_size):
            end = min(start + batch_size, n)
            idx = indices[start:end]

            preds = model(obs_t[idx])
            loss = F.binary_cross_entropy(preds, label_t[idx])

            optimizer.zero_grad()
            loss.backward()
            nn.utils.clip_grad_norm_(model.parameters(), 0.5)
            optimizer.step()

            with torch.no_grad():
                predicted = (preds > 0.5).float()
                total_correct += (predicted == label_t[idx]).sum().item()
                total_samples += len(idx)
                total_loss += loss.item()

    num_batches = max(total_samples // batch_size, 1)
    metrics = {
        "loss": total_loss / num_batches,
        "accuracy": total_correct / max(total_samples, 1) * 100,
        "samples": n,
        "epochs": epochs,
    }
    logger.info(
        "train_combat_net: %d samples, %d epochs, loss=%.4f, acc=%.1f%%",
        n, epochs, metrics["loss"], metrics["accuracy"],
    )
    return model, metrics


if __name__ == "__main__":
    encoder = CombatStateEncoder()
    model = CombatNet(input_dim=encoder.COMBAT_DIM)
    print(f"CombatNet parameter count: {model.param_count():,}")
    print(f"  Input dim: {encoder.COMBAT_DIM}")
    x = torch.randn(4, encoder.COMBAT_DIM)
    out = model(x)
    print(f"  Output shape: {out.shape}")
    print(f"  Output values: {out.detach().numpy()}")
