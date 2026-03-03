"""
PyTorch policy + value network for Slay the Spire.

Replaces numpy PolicyValueNet with real autograd gradients.
Same interface for drop-in compatibility. Supports MPS (Apple Silicon).

Architecture:
- Shared trunk: MLP with LayerNorm (obs_dim → hidden → hidden)
- Policy head: hidden → action_dim logits
- Value head: hidden → 1 (tanh)
- Auxiliary heads: hidden → N (HP prediction, turns-to-kill, act-reach)
"""

from __future__ import annotations

import math
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

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


class StSPolicyValueNet(nn.Module):
    """Dual-headed policy + value network for STS.

    Outputs:
        policy_logits: [batch, action_dim] — raw logits (mask before softmax)
        value: [batch, 1] — P(win) estimate in [-1, 1]
        aux: [batch, num_aux] — auxiliary predictions (HP, turns, act-reach)
    """

    def __init__(
        self,
        obs_dim: int = 1186,
        hidden_dim: int = 256,
        action_dim: int = 2048,
        num_layers: int = 3,
        num_aux: int = 3,
        dropout: float = 0.0,
    ):
        super().__init__()
        self.obs_dim = obs_dim
        self.hidden_dim = hidden_dim
        self.action_dim = action_dim
        self.num_aux = num_aux

        # Shared trunk
        layers: List[nn.Module] = []
        in_dim = obs_dim
        for i in range(num_layers):
            layers.append(nn.Linear(in_dim, hidden_dim))
            layers.append(nn.LayerNorm(hidden_dim))
            layers.append(nn.ReLU())
            if dropout > 0:
                layers.append(nn.Dropout(dropout))
            in_dim = hidden_dim
        self.trunk = nn.Sequential(*layers)

        # Policy head
        self.policy_head = nn.Linear(hidden_dim, action_dim)

        # Value head
        self.value_head = nn.Sequential(
            nn.Linear(hidden_dim, 64),
            nn.ReLU(),
            nn.Linear(64, 1),
            nn.Tanh(),
        )

        # Auxiliary head (HP remaining, turns to kill, act 3 reached)
        if num_aux > 0:
            self.aux_head = nn.Sequential(
                nn.Linear(hidden_dim, 64),
                nn.ReLU(),
                nn.Linear(64, num_aux),
            )
        else:
            self.aux_head = None

        self._init_weights()

    def _init_weights(self):
        """Xavier initialization for better gradient flow."""
        for m in self.modules():
            if isinstance(m, nn.Linear):
                nn.init.xavier_uniform_(m.weight)
                if m.bias is not None:
                    nn.init.zeros_(m.bias)
        # Small init for policy head (near-uniform initial policy)
        nn.init.xavier_uniform_(self.policy_head.weight, gain=0.01)

    def forward(
        self,
        obs: torch.Tensor,
        action_mask: Optional[torch.Tensor] = None,
    ) -> Tuple[torch.Tensor, torch.Tensor, Optional[torch.Tensor]]:
        """Forward pass.

        Args:
            obs: [batch, obs_dim] observation tensor
            action_mask: [batch, action_dim] bool tensor (True=valid)

        Returns:
            (policy_logits, value, aux_preds)
            policy_logits are masked (invalid actions = -inf)
        """
        h = self.trunk(obs)

        # Policy
        logits = self.policy_head(h)
        if action_mask is not None:
            # Use large negative instead of -inf to avoid NaN gradients in log_softmax
            logits = logits.masked_fill(~action_mask, -1e8)

        # Value
        value = self.value_head(h)

        # Auxiliary
        aux = self.aux_head(h) if self.aux_head is not None else None

        return logits, value.squeeze(-1), aux

    def predict_action(
        self,
        obs_np: np.ndarray,
        action_mask_np: np.ndarray,
        temperature: float = 0.0,
    ) -> Tuple[int, float, np.ndarray]:
        """Predict action from numpy observation (inference mode).

        Args:
            obs_np: [obs_dim] float32
            action_mask_np: [action_dim] bool

        Returns:
            (action_index, value, policy_probs)
        """
        device = next(self.parameters()).device

        with torch.no_grad():
            obs = torch.from_numpy(obs_np).float().unsqueeze(0).to(device)
            mask = torch.from_numpy(action_mask_np).bool().unsqueeze(0).to(device)

            logits, value, _ = self.forward(obs, mask)

            # Apply temperature
            if temperature > 0:
                probs = F.softmax(logits / temperature, dim=-1)
                action = torch.multinomial(probs, 1).item()
            else:
                action = logits.argmax(dim=-1).item()

            probs = F.softmax(logits, dim=-1).squeeze(0).cpu().numpy()
            val = value.item()

        return action, val, probs

    def get_policy_and_value(
        self,
        obs_batch: torch.Tensor,
        mask_batch: Optional[torch.Tensor] = None,
    ) -> Tuple[torch.Tensor, torch.Tensor]:
        """Get log-probs and values for a batch (for training).

        Returns:
            (log_probs [batch, action_dim], values [batch])
        """
        logits, values, _ = self.forward(obs_batch, mask_batch)
        log_probs = F.log_softmax(logits, dim=-1)
        return log_probs, values

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
                    "obs_dim": self.obs_dim,
                    "hidden_dim": self.hidden_dim,
                    "action_dim": self.action_dim,
                    "num_aux": self.num_aux,
                },
            },
            path,
        )

    @classmethod
    def load(cls, path: str | Path, device: Optional[torch.device] = None) -> "StSPolicyValueNet":
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


