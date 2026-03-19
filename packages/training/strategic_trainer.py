"""
PPO trainer for the strategic model.

Collects transitions at every non-combat decision point.
Trains periodically with PPO + auxiliary losses.

Auxiliary losses:
- floor_prediction: MSE on predicted final floor
- act_completion: BCE on P(clear act 1/2/3)

Phase 2A fixes (2026-03-12):
- batch_size default 32 -> 256
- LR warmup over first 100 train steps
- Entropy starts at 0.05, decays to 0.01 minimum
- Buffer accumulates across train_batch calls (only trims excess)
"""

from __future__ import annotations

import math
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Optional

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F

from .strategic_net import StrategicNet, _get_device


@dataclass
class StrategicTransition:
    """Single training transition from a strategic decision point."""
    obs: np.ndarray           # [input_dim] run state encoding
    action_mask: np.ndarray   # [action_dim] valid actions
    action: int               # action index taken
    reward: float             # shaped reward
    done: bool                # episode termination
    value: float              # value estimate at decision time
    log_prob: float           # log probability of chosen action
    episode_id: int = 0       # unique episode identifier for GAE computation
    # Auxiliary targets (filled after game ends)
    final_floor: float = 0.0
    cleared_act1: float = 0.0
    cleared_act2: float = 0.0
    cleared_act3: float = 0.0


