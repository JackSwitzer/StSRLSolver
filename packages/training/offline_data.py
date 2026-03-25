"""Offline data loader: converts trajectory files to (s, a, r, s', done) tuples.

Used by IQL trainer for offline RL on existing trajectory data.
"""

from __future__ import annotations

import logging
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Optional, Tuple

import numpy as np
import torch

logger = logging.getLogger(__name__)


@dataclass
class OfflineBatch:
    """A batch of transitions ready for training."""
    states: torch.Tensor       # [batch, obs_dim]
    actions: torch.Tensor      # [batch] int64
    rewards: torch.Tensor      # [batch] float32
    next_states: torch.Tensor  # [batch, obs_dim]
    dones: torch.Tensor        # [batch] float32 (1.0 = terminal)
    action_masks: torch.Tensor # [batch, action_dim]


class OfflineDataset:
    """Dataset of (s, a, r, s', done) transitions loaded from trajectory files.

    Supports random batch sampling and conversion to PyTorch tensors.
    """

    def __init__(
        self,
        states: np.ndarray,
        actions: np.ndarray,
        rewards: np.ndarray,
        next_states: np.ndarray,
        dones: np.ndarray,
        action_masks: np.ndarray,
    ):
        self.states = states           # [N, obs_dim] float32
        self.actions = actions         # [N] int32
        self.rewards = rewards         # [N] float32
        self.next_states = next_states # [N, obs_dim] float32
        self.dones = dones             # [N] float32
        self.action_masks = action_masks  # [N, action_dim] bool

    def __len__(self) -> int:
        return len(self.states)

    def __getitem__(self, idx: int) -> Dict[str, np.ndarray]:
        return {
            "state": self.states[idx],
            "action": self.actions[idx],
            "reward": self.rewards[idx],
            "next_state": self.next_states[idx],
            "done": self.dones[idx],
            "action_mask": self.action_masks[idx],
        }

    def sample_batch(self, batch_size: int) -> OfflineBatch:
        """Sample a random batch and return as PyTorch tensors on CPU."""
        n = len(self)
        idx = np.random.randint(0, n, size=min(batch_size, n))
        return OfflineBatch(
            states=torch.from_numpy(self.states[idx]).float(),
            actions=torch.from_numpy(self.actions[idx]).long(),
            rewards=torch.from_numpy(self.rewards[idx]).float(),
            next_states=torch.from_numpy(self.next_states[idx]).float(),
            dones=torch.from_numpy(self.dones[idx]).float(),
            action_masks=torch.from_numpy(self.action_masks[idx]).bool(),
        )

    def to_torch(self, device: torch.device) -> OfflineBatch:
        """Convert entire dataset to PyTorch tensors on device."""
        return OfflineBatch(
            states=torch.from_numpy(self.states).float().to(device),
            actions=torch.from_numpy(self.actions).long().to(device),
            rewards=torch.from_numpy(self.rewards).float().to(device),
            next_states=torch.from_numpy(self.next_states).float().to(device),
            dones=torch.from_numpy(self.dones).float().to(device),
            action_masks=torch.from_numpy(self.action_masks).bool().to(device),
        )


def load_trajectories(
    dirs: List[Path],
    max_transitions: int = 48000,
) -> OfflineDataset:
    """Load trajectory .npz files and convert to (s, a, r, s', done) tuples.

    Trajectory files contain: obs, masks, actions, rewards, dones, values,
    log_probs, final_floors, cleared_act1, floor.

    For (s, a, r, s', done):
        s  = obs[i]
        a  = actions[i]
        r  = rewards[i]
        s' = obs[i+1] if not done[i] else zeros
        done = dones[i]

    Args:
        dirs: List of directories containing traj_F*.npz files
        max_transitions: Maximum total transitions to load

    Returns:
        OfflineDataset with all loaded transitions
    """
    all_states: List[np.ndarray] = []
    all_actions: List[np.ndarray] = []
    all_rewards: List[np.ndarray] = []
    all_next_states: List[np.ndarray] = []
    all_dones: List[np.ndarray] = []
    all_masks: List[np.ndarray] = []

    loaded = 0
    files_loaded = 0
    files_failed = 0

    # Collect all trajectory files, sorted by floor (highest first)
    traj_files: List[Path] = []
    for d in dirs:
        d = Path(d)
        if d.exists():
            traj_files.extend(d.glob("traj_F*.npz"))
    traj_files.sort(key=lambda p: p.stem, reverse=True)

    for tf in traj_files:
        if loaded >= max_transitions:
            break
        try:
            data = np.load(tf)
            obs = data["obs"]          # [T, obs_dim]
            masks = data["masks"]      # [T, action_dim]
            actions = data["actions"]  # [T]
            rewards = data["rewards"]  # [T]
            dones = data["dones"]      # [T]

            T = len(obs)
            if T < 2:
                continue

            # Build (s, a, r, s', done) pairs
            # s' for step i is obs[i+1], or zeros if done or last step
            obs_dim = obs.shape[1]
            remaining = max_transitions - loaded
            n = min(T, remaining)

            states = obs[:n]
            next_states = np.zeros_like(states)
            for i in range(n):
                if i < T - 1 and not dones[i]:
                    next_states[i] = obs[i + 1]
                # else: zeros (terminal or last step)

            all_states.append(states)
            all_actions.append(actions[:n])
            all_rewards.append(rewards[:n])
            all_next_states.append(next_states)
            all_dones.append(dones[:n].astype(np.float32))
            all_masks.append(masks[:n])

            loaded += n
            files_loaded += 1

        except Exception as e:
            logger.warning("load_trajectories: failed to load %s: %s", tf.name, e)
            files_failed += 1
            continue

    if loaded == 0:
        logger.warning("load_trajectories: no transitions loaded from %d dirs", len(dirs))
        # Return empty dataset with placeholder shapes
        return OfflineDataset(
            states=np.zeros((0, 1), dtype=np.float32),
            actions=np.zeros(0, dtype=np.int32),
            rewards=np.zeros(0, dtype=np.float32),
            next_states=np.zeros((0, 1), dtype=np.float32),
            dones=np.zeros(0, dtype=np.float32),
            action_masks=np.zeros((0, 1), dtype=np.bool_),
        )

    logger.info(
        "load_trajectories: %d transitions from %d files (%d failed)",
        loaded, files_loaded, files_failed,
    )

    return OfflineDataset(
        states=np.concatenate(all_states, axis=0),
        actions=np.concatenate(all_actions, axis=0),
        rewards=np.concatenate(all_rewards, axis=0),
        next_states=np.concatenate(all_next_states, axis=0),
        dones=np.concatenate(all_dones, axis=0),
        action_masks=np.concatenate(all_masks, axis=0),
    )
