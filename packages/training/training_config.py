"""Training configuration -- single source of truth for all tuneable parameters.

All training hyperparams live here. Sweep configs reference these as defaults.
Hot-reload updates this module's dicts directly.
"""

from __future__ import annotations

from typing import Any, Dict

# ---------------------------------------------------------------------------
# Model Architecture
# ---------------------------------------------------------------------------
MODEL_HIDDEN_DIM = 1024
MODEL_NUM_BLOCKS = 8
MODEL_ACTION_DIM = 512

# ---------------------------------------------------------------------------
# Training
# ---------------------------------------------------------------------------
ALGORITHM = "ppo"  # Options: "ppo", "iql", "grpo"

TRAIN_WORKERS = 10
TRAIN_BATCH_SIZE = 256
TRAIN_MAX_BATCH_INFERENCE = 32
INFERENCE_BATCH_TIMEOUT_MS = 15.0  # Batch timeout for inference server (was 5ms)
TRAIN_GAMES_PER_BATCH = 16
TRAIN_PPO_EPOCHS = 8
TRAIN_STEPS_PER_PHASE = 30
TRAIN_COLLECT_GAMES = 100

# ---------------------------------------------------------------------------
# Learning Rate
# ---------------------------------------------------------------------------
LR_BASE = 3e-5
LR_SCHEDULE = "cosine_warm_restarts"
LR_T_0 = 10000
# Per-head LR multipliers (MoE-style: shared trunk trains slower, heads faster)
LR_HEAD_MULTIPLIERS: Dict[str, float] = {
    "trunk": 1.0,       # Shared trunk: base LR
    "policy": 2.0,      # Policy head: 2x base (needs to track changing advantage landscape)
    "value": 3.0,       # Value head: 3x base (needs to converge fast for GAE)
    "auxiliary": 1.0,    # Floor/act prediction: base LR
}

# ---------------------------------------------------------------------------
# Exploration
# ---------------------------------------------------------------------------
ENTROPY_COEFF = 0.05
ENTROPY_MIN = 0.02
TEMPERATURE = 0.9

# ---------------------------------------------------------------------------
# Solver Budgets (per room type: base_ms, base_nodes, cap_ms)
# ---------------------------------------------------------------------------
SOLVER_BUDGETS: Dict[str, tuple] = {
    "monster": (50.0, 5_000, 300_000),       # 5 min cap
    "elite":   (2_000.0, 50_000, 600_000),   # 2s base for elites
    "boss":    (120_000.0, 400_000, 600_000), # 120s base for bosses
}
SOLVER_HP_SCALE_DIVISOR = 100.0  # budget_ms = base * max(1, total_hp / this)

# Solver scoring weights — keep minimal, let model learn strategy
SOLVER_SCORING: Dict[str, float] = {
    "hp_lost_weight": -1.0,         # Per HP lost after enemy turn
    "enemy_kill_bonus": 10.0,       # Per enemy killed this turn
    "remaining_hp_weight": -1.0,    # Normalized remaining enemy HP penalty
    "turns_to_kill_weight": -1.0,   # Per estimated turn to kill
    "calm_bonus": 0.0,              # No stance preference (let search decide)
    "wrath_incoming_scale": 0.0,    # No Wrath penalty (let search decide)
    "wrath_cap": 0.0,               # No Wrath cap
    "unspent_energy_weight": -1.0,  # Per unspent energy with playable cards
    "unspent_playable_weight": -0.5,  # Per playable card left in hand
    "unspent_idle_weight": 0.0,     # No penalty for idle energy
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
    # Boss floors (16/33/50) use BOSS_ENTRY_BONUS from config, not this value.
    # Post-boss floors (17/34/51) use BOSS_SURVIVAL_BONUS from config.
    "floor_milestones": {
        3: 0.25,     # Early progress
        6: 1.50,     # First elite territory
        10: 3.00,    # Mid-act 1
        13: 4.00,    # Late act 1
        15: 6.00,    # Final campfire
        16: 2.00,    # Boss entry (overridden by BOSS_ENTRY_BONUS)
        17: 12.00,   # Beat Act 1 boss (overridden by BOSS_SURVIVAL_BONUS)
        25: 9.00,    # Mid-act 2
        33: 2.00,    # Boss entry (overridden by BOSS_ENTRY_BONUS)
        34: 22.00,   # Beat Act 2 boss (overridden by BOSS_SURVIVAL_BONUS)
        50: 2.00,    # Boss entry (overridden by BOSS_ENTRY_BONUS)
        51: 22.00,   # Beat Act 3 boss (overridden by BOSS_SURVIVAL_BONUS)
        55: 50.00,   # Beat the Heart
    },

    # F16 HP bonus: reward arriving at boss floor healthy
    "f16_hp_bonus_base": 5.00,
    "f16_hp_bonus_per_hp": 0.15,

    # Micro-rewards zeroed — let the model learn behavior
    "shop_remove": 0.0,
    "upgrade_rewards": {},
    "potion_use_elite": 0.0,
    "potion_use_boss": 0.0,
    "potion_kill_same_fight": 0.0,
    "potion_waste_penalty": 0.0,
    "potion_hoard_penalty": 0.0,
    "unspent_energy_reward": 0.0,
    "unspent_playable_reward": 0.0,
    "cards_per_turn_penalties": {},
    "cards_per_turn_bonus_threshold": 999,
    "cards_per_turn_bonus_per_card": 0.0,

    # Terminal rewards
    "win_reward": 10.0,
    "death_penalty_scale": -3.0,  # Multiplied by (1 - progress)
    "death_floor_cutoff": 55,     # progress = floor / this
}

# ---------------------------------------------------------------------------
# PBRS
# ---------------------------------------------------------------------------
PBRS_GAMMA = 0.99