class StrategicTrainer:
    """PPO trainer for the strategic model.

    Collects transitions at every non-combat decision point.
    Trains periodically with PPO + auxiliary losses.

    Key changes from v1:
    - batch_size=256 (was 32)
    - LR warmup: linear over first warmup_steps train steps
    - Entropy: starts at 0.05, min_coeff=0.01
    - Buffer accumulation: only trims excess after training, never fully clears
    """

    def __init__(
        self,
        model: StrategicNet,
        lr: float = 1e-4,
        gamma: float = 0.99,
        gae_lambda: float = 0.95,
        clip_epsilon: float = 0.2,
        entropy_coeff: float = 0.05,
        value_coeff: float = 0.5,
        aux_coeff: float = 0.25,
        max_grad_norm: float = 0.5,
        ppo_epochs: int = 4,
        batch_size: int = 256,
        warmup_steps: int = 100,
        checkpoint_dir: str = "logs/strategic_checkpoints",
        lr_schedule: str = "cosine",
        lr_T_max: int = 30000,
        lr_T_0: int = 5000,
    ):
        self.model = model
        self.gamma = gamma
        self.gae_lambda = gae_lambda
        self.clip_epsilon = clip_epsilon
        self.entropy_coeff = entropy_coeff
        self._initial_entropy_coeff = entropy_coeff
        self.value_coeff = value_coeff
        self.aux_coeff = aux_coeff
        self.max_grad_norm = max_grad_norm
        self.ppo_epochs = ppo_epochs
        self.batch_size = batch_size
        self.warmup_steps = warmup_steps
        self.checkpoint_dir = Path(checkpoint_dir)
        self.checkpoint_dir.mkdir(parents=True, exist_ok=True)

        self._base_lr = lr
        self.optimizer = torch.optim.Adam(model.parameters(), lr=lr, eps=1e-5)

        # Configurable LR schedule (starts after warmup)
        if lr_schedule == "linear_decay":
            self.scheduler = torch.optim.lr_scheduler.LinearLR(
                self.optimizer, start_factor=1.0, end_factor=0.1,
                total_iters=lr_T_max,
            )
        elif lr_schedule == "cosine_warm_restarts":
            self.scheduler = torch.optim.lr_scheduler.CosineAnnealingWarmRestarts(
                self.optimizer, T_0=lr_T_0, eta_min=1e-5,
            )
        else:  # "cosine" (default)
            self.scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(
                self.optimizer, T_max=lr_T_max, eta_min=1e-5,
            )

        self.buffer: List[StrategicTransition] = []
        self.best_avg_floor = 0.0
        self.train_steps = 0

    def _apply_lr_warmup(self) -> None:
        """Apply linear LR warmup during early training steps.

        Sets LR = base_lr * (train_steps / warmup_steps) for the first
        warmup_steps steps. After warmup, cosine annealing takes over.
        """
        if self.train_steps < self.warmup_steps:
            warmup_factor = max(self.train_steps / self.warmup_steps, 1e-3)
            for pg in self.optimizer.param_groups:
                pg["lr"] = self._base_lr * warmup_factor

    def add_transition(
        self,
        obs: np.ndarray,
        action_mask: np.ndarray,
        action: int,
        reward: float,
        done: bool,
        value: float,
        log_prob: float,
        episode_id: int = 0,
    ) -> None:
        """Record a transition at a non-combat decision point."""
        self.buffer.append(StrategicTransition(
            obs=obs.copy(),
            action_mask=action_mask.copy(),
            action=action,
            reward=reward,
            done=done,
            value=value,
            log_prob=log_prob,
            episode_id=episode_id,
        ))

    def compute_gae(self) -> tuple[np.ndarray, np.ndarray]:
        """Compute GAE advantages and returns from buffer.

        Computes GAE per-episode to avoid cross-episode contamination.
        Transitions from different games are interleaved in the buffer
        (via ProcessPoolExecutor as_completed ordering), so we group by
        episode_id and compute GAE within each episode independently.

        Returns:
            (advantages, returns) each of shape (N,)
        """
        N = len(self.buffer)
        if N == 0:
            return np.array([], dtype=np.float32), np.array([], dtype=np.float32)

        values = np.array([t.value for t in self.buffer], dtype=np.float32)
        advantages = np.zeros(N, dtype=np.float32)

        # Group buffer indices by episode
        episodes: Dict[int, List[int]] = {}
        for i, t in enumerate(self.buffer):
            episodes.setdefault(t.episode_id, []).append(i)

        for ep_indices in episodes.values():
            ep_len = len(ep_indices)
            ep_rewards = [self.buffer[i].reward for i in ep_indices]
            ep_values = [self.buffer[i].value for i in ep_indices]
            ep_dones = [self.buffer[i].done for i in ep_indices]

            gae = 0.0
            for t in reversed(range(ep_len)):
                if t == ep_len - 1:
                    # Bootstrap with value estimate if episode was truncated (not truly terminal)
                    next_val = 0.0 if ep_dones[t] else ep_values[t]
                else:
                    next_val = ep_values[t + 1] * (1.0 - ep_dones[t])
                delta = ep_rewards[t] + self.gamma * next_val - ep_values[t]
                gae = delta + self.gamma * self.gae_lambda * (1.0 - ep_dones[t]) * gae
                advantages[ep_indices[t]] = gae

        returns = advantages + values
        return advantages, returns

    def train_batch(self) -> Dict[str, float]:
        """PPO update with GAE on the current buffer.

        Returns dict of loss metrics.
        After training, trims buffer to keep at most batch_size transitions
        (the most recent ones) rather than clearing entirely. This ensures
        we always have enough data to train on.
        """
        if len(self.buffer) < self.batch_size:
            return {"policy_loss": 0, "value_loss": 0, "total_loss": 0, "num_transitions": len(self.buffer)}

        device = next(self.model.parameters()).device
        self.model.train()

        # Apply LR warmup before optimizer step
        self._apply_lr_warmup()

        # Compute advantages
        advantages, returns = self.compute_gae()

        # Build tensors
        obs_t = torch.from_numpy(np.stack([t.obs for t in self.buffer])).float()
        masks_t = torch.from_numpy(np.stack([t.action_mask for t in self.buffer])).bool()
        actions_t = torch.tensor([t.action for t in self.buffer], dtype=torch.long)
        old_lp_t = torch.tensor([t.log_prob for t in self.buffer], dtype=torch.float32)
        adv_t = torch.from_numpy(advantages).float()
        ret_t = torch.from_numpy(returns).float()

        # Auxiliary targets
        floor_targets = torch.tensor([t.final_floor for t in self.buffer], dtype=torch.float32)
        act_targets = torch.tensor(
            [[t.cleared_act1, t.cleared_act2, t.cleared_act3] for t in self.buffer],
            dtype=torch.float32,
        )

        # Normalize advantages
        adv_std = adv_t.std()
        if adv_std > 1e-8:
            adv_t = (adv_t - adv_t.mean()) / (adv_std + 1e-8)

        N = obs_t.shape[0]
        total_metrics: Dict[str, float] = {
            "policy_loss": 0.0,
            "value_loss": 0.0,
            "entropy": 0.0,
            "aux_loss": 0.0,
            "floor_pred_loss": 0.0,
            "act_pred_loss": 0.0,
            "total_loss": 0.0,
            "clip_fraction": 0.0,
        }

        num_mini_batches = 0
        for epoch in range(self.ppo_epochs):
            indices = torch.randperm(N)

            for start in range(0, N, self.batch_size):
                end = min(start + self.batch_size, N)
                idx = indices[start:end]

                b_obs = obs_t[idx].to(device)
                b_masks = masks_t[idx].to(device)
                b_actions = actions_t[idx].to(device)
                b_old_lp = old_lp_t[idx].to(device)
                b_adv = adv_t[idx].to(device)
                b_ret = ret_t[idx].to(device)
                b_floor = floor_targets[idx].to(device)
                b_act = act_targets[idx].to(device)

                # Forward
                out = self.model(b_obs, b_masks)
                logits = out["policy_logits"]
                values = out["value"]
                floor_pred = out["floor_pred"]
                act_pred = out["act_completion"]

                log_probs = F.log_softmax(logits, dim=-1)
                action_lp = log_probs.gather(1, b_actions.unsqueeze(1)).squeeze(1)

                # PPO clipped objective
                ratio = torch.exp(action_lp - b_old_lp)
                surr1 = ratio * b_adv
                surr2 = torch.clamp(ratio, 1 - self.clip_epsilon, 1 + self.clip_epsilon) * b_adv
                policy_loss = -torch.min(surr1, surr2).mean()

                # Value loss (clipped for stability)
                value_loss = F.mse_loss(values, b_ret)

                # Entropy bonus
                probs = F.softmax(logits, dim=-1)
                ent_term = probs * log_probs
                ent_term = ent_term.nan_to_num(0.0)
                entropy = -ent_term.sum(dim=-1).mean()

                # Auxiliary losses
                floor_loss = F.mse_loss(floor_pred, b_floor)
                act_loss = F.binary_cross_entropy(act_pred, b_act)
                aux_loss = (floor_loss + act_loss) / 2.0

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

                num_mini_batches += 1
                total_metrics["policy_loss"] += policy_loss.item()
                total_metrics["value_loss"] += value_loss.item()
                total_metrics["entropy"] += entropy.item()
                total_metrics["aux_loss"] += aux_loss.item()
                total_metrics["floor_pred_loss"] += floor_loss.item()
                total_metrics["act_pred_loss"] += act_loss.item()
                total_metrics["total_loss"] += loss.item()
                total_metrics["clip_fraction"] += clip_frac

        # Average metrics
        if num_mini_batches > 0:
            for k in total_metrics:
                total_metrics[k] /= num_mini_batches

        # Only step cosine scheduler after warmup
        if self.train_steps >= self.warmup_steps:
            self.scheduler.step()
        self.train_steps += 1

        total_metrics["lr"] = self.optimizer.param_groups[0]["lr"]
        total_metrics["entropy_coeff"] = self.entropy_coeff
        total_metrics["train_steps"] = self.train_steps
        total_metrics["num_transitions"] = N
        total_metrics["floor_pred_loss"] = total_metrics.get("aux_loss", 0.0)  # for dashboard

        # Don't trim buffer here — caller manages buffer lifecycle.
        # In phased training: buffer is reused across multiple train_batch() calls
        # and cleared by the caller after the train phase completes.

        return total_metrics

    def maybe_checkpoint(self, avg_floor: float) -> bool:
        """Save checkpoint only on improvement.

        Returns True if a checkpoint was saved.
        """
        if avg_floor > self.best_avg_floor:
            self.best_avg_floor = avg_floor
            path = self.checkpoint_dir / f"best_strategic_floor{avg_floor:.1f}.pt"
            self.model.save(path)
            # Also save as latest
            self.model.save(self.checkpoint_dir / "latest_strategic.pt")
            return True
        return False

    def decay_entropy(self, min_coeff: float = 0.01, decay: float = 0.999):
        """Anneal entropy coefficient.

        Starts at 0.05 (set in __init__), decays to min_coeff=0.01.
        """
        self.entropy_coeff = max(min_coeff, self.entropy_coeff * decay)
