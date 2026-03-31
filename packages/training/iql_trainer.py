"""IQL (Implicit Q-Learning) trainer for offline RL.

Trains on collected trajectory data using:
- Q-network: state,action -> Q-value (twin Q for stability)
- V-network: state -> V-value
- Advantage-weighted policy extraction

Reference: Kostrikov et al., "Offline Reinforcement Learning with
Implicit Q-Learning" (2021)
"""

from __future__ import annotations

import logging
from typing import Dict, Optional

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F

from .offline_data import OfflineBatch, OfflineDataset
from .strategic_net import StrategicNet
from .training_config import (
    IQL_DISCOUNT,
    IQL_EXPECTILE,
    IQL_LR,
    IQL_Q_HIDDEN,
    IQL_TEMPERATURE,
    IQL_VALUE_HIDDEN,
    MODEL_ACTION_DIM,
)

logger = logging.getLogger(__name__)


class ValueNetwork(nn.Module):
    """V(s): state -> scalar value."""

    def __init__(self, input_dim: int, hidden_dim: int = 512):
        super().__init__()
        self.net = nn.Sequential(
            nn.Linear(input_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, 1),
        )

    def forward(self, state: torch.Tensor) -> torch.Tensor:
        """Returns [batch] scalar values."""
        return self.net(state).squeeze(-1)


class QNetwork(nn.Module):
    """Twin Q(s, a): state + action_onehot -> Q-value.

    Uses twin Q-networks for stability (min of two Q-values).
    """

    def __init__(self, input_dim: int, action_dim: int, hidden_dim: int = 512):
        super().__init__()
        combined_dim = input_dim + action_dim
        self.q1 = nn.Sequential(
            nn.Linear(combined_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, 1),
        )
        self.q2 = nn.Sequential(
            nn.Linear(combined_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, hidden_dim),
            nn.ReLU(),
            nn.Linear(hidden_dim, 1),
        )
        self.action_dim = action_dim

    def forward(
        self, state: torch.Tensor, action: torch.Tensor
    ) -> tuple[torch.Tensor, torch.Tensor]:
        """Returns (q1, q2) each [batch] scalar values.

        Args:
            state: [batch, input_dim]
            action: [batch] int64 action indices
        """
        action_onehot = F.one_hot(action, self.action_dim).float()
        sa = torch.cat([state, action_onehot], dim=-1)
        return self.q1(sa).squeeze(-1), self.q2(sa).squeeze(-1)

    def min_q(self, state: torch.Tensor, action: torch.Tensor) -> torch.Tensor:
        """Returns min(q1, q2) for conservative estimate."""
        q1, q2 = self.forward(state, action)
        return torch.min(q1, q2)


def _expectile_loss(diff: torch.Tensor, expectile: float) -> torch.Tensor:
    """Asymmetric L2 loss for expectile regression.

    Weight = expectile when diff > 0, (1 - expectile) when diff <= 0.
    This pushes V toward the upper expectile of Q when expectile > 0.5.
    """
    weight = torch.where(diff > 0, expectile, 1.0 - expectile)
    return (weight * diff.pow(2)).mean()


