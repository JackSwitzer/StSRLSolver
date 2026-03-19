"""Reward configuration for RL training.

Thin adapter over training_config -- all constants are defined there.
This module re-exports them for backwards compatibility and provides
the compute_potential() PBRS function.
"""

from __future__ import annotations

from typing import Any, Dict

from .training_config import (
    PBRS_GAMMA,
    REPLAY_BUFFER_SIZE,
    REPLAY_MIN_FLOOR,
    REPLAY_MIX_RATIO,
    REWARD_WEIGHTS,
    STALL_DETECTION_WINDOW,
    STALL_IMPROVEMENT_THRESHOLD,
)

# Convenience accessors for backwards compat with hot-reload and worker code
DAMAGE_TAKEN_PENALTY = REWARD_WEIGHTS["damage_per_hp"]
POTION_WASTE_PENALTY = REWARD_WEIGHTS["potion_waste_penalty"]
POTION_USE_ELITE_REWARD = REWARD_WEIGHTS["potion_use_elite"]
POTION_USE_BOSS_REWARD = REWARD_WEIGHTS["potion_use_boss"]
POTION_KILL_SAME_FIGHT = REWARD_WEIGHTS["potion_kill_same_fight"]
SHOP_REMOVE_REWARD = REWARD_WEIGHTS["shop_remove"]

# Legacy dicts kept for hot-reload compat -- values wired from REWARD_WEIGHTS
EVENT_REWARDS = {
    "combat_win": REWARD_WEIGHTS["combat_win"],
    "elite_win": REWARD_WEIGHTS["elite_win"],
    "boss_win": REWARD_WEIGHTS["boss_win"],
}
FLOOR_MILESTONES = dict(REWARD_WEIGHTS["floor_milestones"])

# REMOVED: Stance rewards were dominating all other signals. Model should learn
# when stances are valuable from game outcomes, not from heuristic bonuses.
STANCE_CHANGE_REWARDS: Dict[str, float] = {}  # Zeroed -- kept for hot-reload compat

# REMOVED: Card pick rewards had no deck context. PBRS + game outcomes are enough.
CARD_PICK_REWARDS: Dict[str, float] = {}  # Zeroed -- kept for hot-reload compat

# Upgrade rewards (separate from card picks, checked when deck size unchanged)
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

    return 0.45 * floor_pct + 0.30 * hp_pct + 0.15 * deck_quality + 0.10 * relic_bonus