# ---------------------------------------------------------------------------
# Combat MCTS toggle
# ---------------------------------------------------------------------------
# Enable combat MCTS for proper boss evaluation (200 sims/action for bosses).
# Training scripts enable this; fast collection scripts disable it for throughput.
MCTS_COMBAT_ENABLED = True

# ---------------------------------------------------------------------------
# MCTS
# ---------------------------------------------------------------------------
# Combat MCTS budgets (sims per action, by room type)
COMBAT_MCTS_BUDGETS: Dict[str, int] = {
    "monster": 5,     # Fast — monsters are easy, don't waste compute
    "elite": 20,      # Moderate search for elites
    "boss": 200,      # Deep search — bosses are the bottleneck
}

# Strategic MCTS budgets (sims per decision, by phase type)
MCTS_BUDGETS: Dict[str, int] = {
    "card_pick": 200,
    "path": 50,
    "rest": 20,
    "shop": 20,
    "event": 30,
    "other": 10,
}

# Adaptive search budget: spend more compute at critical moments
# Floor-based multipliers (budget * multiplier at these floors)
MCTS_FLOOR_MULTIPLIERS: Dict[int, float] = {
    0: 10.0,   # Neow/start: 1 minute of deep planning
    1: 5.0,    # First path choice — sets the whole run trajectory
    2: 3.0,    # Still early, high leverage
    3: 2.0,    # Tapering off
    15: 3.0,   # Pre-boss floor — rest/upgrade decision is critical
    16: 5.0,   # Boss floor — card pick after boss is high-leverage
    32: 3.0,   # Pre-Act2 boss
    33: 5.0,   # Act 2 boss floor
    49: 3.0,   # Pre-Act3 boss
    50: 5.0,   # Act 3 boss floor
}
# Phase-type multipliers for key decisions (stacks with floor multiplier)
MCTS_PHASE_MULTIPLIERS: Dict[str, float] = {
    "card_pick": 2.0,  # Card picks are the highest-leverage strategic decision
    "rest": 1.5,       # Rest vs upgrade is important
    "path": 1.0,       # Normal
    "shop": 1.0,
    "event": 1.0,
    "other": 0.5,      # Low-impact decisions get less
}
# If only 1 action available, skip search entirely (forced path)
MCTS_SKIP_FORCED = True
# Hard cap on adaptive budget to prevent 10-min single decisions
MCTS_ADAPTIVE_CAP = 5000
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
# Boss HP Progress Reward
# ---------------------------------------------------------------------------
BOSS_HP_PROGRESS_SCALE = 3.0  # boss_dmg_dealt / boss_max_hp * this

# Boss milestone split: entry bonus on room entry, survival bonus on boss beat
BOSS_ENTRY_BONUS = 2.0
BOSS_SURVIVAL_BONUS = 12.0
BOSS_FLOORS = [16, 33, 50]

# ---------------------------------------------------------------------------
# Multi-turn Solver
# ---------------------------------------------------------------------------
MULTI_TURN_DEPTH = 5        # Turns ahead for boss/elite (was 3)
MULTI_TURN_BUDGET_MS = 30_000.0  # 30s for boss multi-turn planning

# ---------------------------------------------------------------------------
# Abort Criteria (per-config)
# ---------------------------------------------------------------------------
ABORT_CLIP_FRACTION = 0.30    # Abort if clip > 30% after grace period
ABORT_VALUE_LOSS = 5.0        # Abort if value loss > 5.0 after grace period
ABORT_ENTROPY_MIN = 0.01      # Abort if entropy < 0.01 (collapsed)
ABORT_GRACE_GAMES = 2000      # No abort checks until this many games (BC->PPO needs warmup)

# ---------------------------------------------------------------------------
# IQL (Implicit Q-Learning) — Offline RL
# ---------------------------------------------------------------------------
IQL_EXPECTILE = 0.7
IQL_DISCOUNT = 0.99
IQL_LR = 3e-4
IQL_TEMPERATURE = 3.0
IQL_VALUE_HIDDEN = 512
IQL_Q_HIDDEN = 512

# ---------------------------------------------------------------------------
# GRPO (Group Relative Policy Optimization)
# ---------------------------------------------------------------------------
GRPO_ROLLOUTS_CARD = 5       # Rollouts per card pick decision
GRPO_ROLLOUTS_OTHER = 2      # Rollouts per other decision
GRPO_CLIP = 0.2
GRPO_LR = 3e-5

# ---------------------------------------------------------------------------
# Auxiliary Heads
# ---------------------------------------------------------------------------
# Weight for each auxiliary head loss. Set to 0 to disable a head.
AUX_HEADS: Dict[str, float] = {
    "floor_pred": 0.15,       # Predict final floor reached (existing)
    "act_completion": 0.10,   # P(clear act 1/2/3) (existing)
    "deck_quality": 0.10,     # Predict floor reached from current state
    "combat_horizon": 0.10,   # Predict turns until combat ends
    "win_loss": 0.20,         # P(win_run) from any state
    "boss_ready": 0.15,       # P(beat_boss) given current deck+HP+relics
}

# ---------------------------------------------------------------------------
# Value Normalization
# ---------------------------------------------------------------------------
VALUE_NORM_METHOD = "popart"  # Options: "popart", "clip", "none"
POPART_BETA = 0.0003         # EMA decay rate for PopArt running stats
MAX_RETURN = 30.0            # Clip ceiling for "clip" method; returns scaled to [0, 1]

# ---------------------------------------------------------------------------
# Stall Detection (effectively disabled)
# ---------------------------------------------------------------------------
STALL_DETECTION_WINDOW = 50000
STALL_IMPROVEMENT_THRESHOLD = 0.0
