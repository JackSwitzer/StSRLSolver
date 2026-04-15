"""Reward configuration for RL training.

Unified, hot-reloadable reward weights and shaping functions.
All reward constants are sourced from training_config.py; this module
re-exports the mutable dicts that worker.py and training_runner.py use at runtime.

Philosophy: strip heuristic shaping, let game outcomes drive learning.
PBRS potential function + game events (win/loss/milestones) are the primary signals.
"""

from __future__ import annotations

from typing import Dict

from .training_config import (
    REPLAY_BUFFER_SIZE,
    REPLAY_MIN_FLOOR,
    REPLAY_MIX_RATIO,
    REWARD_WEIGHTS,
    STALL_DETECTION_WINDOW,
    STALL_IMPROVEMENT_THRESHOLD,
)

# Re-export for hot-reload consumers (training_runner.py mutates these dicts in place)
__all__ = [
    "REWARD_WEIGHTS",
    "EVENT_REWARDS",
    "FLOOR_MILESTONES",
    "UPGRADE_REWARDS",
    "REPLAY_BUFFER_SIZE",
    "REPLAY_MIN_FLOOR",
    "REPLAY_MIX_RATIO",
    "STALL_DETECTION_WINDOW",
    "STALL_IMPROVEMENT_THRESHOLD",
    "compute_potential",
    "compute_boss_hp_progress",
]

# Mutable dicts wired from REWARD_WEIGHTS -- hot-reload updates REWARD_WEIGHTS,
# then training_runner.py propagates to these via .update().
EVENT_REWARDS: Dict[str, float] = {
    "combat_win": REWARD_WEIGHTS["combat_win"],
    "elite_win": REWARD_WEIGHTS["elite_win"],
    "boss_win": REWARD_WEIGHTS["boss_win"],
}
FLOOR_MILESTONES: Dict[int, float] = dict(REWARD_WEIGHTS["floor_milestones"])
UPGRADE_REWARDS: Dict[str, float] = dict(REWARD_WEIGHTS.get("upgrade_rewards", {}))


# ---------------------------------------------------------------------------
# PBRS potential function
# ---------------------------------------------------------------------------

def compute_potential(run_state) -> float:
    """Compute the potential Phi(s) for PBRS.

    Components:
    - floor_pct: progress through the run (floor / 55)
    - hp_pct: current health percentage
    - deck_quality: heuristic for deck composition quality

    Returns a scalar potential value.
    """
    hp_pct = run_state.current_hp / max(run_state.max_hp, 1)
    floor_pct = getattr(run_state, "floor", 0) / 55.0
    deck_size = len(getattr(run_state, "deck", []))
    # Ideal deck is 12-25 cards; penalize bloat
    if 12 <= deck_size <= 25:
        deck_quality = 1.0
    elif deck_size < 12:
        deck_quality = 0.8
    else:
        deck_quality = max(0.5, 1.0 - (deck_size - 25) * 0.02)

    # Relic count bonus (relics are always positive)
    relic_count = len(getattr(run_state, "relics", []))
    relic_bonus = min(relic_count * 0.02, 0.15)

    return 1.5 * floor_pct + 0.80 * hp_pct + 0.15 * deck_quality + 0.10 * relic_bonus


def compute_boss_hp_progress(boss_dmg_dealt: float, boss_max_hp: float) -> float:
    """Compute continuous reward for boss fight damage.

    Creates gradient between 'reached boss' and 'killed boss':
    boss_dmg_dealt / boss_max_hp * BOSS_HP_PROGRESS_SCALE

    E.g., dealing 50% of boss HP = 1.5 reward (with scale=3.0)
    """
    from .training_config import BOSS_HP_PROGRESS_SCALE
    if boss_max_hp <= 0:
        return 0.0
    return (boss_dmg_dealt / boss_max_hp) * BOSS_HP_PROGRESS_SCALE
