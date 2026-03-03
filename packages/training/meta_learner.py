"""
Meta-Learner: learns combat strategy weights from completed combat outcomes.

Uses simple Q-learning over a small state/action space:
- State features: player HP %, enemy count, energy, stance category, hand quality
- Actions: aggressive (max damage), defensive (max block), balanced
- Reward: -hp_lost + kills * bonus

Updates line scoring weights used by CombatPlanner. Retrains every N episodes.
"""

from __future__ import annotations

import json
import math
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

import numpy as np


# Strategy actions
AGGRESSIVE = 0
DEFENSIVE = 1
BALANCED = 2
STRATEGY_NAMES = ["aggressive", "defensive", "balanced"]
NUM_STRATEGIES = 3

# State discretization bins
HP_BINS = 4       # [0-25%, 25-50%, 50-75%, 75-100%]
ENEMY_BINS = 3    # [1, 2, 3+]
ENERGY_BINS = 3   # [0-1, 2-3, 4+]
STANCE_BINS = 4   # [Neutral, Wrath, Calm, Divinity]
HAND_BINS = 3     # [poor, ok, good]

NUM_STATES = HP_BINS * ENEMY_BINS * ENERGY_BINS * STANCE_BINS * HAND_BINS  # 432


@dataclass
class CombatLog:
    """Compressed log of a single combat for meta-learner training."""
    floor: int = 0
    enemy_id: str = ""
    turns: int = 0
    hp_lost: int = 0
    damage_dealt: int = 0
    was_lethal_available: bool = False
    used_potion: bool = False
    lines_considered: int = 0
    line_chosen_rank: int = 0
    expected_hp_loss: int = 0
    actual_hp_loss: int = 0
    strategy_used: int = BALANCED  # Which strategy was chosen each turn
    # Per-turn snapshots (compact)
    turn_snapshots: List[Dict[str, Any]] = field(default_factory=list)


def _discretize_hp(hp_pct: float) -> int:
    if hp_pct <= 0.25:
        return 0
    elif hp_pct <= 0.50:
        return 1
    elif hp_pct <= 0.75:
        return 2
    return 3


def _discretize_enemies(count: int) -> int:
    return min(count - 1, 2) if count > 0 else 0


def _discretize_energy(energy: int) -> int:
    if energy <= 1:
        return 0
    elif energy <= 3:
        return 1
    return 2


def _discretize_stance(stance: str) -> int:
    return {"Neutral": 0, "Wrath": 1, "Calm": 2, "Divinity": 3}.get(stance, 0)


def _discretize_hand_quality(hand_score: float) -> int:
    """hand_score: average card quality in hand (0-1 scale)."""
    if hand_score < 0.3:
        return 0  # poor
    elif hand_score < 0.6:
        return 1  # ok
    return 2  # good


def _state_index(hp_bin: int, enemy_bin: int, energy_bin: int, stance_bin: int, hand_bin: int) -> int:
    """Flatten discrete state to single index."""
    idx = hp_bin
    idx = idx * ENEMY_BINS + enemy_bin
    idx = idx * ENERGY_BINS + energy_bin
    idx = idx * STANCE_BINS + stance_bin
    idx = idx * HAND_BINS + hand_bin
    return idx


