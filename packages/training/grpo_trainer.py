"""GRPO (Group Relative Policy Optimization) trainer.

For each decision point, runs K rollouts and computes advantages
relative to the group mean. No value head needed -- advantages come
from comparing outcomes within each group.

Reference: DeepSeek-R1 approach adapted for game decision-making.
"""

from __future__ import annotations

import logging
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F

from .strategic_net import StrategicNet, _get_device
from .training_config import (
    GRPO_CLIP,
    GRPO_LR,
    GRPO_ROLLOUTS_CARD,
    GRPO_ROLLOUTS_OTHER,
)

logger = logging.getLogger(__name__)


@dataclass
class GroupSample:
    """Single rollout within a group."""
    action_idx: int        # Action taken
    obs: np.ndarray        # State observation at decision point
    action_mask: np.ndarray  # Valid actions mask
    log_prob: float        # Log probability under current policy
    total_return: float    # Return from this rollout to episode end


@dataclass
class GroupResult:
    """A group of K rollouts from the same decision point."""
    samples: List[GroupSample]
    phase_type: str
    advantage: Optional[np.ndarray] = None  # Computed after all rollouts

    def compute_advantages(self) -> np.ndarray:
        """Compute group-relative advantages: (return - mean) / std."""
        returns = np.array([s.total_return for s in self.samples], dtype=np.float32)
        mean = returns.mean()
        std = returns.std()
        if std < 1e-8:
            self.advantage = np.zeros_like(returns)
        else:
            self.advantage = (returns - mean) / (std + 1e-8)
        return self.advantage


