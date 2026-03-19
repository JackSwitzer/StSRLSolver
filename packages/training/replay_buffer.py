"""Trajectory replay buffer for experience replay during training."""

from __future__ import annotations

from typing import Any, Dict, List

import numpy as np

from .training_config import REPLAY_BUFFER_SIZE, REPLAY_MIN_FLOOR


class TrajectoryReplayBuffer:
    """Priority buffer that keeps the highest-floor trajectories for replay.

    Transitions from top runs are mixed into training batches so the model
    learns from its best experiences, not just its latest (often worse) ones.
    """

    def __init__(self, max_trajectories: int = REPLAY_BUFFER_SIZE,
                 min_floor: int = REPLAY_MIN_FLOOR):
        self.max_trajectories = max_trajectories
        self.min_floor = min_floor
        self._buffer: List[Dict[str, Any]] = []  # [{floor, transitions}]
        self._total_transitions = 0

    def maybe_add(self, floor: int, transitions: List[Dict[str, Any]], won: bool) -> bool:
        """Add trajectory if it meets quality threshold. Returns True if added."""
        if floor < self.min_floor and not won:
            return False
        if not transitions:
            return False

        entry = {"floor": floor, "won": won, "transitions": transitions}

        if len(self._buffer) < self.max_trajectories:
            self._buffer.append(entry)
            self._total_transitions += len(transitions)
            return True

        # Replace worst trajectory if this one is better
        worst_idx = min(range(len(self._buffer)),
                        key=lambda i: (self._buffer[i]["won"], self._buffer[i]["floor"]))
        worst = self._buffer[worst_idx]
        if (won, floor) > (worst["won"], worst["floor"]):
            self._total_transitions -= len(worst["transitions"])
            self._buffer[worst_idx] = entry
            self._total_transitions += len(transitions)
            return True
        return False

    def sample_transitions(self, n: int) -> List[Dict[str, Any]]:
        """Sample n transitions from the buffer, weighted toward higher-floor runs."""
        if not self._buffer or self._total_transitions == 0:
            return []

        # Weight by floor^2 so better runs are sampled much more
        weights = np.array([(e["floor"] ** 2) for e in self._buffer], dtype=np.float64)
        weights /= weights.sum()

        result = []
        for _ in range(n):
            traj_idx = int(np.random.choice(len(self._buffer), p=weights))
            traj = self._buffer[traj_idx]["transitions"]
            t_idx = int(np.random.randint(len(traj)))
            result.append(traj[t_idx])
        return result

    @property
    def size(self) -> int:
        return len(self._buffer)

    @property
    def best_floor(self) -> int:
        if not self._buffer:
            return 0
        return max(e["floor"] for e in self._buffer)
