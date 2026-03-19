"""Reward configuration for RL training.

Unified, hot-reloadable reward weights and shaping functions.
All reward constants live here; worker.py and overnight.py import from this module.

Philosophy: strip heuristic shaping, let game outcomes drive learning.
PBRS potential function + game events (win/loss/milestones) are the primary signals.
"""

from __future__ import annotations

from typing import Any, Dict


# ---------------------------------------------------------------------------
# REWARD_WEIGHTS — unified, hot-reloadable reward configuration
# ---------------------------------------------------------------------------

REWARD_WEIGHTS: Dict[str, Any] = {
    # HP damage penalty REMOVED — was actively punishing survival
    "damage_per_hp": 0.0,

    # Combat win rewards (boosted — these are the primary progress signal)
    "combat_win": 0.10,
    "elite_win": 0.50,
    "boss_win": 2.00,

    # Floor milestones — 5-10x boost to create strong gradient toward deeper runs
    "floor_milestones": {
        6: 0.50,     # First elite territory
        10: 1.00,    # Mid-act 1
        15: 2.00,    # Final campfire
        16: 3.00,    # Reached Act 1 boss
        17: 5.00,    # Beat Act 1 boss
        25: 3.00,    # Mid-act 2
        33: 5.00,    # Reached Act 2 boss
        34: 8.00,    # Beat Act 2 boss
        50: 8.00,    # Reached Act 3 boss
        51: 12.00,   # Beat Act 3 boss
        55: 15.00,   # Beat the Heart
    },

    # F16 HP bonus: reward arriving at boss floor healthy
    "f16_hp_bonus_base": 0.50,
    "f16_hp_bonus_per_hp": 0.02,

    # Deck management
    "shop_remove": 0.15,

    # Upgrade rewards — empty (let outcomes drive)
    "upgrade_rewards": {},

    # Potions — zeroed (let outcomes drive)
    "potion_use_elite": 0.0,
    "potion_use_boss": 0.0,
    "potion_kill_same_fight": 0.0,
    "potion_waste_penalty": 0.0,
    "potion_hoard_penalty": 0.0,

    # Terminal rewards
    "win_reward": 10.0,
    "death_penalty_scale": -2.0,  # Multiplied by max(0, 1 - floor/cutoff)
    "death_floor_cutoff": 20,     # Floors below this get full death penalty
}

# Legacy dicts kept for hot-reload compat — values wired from REWARD_WEIGHTS
EVENT_REWARDS = {
    "combat_win": REWARD_WEIGHTS["combat_win"],
    "elite_win": REWARD_WEIGHTS["elite_win"],
    "boss_win": REWARD_WEIGHTS["boss_win"],
}
FLOOR_MILESTONES = dict(REWARD_WEIGHTS["floor_milestones"])

# Stall detection: effectively disabled (was firing on noise, pushing entropy to cap)
STALL_DETECTION_WINDOW = 50000
STALL_IMPROVEMENT_THRESHOLD = 0.0

# REMOVED: Stance rewards were dominating all other signals.
STANCE_CHANGE_REWARDS: Dict[str, float] = {}  # Zeroed — kept for hot-reload compat

# REMOVED: Card pick rewards had no deck context.
CARD_PICK_REWARDS: Dict[str, float] = {}  # Zeroed — kept for hot-reload compat

# Upgrade rewards — empty
UPGRADE_REWARDS: Dict[str, float] = dict(REWARD_WEIGHTS.get("upgrade_rewards", {}))

# Best trajectory replay constants
REPLAY_BUFFER_SIZE = 75        # Top ~15% of runs (keeps only the best)
REPLAY_MIN_FLOOR = 12          # Only replay runs that got deep into Act 1
REPLAY_MIX_RATIO = 0.25        # 25% of each batch is replayed best trajectories

# ---------------------------------------------------------------------------
# Solver budgets per room type — (time_s, node_limit)
# ---------------------------------------------------------------------------

SOLVER_BUDGETS: Dict[str, Dict[str, Any]] = {
    "monster": {"time_s": 5.0, "nodes": 50_000},
    "elite": {"time_s": 30.0, "nodes": 300_000},
    "boss": {"time_s": 60.0, "nodes": 1_000_000},
}


# ---------------------------------------------------------------------------
# PBRS potential function — simplified to floor_pct only
# ---------------------------------------------------------------------------

def compute_potential(run_state) -> float:
    """Compute the potential Phi(s) for PBRS.

    Simplified: floor progress only. HP and deck quality were adding noise.

    Returns a scalar potential value.
    """
    floor_pct = getattr(run_state, "floor", 0) / 55.0
    return floor_pct
