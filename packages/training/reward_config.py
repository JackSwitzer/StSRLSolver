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
    # Per-HP damage penalty (was -0.03, reduced 6x — was 30x stronger than boss win)
    "damage_per_hp": -0.005,

    # Combat win rewards
    "combat_win": 0.05,
    "elite_win": 0.30,
    "boss_win": 0.80,

    # Floor milestones (one-time per game)
    "floor_milestones": {
        6: 0.10,     # First elite territory
        10: 0.15,    # Mid-act 1
        15: 0.20,    # Final campfire before Act 1 boss
        16: 0.25,    # Reached Act 1 boss
        17: 1.00,    # Beat Act 1 boss
        25: 0.50,    # Mid-act 2
        33: 1.00,    # Reached Act 2 boss
        34: 2.00,    # Beat Act 2 boss
        50: 2.00,    # Reached Act 3 boss
        51: 3.00,    # Beat Act 3 boss
        55: 5.00,    # Beat the Heart (win)
    },

    # F16 HP bonus: reward arriving at boss floor healthy
    "f16_hp_bonus_base": 0.50,
    "f16_hp_bonus_per_hp": 0.02,

    # Deck management
    "shop_remove": 0.40,

    # Upgrade rewards (separate from card picks — upgrades don't change deck size)
    "upgrade_rewards": {
        "Eruption": 0.30,    "Eruption+": 0.0,   # 2->1 cost is huge
        "Vigilance": 0.10,   "Vigilance+": 0.0,
        "Defend_P": -1.50,   "Defend_P+": 0.0,    # Bad upgrade target
        "Strike_P": -0.50,   "Strike_P+": 0.0,    # Wasted upgrade
    },

    # Potions
    "potion_use_elite": 0.50,
    "potion_use_boss": 0.50,
    "potion_kill_same_fight": 0.50,
    "potion_waste_penalty": -0.15,
    "potion_hoard_penalty": -0.30,

    # Terminal rewards
    "win_reward": 10.0,
    "death_penalty_scale": -1.0,  # Multiplied by (1 - progress)
}

# Convenience accessors for backwards compat with hot-reload and worker code
DAMAGE_TAKEN_PENALTY = REWARD_WEIGHTS["damage_per_hp"]
POTION_WASTE_PENALTY = REWARD_WEIGHTS["potion_waste_penalty"]
POTION_USE_ELITE_REWARD = REWARD_WEIGHTS["potion_use_elite"]
POTION_USE_BOSS_REWARD = REWARD_WEIGHTS["potion_use_boss"]
POTION_KILL_SAME_FIGHT = REWARD_WEIGHTS["potion_kill_same_fight"]
SHOP_REMOVE_REWARD = REWARD_WEIGHTS["shop_remove"]

# Legacy dicts kept for hot-reload compat — values wired from REWARD_WEIGHTS
EVENT_REWARDS = {
    "combat_win": REWARD_WEIGHTS["combat_win"],
    "elite_win": REWARD_WEIGHTS["elite_win"],
    "boss_win": REWARD_WEIGHTS["boss_win"],
}
FLOOR_MILESTONES = dict(REWARD_WEIGHTS["floor_milestones"])

# Stall detection: if avg floor doesn't improve over this many games, reset entropy
STALL_DETECTION_WINDOW = 2000
STALL_IMPROVEMENT_THRESHOLD = 0.5

# REMOVED: Stance rewards were dominating all other signals (Wrath 1.50 was
# the single strongest reward in the system). Model should learn when stances
# are valuable from game outcomes, not from heuristic bonuses.
STANCE_CHANGE_REWARDS: Dict[str, float] = {}  # Zeroed — kept for hot-reload compat

# REMOVED: Card pick rewards had no deck context (Rushdown is great in a stance
# deck, bad in a pressure points deck). PBRS + game outcomes are enough.
CARD_PICK_REWARDS: Dict[str, float] = {}  # Zeroed — kept for hot-reload compat

# Upgrade rewards (new — separate from card picks, checked when deck size unchanged)
UPGRADE_REWARDS: Dict[str, float] = dict(REWARD_WEIGHTS.get("upgrade_rewards", {}))

# Best trajectory replay constants
REPLAY_BUFFER_SIZE = 75        # Top ~15% of runs (keeps only the best)
REPLAY_MIN_FLOOR = 12          # Only replay runs that got deep into Act 1
REPLAY_MIX_RATIO = 0.25        # 25% of each batch is replayed best trajectories


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
