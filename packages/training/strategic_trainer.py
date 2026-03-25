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

import logging
import math
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Optional

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F

logger = logging.getLogger(__name__)

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
        # Per-head LR multipliers (MoE-style: value head trains faster)
        from .training_config import LR_HEAD_MULTIPLIERS
        param_groups = [
            {"params": list(model.input_proj.parameters()) + list(model.trunk.parameters()),
             "lr": lr * LR_HEAD_MULTIPLIERS.get("trunk", 1.0)},
            {"params": list(model.policy_head.parameters()),
             "lr": lr * LR_HEAD_MULTIPLIERS.get("policy", 2.0)},
            {"params": list(model.value_head.parameters()),
             "lr": lr * LR_HEAD_MULTIPLIERS.get("value", 3.0)},
            {"params": list(model.floor_head.parameters()) + list(model.act_head.parameters()),
             "lr": lr * LR_HEAD_MULTIPLIERS.get("auxiliary", 1.0)},
        ]
        self.optimizer = torch.optim.Adam(param_groups, eps=1e-5)

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

                # Value loss: predict raw returns (GAE expects raw-scale values)
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

        # Diagnostic metrics
        with torch.no_grad():
            # Explained variance: how well does value predict returns?
            ret_np = ret_t.cpu().numpy()
            val_np = np.array([t.value for t in self.buffer], dtype=np.float32)
            var_ret = np.var(ret_np)
            if var_ret > 1e-8:
                total_metrics["explained_variance"] = float(1 - np.var(ret_np - val_np) / var_ret)
            else:
                total_metrics["explained_variance"] = 0.0
            total_metrics["mean_value"] = float(np.mean(val_np))
            total_metrics["mean_advantage"] = float(advantages.mean())
            total_metrics["mean_return"] = float(ret_np.mean())

            # KL divergence (approx) between old and new policy
            out_final = self.model(obs_t[:min(256, N)].to(device), masks_t[:min(256, N)].to(device))
            new_lp = F.log_softmax(out_final["policy_logits"], dim=-1)
            new_action_lp = new_lp.gather(1, actions_t[:min(256, N)].to(device).unsqueeze(1)).squeeze(1)
            kl = (old_lp_t[:min(256, N)].to(device) - new_action_lp).mean()
            total_metrics["kl_divergence"] = float(kl.cpu())

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

    def bc_pretrain(self, trajectory_dir: Path, epochs: int = 10, max_transitions: int = 48000) -> Dict[str, float]:
        """Behavioral cloning: supervised learning on expert trajectories."""
        from .training_config import MODEL_ACTION_DIM
        device = next(self.model.parameters()).device
        _ACTION_DIM = MODEL_ACTION_DIM

        traj_files = sorted(trajectory_dir.glob("traj_F*.npz"), key=lambda p: p.stem, reverse=True)
        if not traj_files:
            logger.info("BC pretrain: no trajectory files found")
            return {"bc_loss": 0, "bc_accuracy": 0}

        obs_list, mask_list, action_list, floor_list = [], [], [], []
        loaded = 0
        for tf in traj_files:
            if loaded >= max_transitions: break
            try:
                data = np.load(tf)
                n = len(data["obs"])
                for i in range(n):
                    if loaded >= max_transitions: break
                    obs_i = data["obs"][i]
                    # Skip mismatched dimensions (older trajectories)
                    if obs_i.shape[0] != self.model.input_dim:
                        continue
                    obs_list.append(obs_i)
                    mask_i = data["masks"][i]
                    if mask_i.shape[0] < _ACTION_DIM:
                        mask_i = np.pad(mask_i, (0, _ACTION_DIM - mask_i.shape[0]))
                    mask_list.append(mask_i)
                    action_list.append(int(data["actions"][i]))
                    floor_list.append(float(data["final_floors"][i]))
                    loaded += 1
            except Exception as e:
                logger.warning("BC: failed to load %s: %s", tf.name, e)
                continue

        if loaded == 0:
            return {"bc_loss": 0, "bc_accuracy": 0}

        logger.info("BC pretrain: %d transitions from %d files", loaded, len(traj_files))

        obs_t = torch.from_numpy(np.stack(obs_list)).float().to(device)
        mask_t = torch.from_numpy(np.stack(mask_list)).bool().to(device)
        action_t = torch.tensor(action_list, dtype=torch.long).to(device)
        floor_t = torch.tensor(floor_list, dtype=torch.float32).to(device)

        self.model.train()
        total_loss, total_correct, total_samples = 0.0, 0, 0

        for epoch in range(epochs):
            indices = torch.randperm(loaded)
            for start in range(0, loaded, self.batch_size):
                end = min(start + self.batch_size, loaded)
                idx = indices[start:end]
                out = self.model(obs_t[idx], mask_t[idx])
                log_probs = F.log_softmax(out["policy_logits"], dim=-1)
                bc_loss = F.nll_loss(log_probs, action_t[idx])
                bc_value_loss = F.mse_loss(out["value"], floor_t[idx])
                loss = bc_loss + 0.5 * bc_value_loss

                self.optimizer.zero_grad()
                loss.backward()
                nn.utils.clip_grad_norm_(self.model.parameters(), self.max_grad_norm)
                self.optimizer.step()

                with torch.no_grad():
                    total_correct += (out["policy_logits"].argmax(dim=-1) == action_t[idx]).sum().item()
                    total_samples += len(idx)
                    total_loss += loss.item()

            logger.info("BC epoch %d/%d: loss=%.4f, acc=%.1f%%", epoch+1, epochs,
                        total_loss / max(total_samples // self.batch_size, 1),
                        total_correct / max(total_samples, 1) * 100)

        return {"bc_loss": total_loss / max(total_samples // self.batch_size, 1),
                "bc_accuracy": total_correct / max(total_samples, 1) * 100,
                "bc_transitions": loaded}

    def decay_entropy(self, min_coeff: float = 0.01, decay: float = 0.999):
        """Anneal entropy coefficient.

        Starts at 0.05 (set in __init__), decays to min_coeff=0.01.
        """
        self.entropy_coeff = max(min_coeff, self.entropy_coeff * decay)

    def calibrate_value_head(
        self,
        learned_rewards_path: str = "data/learned_rewards.json",
        trajectory_dir: Optional[Path] = None,
        epochs: int = 5,
        max_samples: int = 3000,
    ) -> Dict[str, float]:
        """Calibrate value head using trajectory data as value targets.

        Loads card lift ratios from learned_rewards_path and trajectory
        observations. Trains the value head (only) to predict floor progress,
        giving it meaningful gradients before PPO training begins.

        Args:
            learned_rewards_path: Path to JSON with card lift ratios
            trajectory_dir: Directory with trajectory .npz files
            epochs: Training epochs for calibration
            max_samples: Maximum number of calibration samples

        Returns:
            Dict with calibration metrics (loss, samples, etc.)
        """
        import json

        device = next(self.model.parameters()).device

        # Load card lift ratios (optional — used for logging only)
        card_lift_entries = 0
        try:
            with open(learned_rewards_path) as f:
                lift_data = json.load(f)
            for card, data in lift_data.items():
                if isinstance(data, (int, float)) or (isinstance(data, dict) and "lift_ratio" in data):
                    card_lift_entries += 1
        except FileNotFoundError:
            logger.info("calibrate_value_head: %s not found, continuing with trajectory data", learned_rewards_path)
        except json.JSONDecodeError as e:
            logger.warning("calibrate_value_head: invalid JSON in %s: %s", learned_rewards_path, e)

        # Load trajectory observations
        from .training_config import MODEL_ACTION_DIM
        obs_list, value_targets = [], []
        loaded = 0

        if trajectory_dir is not None:
            traj_files = sorted(trajectory_dir.glob("traj_F*.npz"), key=lambda p: p.stem, reverse=True)
            for tf in traj_files:
                if loaded >= max_samples:
                    break
                try:
                    data = np.load(tf)
                    for i in range(len(data["obs"])):
                        if loaded >= max_samples:
                            break
                        obs_i = data["obs"][i]
                        if obs_i.shape[0] != self.model.input_dim:
                            continue
                        obs_list.append(obs_i)
                        floor_val = float(data["final_floors"][i]) if "final_floors" in data else 0.5
                        value_targets.append(floor_val)
                        loaded += 1
                except Exception as e:
                    logger.warning("calibrate_value_head: failed to load %s: %s", tf.name, e)
                    continue

        if loaded == 0:
            logger.info("calibrate_value_head: no trajectory data available")
            return {"calibration_loss": 0, "calibration_samples": 0}

        logger.info("calibrate_value_head: %d samples, %d card lift entries", loaded, card_lift_entries)

        obs_t = torch.from_numpy(np.stack(obs_list)).float().to(device)
        target_t = torch.tensor(value_targets, dtype=torch.float32).to(device)

        # Only train value head, freeze policy
        for name, param in self.model.named_parameters():
            if "value" not in name and "shared" not in name:
                param.requires_grad_(False)

        self.model.train()
        total_loss = 0.0
        num_batches = 0

        # Create dummy masks (all True for first action)
        mask_t = torch.zeros(loaded, MODEL_ACTION_DIM, dtype=torch.bool, device=device)
        mask_t[:, 0] = True

        for epoch in range(epochs):
            indices = torch.randperm(loaded)
            for start in range(0, loaded, self.batch_size):
                end = min(start + self.batch_size, loaded)
                idx = indices[start:end]

                out = self.model(obs_t[idx], mask_t[idx])
                value_pred = out["value"]
                loss = F.mse_loss(value_pred, target_t[idx])

                self.optimizer.zero_grad()
                loss.backward()
                nn.utils.clip_grad_norm_(self.model.parameters(), self.max_grad_norm)
                self.optimizer.step()

                total_loss += loss.item()
                num_batches += 1

        # Unfreeze all parameters
        for param in self.model.parameters():
            param.requires_grad_(True)

        avg_loss = total_loss / max(num_batches, 1)
        logger.info("calibrate_value_head: %d epochs, avg_loss=%.4f", epochs, avg_loss)

        return {
            "calibration_loss": avg_loss,
            "calibration_samples": loaded,
            "calibration_epochs": epochs,
            "card_lift_entries": card_lift_entries,
        }