class PPOTrainer:
    """Proximal Policy Optimization trainer for the meta model.

    Implements PPO-clip with:
    - GAE advantage estimation (gamma=1.0 for episodic, lambda=0.95)
    - Multiple gradient steps per batch (K=4 epochs)
    - Entropy bonus with annealing
    - Gradient clipping
    - Auxiliary loss heads
    """

    def __init__(
        self,
        model: StSPolicyValueNet,
        lr: float = 3e-4,
        gamma: float = 1.0,
        gae_lambda: float = 0.95,
        clip_epsilon: float = 0.2,
        entropy_coeff: float = 0.01,
        value_coeff: float = 0.5,
        aux_coeff: float = 0.25,
        max_grad_norm: float = 0.5,
        ppo_epochs: int = 4,
        batch_size: int = 256,
    ):
        self.model = model
        self.gamma = gamma
        self.gae_lambda = gae_lambda
        self.clip_epsilon = clip_epsilon
        self.entropy_coeff = entropy_coeff
        self.value_coeff = value_coeff
        self.aux_coeff = aux_coeff
        self.max_grad_norm = max_grad_norm
        self.ppo_epochs = ppo_epochs
        self.batch_size = batch_size

        self.optimizer = torch.optim.Adam(model.parameters(), lr=lr, eps=1e-5)
        self.scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(
            self.optimizer, T_max=10000, eta_min=1e-5,
        )

        self.train_steps = 0

    def compute_gae(
        self,
        rewards: torch.Tensor,
        values: torch.Tensor,
        dones: torch.Tensor,
    ) -> Tuple[torch.Tensor, torch.Tensor]:
        """Compute GAE advantages and returns.

        Args:
            rewards: [T] per-step rewards
            values: [T+1] value estimates (includes bootstrap value)
            dones: [T] episode termination flags

        Returns:
            (advantages [T], returns [T])
        """
        T = len(rewards)
        advantages = torch.zeros(T, device=rewards.device)
        gae = 0.0

        for t in reversed(range(T)):
            next_val = values[t + 1] * (1 - dones[t])
            delta = rewards[t] + self.gamma * next_val - values[t]
            gae = delta + self.gamma * self.gae_lambda * (1 - dones[t]) * gae
            advantages[t] = gae

        returns = advantages + values[:T]
        return advantages, returns

    def train_on_batch(
        self,
        obs: torch.Tensor,
        actions: torch.Tensor,
        old_log_probs: torch.Tensor,
        advantages: torch.Tensor,
        returns: torch.Tensor,
        masks: Optional[torch.Tensor] = None,
        aux_targets: Optional[torch.Tensor] = None,
    ) -> Dict[str, float]:
        """Run PPO update on a collected batch.

        Args:
            obs: [N, obs_dim]
            actions: [N] action indices
            old_log_probs: [N] log probs under old policy
            advantages: [N] GAE advantages
            returns: [N] target returns
            masks: [N, action_dim] action masks
            aux_targets: [N, num_aux] auxiliary targets

        Returns:
            Dict of loss metrics.
        """
        self.model.train()
        device = next(self.model.parameters()).device

        # Move to device
        obs = obs.to(device)
        actions = actions.to(device)
        old_log_probs = old_log_probs.to(device)
        advantages = advantages.to(device)
        returns = returns.to(device)
        if masks is not None:
            masks = masks.to(device)
        if aux_targets is not None:
            aux_targets = aux_targets.to(device)

        # Normalize advantages
        adv_std = advantages.std()
        if adv_std > 1e-8:
            advantages = (advantages - advantages.mean()) / (adv_std + 1e-8)

        N = obs.shape[0]
        total_metrics: Dict[str, float] = {
            "policy_loss": 0.0,
            "value_loss": 0.0,
            "entropy": 0.0,
            "aux_loss": 0.0,
            "total_loss": 0.0,
            "clip_fraction": 0.0,
        }

        for epoch in range(self.ppo_epochs):
            # Shuffle indices
            indices = torch.randperm(N, device=device)

            for start in range(0, N, self.batch_size):
                end = min(start + self.batch_size, N)
                idx = indices[start:end]

                b_obs = obs[idx]
                b_actions = actions[idx]
                b_old_lp = old_log_probs[idx]
                b_adv = advantages[idx]
                b_ret = returns[idx]
                b_masks = masks[idx] if masks is not None else None
                b_aux = aux_targets[idx] if aux_targets is not None else None

                # Forward
                logits, values, aux_preds = self.model(b_obs, b_masks)
                log_probs = F.log_softmax(logits, dim=-1)
                action_log_probs = log_probs.gather(1, b_actions.unsqueeze(1)).squeeze(1)

                # PPO clipped objective
                ratio = torch.exp(action_log_probs - b_old_lp)
                surr1 = ratio * b_adv
                surr2 = torch.clamp(ratio, 1 - self.clip_epsilon, 1 + self.clip_epsilon) * b_adv
                policy_loss = -torch.min(surr1, surr2).mean()

                # Value loss
                value_loss = F.mse_loss(values, b_ret)

                # Entropy bonus (handle masked -inf: replace 0*-inf NaN with 0)
                probs = F.softmax(logits, dim=-1)
                ent_term = probs * log_probs
                ent_term = ent_term.nan_to_num(0.0)  # 0 * -inf = NaN → 0
                entropy = -ent_term.sum(dim=-1).mean()

                # Auxiliary loss
                aux_loss = torch.tensor(0.0, device=device)
                if b_aux is not None and aux_preds is not None:
                    aux_loss = F.mse_loss(aux_preds, b_aux)

                # Total loss
                loss = (
                    policy_loss
                    + self.value_coeff * value_loss
                    - self.entropy_coeff * entropy
                    + self.aux_coeff * aux_loss
                )

                # Backprop
                self.optimizer.zero_grad()
                loss.backward()
                nn.utils.clip_grad_norm_(self.model.parameters(), self.max_grad_norm)
                self.optimizer.step()

                # Track metrics
                with torch.no_grad():
                    clip_frac = ((ratio - 1).abs() > self.clip_epsilon).float().mean().item()

                batch_count = self.ppo_epochs * math.ceil(N / self.batch_size)
                total_metrics["policy_loss"] += policy_loss.item() / batch_count
                total_metrics["value_loss"] += value_loss.item() / batch_count
                total_metrics["entropy"] += entropy.item() / batch_count
                total_metrics["aux_loss"] += aux_loss.item() / batch_count
                total_metrics["total_loss"] += loss.item() / batch_count
                total_metrics["clip_fraction"] += clip_frac / batch_count

        self.scheduler.step()
        self.train_steps += 1

        total_metrics["lr"] = self.optimizer.param_groups[0]["lr"]
        total_metrics["train_steps"] = self.train_steps

        return total_metrics

    def decay_entropy(self, min_coeff: float = 0.001, decay: float = 0.999):
        """Anneal entropy coefficient."""
        self.entropy_coeff = max(min_coeff, self.entropy_coeff * decay)
