"""Training configuration -- single source of truth for all tuneable parameters.

All training hyperparams live here. Sweep configs reference these as defaults.
Hot-reload updates this module's dicts directly.
"""

from __future__ import annotations

from typing import Any, Dict

# ---------------------------------------------------------------------------
# Model Architecture
# ---------------------------------------------------------------------------
MODEL_HIDDEN_DIM = 768
MODEL_NUM_BLOCKS = 4
MODEL_ACTION_DIM = 512

# ---------------------------------------------------------------------------
# Training
# ---------------------------------------------------------------------------
TRAIN_WORKERS = 10
TRAIN_BATCH_SIZE = 256
TRAIN_MAX_BATCH_INFERENCE = 16
TRAIN_GAMES_PER_BATCH = 16
TRAIN_PPO_EPOCHS = 4
TRAIN_STEPS_PER_PHASE = 10
TRAIN_COLLECT_GAMES = 100

# ---------------------------------------------------------------------------
# Learning Rate
# ---------------------------------------------------------------------------
LR_BASE = 1e-4
LR_SCHEDULE = "cosine_warm_restarts"
LR_T_0 = 10000
LR_WARMUP_STEPS = 100

# ---------------------------------------------------------------------------
# Exploration
# ---------------------------------------------------------------------------
ENTROPY_COEFF = 0.05
ENTROPY_MIN = 0.02
ENTROPY_FLOOR_AVG_FLOOR = 12.0  # Don't decay until avg_floor > this
TEMPERATURE = 0.9

# ---------------------------------------------------------------------------
# Solver Budgets (per room type: base_ms, base_nodes, cap_ms)
# ---------------------------------------------------------------------------
SOLVER_BUDGETS: Dict[str, tuple] = {
    "monster": (50.0, 5_000, 300_000),      # 5 min cap
    "elite":   (200.0, 20_000, 600_000),    # 10 min cap
    "boss":    (500.0, 50_000, 1_200_000),  # 20 min cap
}
SOLVER_HP_SCALE_DIVISOR = 100.0  # budget_ms = base * max(1, total_hp / this)

# Solver scoring weights — keep minimal, let model learn strategy
SOLVER_SCORING: Dict[str, float] = {
    "hp_lost_weight": -3.0,         # Per HP lost after enemy turn
    "enemy_kill_bonus": 30.0,       # Per enemy killed this turn
    "remaining_hp_weight": -10.0,   # Normalized remaining enemy HP penalty
    "turns_to_kill_weight": -5.0,   # Per estimated turn to kill
    "calm_bonus": 8.0,              # Ending turn in Calm (energy bank)
    "wrath_incoming_scale": 1.0,    # Wrath penalty = incoming * this (capped at wrath_cap)
    "wrath_cap": 15.0,              # Max Wrath penalty
    "unspent_energy_weight": -3.0,  # Per unspent energy with playable cards
    "unspent_playable_weight": -2.0,  # Per playable card left in hand
    "unspent_idle_weight": -1.0,    # Per unspent energy with no playable cards
}

# ---------------------------------------------------------------------------
# Rewards
# ---------------------------------------------------------------------------
REWARD_WEIGHTS: Dict[str, Any] = {
    # HP damage penalty REMOVED -- was actively punishing survival
    "damage_per_hp": 0.0,

    # Combat win rewards (boosted -- these are the primary progress signal)
    "combat_win": 0.30,
    "elite_win": 1.50,
    "boss_win": 5.00,

    # Floor milestones -- 5-10x boost to create strong gradient toward deeper runs
    "floor_milestones": {
        3: 0.25,     # Early progress
        6: 1.50,     # First elite territory
        10: 3.00,    # Mid-act 1
        13: 4.00,    # Late act 1
        15: 6.00,    # Final campfire
        16: 9.00,    # Reached Act 1 boss
        17: 15.00,   # Beat Act 1 boss
        25: 9.00,    # Mid-act 2
        33: 15.00,   # Reached Act 2 boss
        34: 24.00,   # Beat Act 2 boss
        50: 24.00,   # Reached Act 3 boss
        51: 36.00,   # Beat Act 3 boss
        55: 50.00,   # Beat the Heart
    },

    # F16 HP bonus: reward arriving at boss floor healthy
    "f16_hp_bonus_base": 1.50,
    "f16_hp_bonus_per_hp": 0.05,

    # Deck management
    "shop_remove": 0.40,

    # Upgrade rewards (separate from card picks)
    "upgrade_rewards": {
        "Eruption": 0.30,    "Eruption+": 0.0,
        "Vigilance": 0.10,   "Vigilance+": 0.0,
        "Defend_P": -1.50,   "Defend_P+": 0.0,
        "Strike_P": -0.50,   "Strike_P+": 0.0,
    },

    # Potions
    "potion_use_elite": 0.50,
    "potion_use_boss": 0.50,
    "potion_kill_same_fight": 0.50,
    "potion_waste_penalty": -0.15,
    "potion_hoard_penalty": -0.30,

    # Terminal rewards
    "win_reward": 10.0,
    "death_penalty_scale": -0.3,  # Multiplied by (1 - progress)
    "death_floor_cutoff": 55,     # progress = floor / this
}

# ---------------------------------------------------------------------------
# PBRS
# ---------------------------------------------------------------------------
PBRS_GAMMA = 0.99
PBRS_WEIGHTS: Dict[str, float] = {
    "floor": 1.5,
    "hp": 0.30,
    "deck_quality": 0.15,
    "relic": 0.10,
}

# ---------------------------------------------------------------------------
# MCTS
# ---------------------------------------------------------------------------
MCTS_BUDGETS: Dict[str, int] = {
    "card_pick": 200,
    "path": 50,
    "rest": 20,
    "shop": 20,
    "event": 30,
    "other": 10,
}
MCTS_UCB_C = 1.414
MCTS_BLEND_RATIO = 0.8       # MCTS weight (1 - this = model weight)
STRATEGIC_BLEND_RATIO = 0.7  # Strategic search weight

# ---------------------------------------------------------------------------
# Exploration
# ---------------------------------------------------------------------------
EXPLORE_TEMP_MULTIPLIER = 1.5  # Exploration temp = base temp * this
EXPLORE_GAME_RATIO = 4         # Every Nth game uses explore temp

# ---------------------------------------------------------------------------
# Replay
# ---------------------------------------------------------------------------
REPLAY_BUFFER_SIZE = 75
REPLAY_MIN_FLOOR = 12
REPLAY_MIX_RATIO = 0.25

# ---------------------------------------------------------------------------
# Stall Detection (effectively disabled)
# ---------------------------------------------------------------------------
STALL_DETECTION_WINDOW = 50000
STALL_IMPROVEMENT_THRESHOLD = 0.0
