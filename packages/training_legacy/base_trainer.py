"""Base trainer ABC for all training algorithms.

Defines the common interface that PPO, IQL, and GRPO trainers implement.
"""

from __future__ import annotations

import logging
from abc import ABC, abstractmethod
from pathlib import Path
from typing import Any, Dict

logger = logging.getLogger(__name__)


class BaseTrainer(ABC):
    """Abstract base class for training algorithms.

    All trainers track train_steps and provide a uniform interface
    for the training runner dispatch.
    """

    train_steps: int = 0

    @abstractmethod
    def train_step(self, batch: Any) -> Dict[str, float]:
        """Run a single training step on a batch of data.

        Args:
            batch: Algorithm-specific batch format.

        Returns:
            Dict of loss/metric names to float values.
        """
        ...

    @abstractmethod
    def save_checkpoint(self, path: Path) -> None:
        """Save trainer state (model + optimizer) to disk."""
        ...

    @abstractmethod
    def load_checkpoint(self, path: Path) -> None:
        """Restore trainer state from a checkpoint."""
        ...