class IQLTrainer:
    """Implicit Q-Learning trainer for offline RL.

    Training loop:
        1. Sample batch from OfflineDataset
        2. Train V with expectile regression: L_V = expectile_loss(Q_target - V(s), tau)
        3. Train Q with Bellman: L_Q = MSE(Q(s,a), r + gamma * V(s'))
        4. Extract policy: advantage = Q(s,a) - V(s), weight = exp(adv / temp)
        5. Train policy: advantage-weighted behavioral cloning
    """

    def __init__(
        self,
        policy: StrategicNet,
        input_dim: int = 480,
        action_dim: int = MODEL_ACTION_DIM,
        lr: float = IQL_LR,
        discount: float = IQL_DISCOUNT,
        expectile: float = IQL_EXPECTILE,
        temperature: float = IQL_TEMPERATURE,
        value_hidden: int = IQL_VALUE_HIDDEN,
        q_hidden: int = IQL_Q_HIDDEN,
        max_grad_norm: float = 0.5,
    ):
        self.policy = policy
        self.discount = discount
        self.expectile = expectile
        self.temperature = temperature
        self.max_grad_norm = max_grad_norm
        self.action_dim = action_dim

        self.device = next(policy.parameters()).device

        self.v_net = ValueNetwork(input_dim, value_hidden).to(self.device)
        self.q_net = QNetwork(input_dim, action_dim, q_hidden).to(self.device)

        # Target Q-network (EMA updated)
        self.q_target = QNetwork(input_dim, action_dim, q_hidden).to(self.device)
        self.q_target.load_state_dict(self.q_net.state_dict())

        self.v_optimizer = torch.optim.Adam(self.v_net.parameters(), lr=lr, eps=1e-5)
        self.q_optimizer = torch.optim.Adam(self.q_net.parameters(), lr=lr, eps=1e-5)
        self.policy_optimizer = torch.optim.Adam(policy.parameters(), lr=lr, eps=1e-5)

        self.train_steps = 0

    def _update_target(self, tau: float = 0.005):
        """Soft update target Q-network: target = tau * q + (1-tau) * target."""
        for tp, p in zip(self.q_target.parameters(), self.q_net.parameters()):
            tp.data.copy_(tau * p.data + (1.0 - tau) * tp.data)

    def train_step(self, batch: OfflineBatch) -> Dict[str, float]:
        """Single IQL training step on a batch.

        Args:
            batch: OfflineBatch with states, actions, rewards, next_states, dones, action_masks

        Returns:
            Dict of loss metrics
        """
        device = self.device

        s = batch.states.to(device)
        a = batch.actions.to(device)
        r = batch.rewards.to(device)
        s_next = batch.next_states.to(device)
        done = batch.dones.to(device)
        masks = batch.action_masks.to(device)

        # --- 1. Train V with expectile regression ---
        with torch.no_grad():
            q_target = self.q_target.min_q(s, a)

        v = self.v_net(s)
        v_loss = _expectile_loss(q_target - v, self.expectile)

        self.v_optimizer.zero_grad()
        v_loss.backward()
        nn.utils.clip_grad_norm_(self.v_net.parameters(), self.max_grad_norm)
        self.v_optimizer.step()

        # --- 2. Train Q with Bellman target ---
        with torch.no_grad():
            v_next = self.v_net(s_next)
            q_backup = r + self.discount * (1.0 - done) * v_next

        q1, q2 = self.q_net(s, a)
        q_loss = F.mse_loss(q1, q_backup) + F.mse_loss(q2, q_backup)

        self.q_optimizer.zero_grad()
        q_loss.backward()
        nn.utils.clip_grad_norm_(self.q_net.parameters(), self.max_grad_norm)
        self.q_optimizer.step()

        # --- 3. Policy extraction: advantage-weighted BC ---
        with torch.no_grad():
            q_val = self.q_target.min_q(s, a)
            v_val = self.v_net(s)
            advantage = q_val - v_val
            weight = torch.exp(advantage / self.temperature)
            # Clamp weights to prevent explosion
            weight = torch.clamp(weight, max=100.0)

        self.policy.train()
        out = self.policy(s, masks)
        log_probs = F.log_softmax(out["policy_logits"], dim=-1)
        action_log_prob = log_probs.gather(1, a.unsqueeze(1)).squeeze(1)
        policy_loss = -(weight * action_log_prob).mean()

        self.policy_optimizer.zero_grad()
        policy_loss.backward()
        nn.utils.clip_grad_norm_(self.policy.parameters(), self.max_grad_norm)
        self.policy_optimizer.step()

        # --- 4. Soft update target Q ---
        self._update_target()
        self.train_steps += 1

        return {
            "v_loss": v_loss.item(),
            "q_loss": q_loss.item(),
            "policy_loss": policy_loss.item(),
            "advantage_mean": advantage.mean().item(),
            "weight_mean": weight.mean().item(),
            "v_mean": v.mean().item(),
            "q_mean": q_val.mean().item(),
            "train_steps": self.train_steps,
        }

    def train_offline(
        self,
        dataset: OfflineDataset,
        epochs: int = 10,
        batch_size: int = 256,
        steps_per_epoch: Optional[int] = None,
    ) -> Dict[str, float]:
        """Train IQL on offline dataset for multiple epochs.

        Args:
            dataset: OfflineDataset with transitions
            epochs: Number of passes through the data
            batch_size: Mini-batch size
            steps_per_epoch: If set, limits steps per epoch (otherwise full pass)

        Returns:
            Averaged metrics across all steps
        """
        if len(dataset) == 0:
            logger.warning("train_offline: empty dataset")
            return {"v_loss": 0.0, "q_loss": 0.0, "policy_loss": 0.0, "steps": 0}

        n = len(dataset)
        if steps_per_epoch is None:
            steps_per_epoch = max(n // batch_size, 1)

        total_metrics: Dict[str, float] = {}
        total_steps = 0

        for epoch in range(epochs):
            for step in range(steps_per_epoch):
                batch = dataset.sample_batch(batch_size)
                metrics = self.train_step(batch)

                for k, v in metrics.items():
                    total_metrics[k] = total_metrics.get(k, 0.0) + v
                total_steps += 1

            logger.info(
                "IQL epoch %d/%d: v_loss=%.4f, q_loss=%.4f, policy_loss=%.4f",
                epoch + 1, epochs,
                total_metrics.get("v_loss", 0) / max(total_steps, 1),
                total_metrics.get("q_loss", 0) / max(total_steps, 1),
                total_metrics.get("policy_loss", 0) / max(total_steps, 1),
            )

        # Average all metrics
        if total_steps > 0:
            for k in total_metrics:
                total_metrics[k] /= total_steps
        total_metrics["steps"] = total_steps
        total_metrics["epochs"] = epochs
        total_metrics["dataset_size"] = n

        return total_metrics