class CombatMetaLearner:
    """Learns from combat outcomes to improve line scoring weights."""

    def __init__(
        self,
        learning_rate: float = 0.1,
        discount: float = 0.95,
        epsilon: float = 0.15,
    ):
        self.lr = learning_rate
        self.discount = discount
        self.epsilon = epsilon

        # Q-table: state x action
        self.q_table = np.zeros((NUM_STATES, NUM_STRATEGIES), dtype=np.float32)

        # Strategy weight modifiers (applied to line scoring)
        # aggressive: boost damage weight, reduce block weight
        # defensive: boost block weight, reduce damage weight
        # balanced: default weights
        self.strategy_modifiers = {
            AGGRESSIVE: {"damage_weight": 3.0, "block_weight": 0.5, "kill_bonus": 80.0},
            DEFENSIVE:  {"damage_weight": 1.0, "block_weight": 3.0, "kill_bonus": 30.0},
            BALANCED:   {"damage_weight": 2.0, "block_weight": 1.5, "kill_bonus": 50.0},
        }

        self.episodes_seen = 0
        self.total_updates = 0

    def get_state_index(
        self,
        hp_pct: float,
        enemy_count: int,
        energy: int,
        stance: str,
        hand_quality: float,
    ) -> int:
        """Convert continuous features to discrete state index."""
        return _state_index(
            _discretize_hp(hp_pct),
            _discretize_enemies(enemy_count),
            _discretize_energy(energy),
            _discretize_stance(stance),
            _discretize_hand_quality(hand_quality),
        )

    def get_strategy(
        self,
        hp_pct: float,
        enemy_count: int,
        energy: int,
        stance: str,
        hand_quality: float,
    ) -> int:
        """Choose strategy using epsilon-greedy policy."""
        if np.random.random() < self.epsilon:
            return np.random.randint(NUM_STRATEGIES)

        state_idx = self.get_state_index(hp_pct, enemy_count, energy, stance, hand_quality)
        return int(np.argmax(self.q_table[state_idx]))

    def get_strategy_weights(
        self,
        hp_pct: float,
        enemy_count: int,
        energy: int,
        stance: str,
        hand_quality: float,
    ) -> Dict[str, float]:
        """Returns scoring weights for the chosen strategy."""
        strategy = self.get_strategy(hp_pct, enemy_count, energy, stance, hand_quality)
        return self.strategy_modifiers[strategy]

    def update(self, combat_log: CombatLog) -> None:
        """Update Q-table from a completed combat's turn snapshots."""
        snapshots = combat_log.turn_snapshots
        if not snapshots:
            return

        for i, snap in enumerate(snapshots):
            state_idx = self.get_state_index(
                snap.get("hp_pct", 0.5),
                snap.get("enemy_count", 1),
                snap.get("energy", 3),
                snap.get("stance", "Neutral"),
                snap.get("hand_quality", 0.5),
            )
            action = snap.get("strategy", BALANCED)

            # Reward: negative HP lost + kill bonus
            reward = -snap.get("hp_lost_this_turn", 0) + snap.get("kills_this_turn", 0) * 10.0

            # Next state value
            if i + 1 < len(snapshots):
                next_snap = snapshots[i + 1]
                next_idx = self.get_state_index(
                    next_snap.get("hp_pct", 0.5),
                    next_snap.get("enemy_count", 1),
                    next_snap.get("energy", 3),
                    next_snap.get("stance", "Neutral"),
                    next_snap.get("hand_quality", 0.5),
                )
                next_value = float(np.max(self.q_table[next_idx]))
            else:
                # Terminal: bonus if player survived (HP > 0), penalty if died
                player_survived = snap.get("player_hp", 0) - combat_log.hp_lost > 0
                next_value = 20.0 if player_survived else -20.0

            # Q-learning update
            current = self.q_table[state_idx, action]
            target = reward + self.discount * next_value
            self.q_table[state_idx, action] += self.lr * (target - current)

        self.episodes_seen += 1
        self.total_updates += len(snapshots)

    def update_batch(self, combat_logs: List[CombatLog]) -> None:
        """Update from a batch of combat logs."""
        for log in combat_logs:
            self.update(log)

    def save(self, path: Path) -> None:
        """Save Q-table and config to disk."""
        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)
        data = {
            "q_table": self.q_table.tolist(),
            "lr": self.lr,
            "discount": self.discount,
            "epsilon": self.epsilon,
            "episodes_seen": self.episodes_seen,
            "total_updates": self.total_updates,
        }
        with open(path, "w") as f:
            json.dump(data, f)

    def load(self, path: Path) -> bool:
        """Load Q-table from disk. Returns True if loaded successfully."""
        path = Path(path)
        if not path.exists():
            return False
        try:
            with open(path) as f:
                data = json.load(f)
            self.q_table = np.array(data["q_table"], dtype=np.float32)
            self.lr = data.get("lr", self.lr)
            self.discount = data.get("discount", self.discount)
            self.epsilon = data.get("epsilon", self.epsilon)
            self.episodes_seen = data.get("episodes_seen", 0)
            self.total_updates = data.get("total_updates", 0)
            return True
        except Exception:
            return False

    def decay_epsilon(self, min_epsilon: float = 0.05) -> None:
        """Decay exploration rate."""
        self.epsilon = max(min_epsilon, self.epsilon * 0.995)

    def get_stats(self) -> Dict[str, Any]:
        """Return learner statistics."""
        q_nonzero = int(np.count_nonzero(self.q_table))
        q_max = float(np.max(self.q_table)) if q_nonzero > 0 else 0.0
        q_min = float(np.min(self.q_table[self.q_table != 0])) if q_nonzero > 0 else 0.0
        return {
            "episodes_seen": self.episodes_seen,
            "total_updates": self.total_updates,
            "epsilon": round(self.epsilon, 4),
            "q_nonzero": q_nonzero,
            "q_total_cells": NUM_STATES * NUM_STRATEGIES,
            "q_max": round(q_max, 2),
            "q_min": round(q_min, 2),
        }