class GRPOTrainer:
    """Group Relative Policy Optimization trainer.

    For each strategic decision:
        1. Save runner state
        2. Play K completions from that point (each with a different action)
        3. Compute returns for each completion
        4. Group advantage = (return_k - mean(returns)) / std(returns)
        5. Policy gradient: sum_k [advantage_k * log_prob(action_k)]
    """

    def __init__(
        self,
        model: StrategicNet,
        lr: float = GRPO_LR,
        clip: float = GRPO_CLIP,
        max_grad_norm: float = 0.5,
        rollouts_card: int = GRPO_ROLLOUTS_CARD,
        rollouts_other: int = GRPO_ROLLOUTS_OTHER,
    ):
        self.model = model
        self.clip = clip
        self.max_grad_norm = max_grad_norm
        self.rollouts_card = rollouts_card
        self.rollouts_other = rollouts_other

        self.optimizer = torch.optim.Adam(model.parameters(), lr=lr, eps=1e-5)
        self.train_steps = 0

    def _get_n_rollouts(self, phase_type: str) -> int:
        """Number of rollouts based on decision type."""
        if phase_type == "card_pick":
            return self.rollouts_card
        return self.rollouts_other

    def collect_group(
        self,
        runner,
        actions: list,
        phase_type: str,
        encoder,
        reward_fn,
        n_rollouts: Optional[int] = None,
        max_steps: int = 200,
    ) -> Optional[GroupResult]:
        """Collect a group of rollouts from a decision point.

        For each rollout:
            1. Copy the runner state
            2. Take the selected action
            3. Play forward until episode ends or max_steps
            4. Compute cumulative return

        Args:
            runner: GameRunner at the decision point
            actions: Available actions at this point
            phase_type: Decision type ("card_pick", "path", "rest", etc.)
            encoder: RunStateEncoder for encoding observations
            reward_fn: Callable(runner_before, runner_after) -> float for
                       computing per-step rewards
            n_rollouts: Override number of rollouts (default from config)
            max_steps: Maximum steps per rollout

        Returns:
            GroupResult with K samples, or None if collection fails
        """
        from packages.engine.game import GamePhase

        if n_rollouts is None:
            n_rollouts = self._get_n_rollouts(phase_type)

        n_actions = len(actions)
        if n_actions == 0:
            return None

        # Limit rollouts to available actions (no point rolling out same action twice)
        n_rollouts = min(n_rollouts, n_actions)

        # Encode the current state for all samples
        rs = runner.run_state
        obs = encoder.encode(
            rs, phase_type=phase_type,
            boss_name=getattr(runner, "_boss_name", ""),
            room_type=getattr(runner, "current_room_type", ""),
            actions=actions, runner=runner,
        )

        # Get current policy for log probs
        from .training_config import MODEL_ACTION_DIM
        mask = np.zeros(MODEL_ACTION_DIM, dtype=np.bool_)
        mask[:n_actions] = True

        device = next(self.model.parameters()).device
        with torch.no_grad():
            obs_t = torch.from_numpy(obs).float().unsqueeze(0).to(device)
            mask_t = torch.from_numpy(mask).bool().unsqueeze(0).to(device)
            out = self.model(obs_t, mask_t)
            logits = out["policy_logits"].squeeze(0)
            log_probs = F.log_softmax(logits, dim=-1).cpu().numpy()

        # Select which actions to roll out
        # Use top-K by policy probability to focus on promising actions
        probs = np.exp(log_probs[:n_actions] - log_probs[:n_actions].max())
        probs /= probs.sum()
        if n_rollouts < n_actions:
            rollout_indices = np.argsort(-probs)[:n_rollouts]
        else:
            rollout_indices = np.arange(n_actions)

        samples: List[GroupSample] = []
        for action_idx in rollout_indices:
            try:
                runner_copy = runner.copy()
                runner_copy.take_action(actions[action_idx])

                # Play forward to completion
                total_return = 0.0
                step = 0
                gamma = 0.99
                discount = 1.0

                while not runner_copy.game_over and step < max_steps:
                    try:
                        avail = runner_copy.get_available_actions()
                    except (RuntimeError, ValueError, KeyError, AttributeError, IndexError) as e:
                        logger.warning("GRPO rollout: get_actions failed at step %d: %s", step, e)
                        break
                    if not avail:
                        break

                    # Simple rollout policy: first legal action
                    # (using model here would be expensive; first-legal is fast)
                    runner_copy.take_action(avail[0])

                    # Accumulate reward from reward_fn if provided
                    if reward_fn is not None:
                        try:
                            r = reward_fn(runner_copy)
                            total_return += discount * r
                            discount *= gamma
                        except (RuntimeError, ValueError, KeyError, AttributeError, IndexError) as e:
                            logger.warning("GRPO rollout: reward_fn failed at step %d: %s", step, e)

                    step += 1

                # Terminal reward based on final state
                if runner_copy.game_won:
                    total_return += discount * 10.0
                else:
                    final_floor = getattr(runner_copy.run_state, "floor", 0)
                    total_return += discount * (final_floor / 55.0)

                samples.append(GroupSample(
                    action_idx=int(action_idx),
                    obs=obs.copy(),
                    action_mask=mask.copy(),
                    log_prob=float(log_probs[action_idx]),
                    total_return=total_return,
                ))

            except Exception as e:
                logger.warning("GRPO rollout failed for action %d: %s", action_idx, e)
                continue

        if len(samples) < 2:
            return None

        group = GroupResult(samples=samples, phase_type=phase_type)
        group.compute_advantages()
        return group

    def train_batch(self, groups: List[GroupResult]) -> Dict[str, float]:
        """Train policy on a batch of group results.

        For each group, computes the GRPO policy gradient:
            L = -sum_k [advantage_k * log_prob(action_k)]
        with PPO-style clipping.

        Args:
            groups: List of GroupResult, each with computed advantages

        Returns:
            Dict of training metrics
        """
        if not groups:
            return {"policy_loss": 0.0, "groups": 0}

        device = next(self.model.parameters()).device
        self.model.train()

        # Flatten all samples with their advantages
        all_obs: List[np.ndarray] = []
        all_masks: List[np.ndarray] = []
        all_actions: List[int] = []
        all_old_lp: List[float] = []
        all_advantages: List[float] = []

        for group in groups:
            if group.advantage is None:
                group.compute_advantages()
            for i, sample in enumerate(group.samples):
                all_obs.append(sample.obs)
                all_masks.append(sample.action_mask)
                all_actions.append(sample.action_idx)
                all_old_lp.append(sample.log_prob)
                all_advantages.append(float(group.advantage[i]))

        n = len(all_obs)
        if n == 0:
            return {"policy_loss": 0.0, "groups": len(groups)}

        obs_t = torch.from_numpy(np.stack(all_obs)).float().to(device)
        masks_t = torch.from_numpy(np.stack(all_masks)).bool().to(device)
        actions_t = torch.tensor(all_actions, dtype=torch.long).to(device)
        old_lp_t = torch.tensor(all_old_lp, dtype=torch.float32).to(device)
        adv_t = torch.tensor(all_advantages, dtype=torch.float32).to(device)

        # Normalize advantages across the batch
        adv_std = adv_t.std()
        if adv_std > 1e-8:
            adv_t = (adv_t - adv_t.mean()) / (adv_std + 1e-8)

        # Forward pass
        out = self.model(obs_t, masks_t)
        logits = out["policy_logits"]
        log_probs = F.log_softmax(logits, dim=-1)
        action_lp = log_probs.gather(1, actions_t.unsqueeze(1)).squeeze(1)

        # PPO-style clipped objective
        ratio = torch.exp(action_lp - old_lp_t)
        surr1 = ratio * adv_t
        surr2 = torch.clamp(ratio, 1 - self.clip, 1 + self.clip) * adv_t
        policy_loss = -torch.min(surr1, surr2).mean()

        # Entropy bonus (encourage exploration)
        probs = F.softmax(logits, dim=-1)
        entropy = -(probs * log_probs).nan_to_num(0.0).sum(dim=-1).mean()

        loss = policy_loss - 0.01 * entropy

        self.optimizer.zero_grad()
        loss.backward()
        nn.utils.clip_grad_norm_(self.model.parameters(), self.max_grad_norm)
        self.optimizer.step()

        self.train_steps += 1

        with torch.no_grad():
            clip_frac = ((ratio - 1).abs() > self.clip).float().mean().item()

        return {
            "policy_loss": policy_loss.item(),
            "entropy": entropy.item(),
            "total_loss": loss.item(),
            "clip_fraction": clip_frac,
            "groups": len(groups),
            "samples": n,
            "advantage_std": adv_t.std().item(),
            "train_steps": self.train_steps,
        }
