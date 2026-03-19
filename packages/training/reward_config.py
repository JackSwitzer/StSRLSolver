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
    # (F16 runs had WORSE reward than F6 runs due to accumulated damage)
    "damage_per_hp": 0.0,

    # Combat win rewards (boosted — these are the primary progress signal)
    "combat_win": 0.10,
    "elite_win": 0.50,
    "boss_win": 2.00,

    # Floor milestones — 5-10x boost to create strong gradient toward deeper runs
    "floor_milestones": {
        6: 0.50,     # First elite territory (was 0.10)
        10: 1.00,    # Mid-act 1 (was 0.15)
        15: 2.00,    # Final campfire (was 0.20)
        16: 3.00,    # Reached Act 1 boss (was 0.25)
        17: 5.00,    # Beat Act 1 boss (was 1.00)
        25: 3.00,    # Mid-act 2 (was 0.50)
        33: 5.00,    # Reached Act 2 boss (was 1.00)
        34: 8.00,    # Beat Act 2 boss (was 2.00)
        50: 8.00,    # Reached Act 3 boss (was 2.00)
        51: 12.00,   # Beat Act 3 boss (was 3.00)
        55: 15.00,   # Beat the Heart (was 5.00)
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

# Stall detection: effectively disabled (was firing on noise, pushing entropy to cap)
STALL_DETECTION_WINDOW = 50000
STALL_IMPROVEMENT_THRESHOLD = 0.0

# REMOVED: Stance rewards were dominating all other signals (Wrath 1.50 was
# the single strongest reward in the system). Model should learn when stances
# are valuable from game outcomes, not from heuristic bonuses.
STANCE_CHANGE_REWARDS: Dict[str, float] = {}  # Zeroed — kept for hot-reload compat

# REMOVED: Card pick rewards had no deck context (Rushdown is great in a stance
# deck, bad in a pressure points deck). PBRS + game outcomes are enough.
CARD_PICK_REWARDS: Dict[str, float] = {}  # Zeroed — kept for hot-reload compat

# Upgrade rewards (new — separate from card picks, checked when deck size unchanged)
UPGRADE_REWARDS: Dict[str, float] = dict(REWARD_WEIGHTS.get("upgrade_rewards", {}))

# Solver time budgets per room type: (base_ms, node_budget, max_ms_cap)
# Worker scales dynamically: budget_ms = base_ms * max(1.0, total_enemy_hp / 100.0)
SOLVER_BUDGETS: Dict[str, tuple] = {
    "monster": (50.0, 5_000, 300_000),       # 5min cap
    "elite":   (200.0, 20_000, 600_000),     # 10min cap
    "boss":    (500.0, 50_000, 1_200_000),   # 20min cap
}

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
