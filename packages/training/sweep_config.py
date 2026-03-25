"""Hyperparameter sweep configuration templates and ascension breakpoints."""

from __future__ import annotations

from typing import Any, Dict, List, Tuple

from .training_config import ENTROPY_COEFF, LR_BASE, LR_SCHEDULE, LR_T_0, TEMPERATURE

DEFAULT_SWEEP_CONFIGS: List[Dict[str, Any]] = [
    # Config A: Old baseline (control)
    {"name": "baseline_control",
     "lr": LR_BASE, "lr_schedule": LR_SCHEDULE, "lr_T_0": LR_T_0,
     "batch_size": 256, "entropy_coeff": ENTROPY_COEFF, "temperature": TEMPERATURE,
     "turn_solver_ms": 50.0},
    # Config B: Tuned rewards (3x milestones, -0.3 death, 1.5x PBRS)
    {"name": "reward_tuned_v12",
     "lr": LR_BASE, "lr_schedule": LR_SCHEDULE, "lr_T_0": LR_T_0,
     "batch_size": 256, "entropy_coeff": ENTROPY_COEFF, "temperature": TEMPERATURE,
     "turn_solver_ms": 100.0},
    # Config C: Tuned rewards + MCTS strategic search
    {"name": "reward_tuned_mcts",
     "lr": LR_BASE, "lr_schedule": LR_SCHEDULE, "lr_T_0": LR_T_0,
     "batch_size": 256, "entropy_coeff": ENTROPY_COEFF, "temperature": TEMPERATURE,
     "turn_solver_ms": 100.0, "strategic_search": True},
    # Config D: Tuned rewards + BC warmup (uses best_trajectories if available)
    {"name": "reward_tuned_bc",
     "lr": LR_BASE, "lr_schedule": LR_SCHEDULE, "lr_T_0": LR_T_0,
     "batch_size": 256, "entropy_coeff": ENTROPY_COEFF, "temperature": TEMPERATURE,
     "turn_solver_ms": 100.0},
    # Config E: Full MCTS UCB (200 sims for cards, 50 for paths)
    {"name": "full_mcts_ucb",
     "lr": LR_BASE, "lr_schedule": LR_SCHEDULE, "lr_T_0": LR_T_0,
     "batch_size": 256, "entropy_coeff": ENTROPY_COEFF, "temperature": TEMPERATURE,
     "turn_solver_ms": 100.0, "mcts_enabled": True},
]

# Weekend sweep: single MCTS 500 config — all-in on deep search for 90h
WEEKEND_SWEEP_CONFIGS: List[Dict[str, Any]] = [
    {"name": "mcts_500_alphazero",
     "lr": LR_BASE, "lr_schedule": LR_SCHEDULE, "lr_T_0": LR_T_0,
     "batch_size": 256, "entropy_coeff": ENTROPY_COEFF, "temperature": TEMPERATURE,
     "turn_solver_ms": 100.0, "mcts_enabled": True, "mcts_card_sims": 500},
]

# Overnight ablation: full 5-config sweep on 8 fixed seeds, 18M model + Wrath fix
OVERNIGHT_SWEEP_CONFIGS = DEFAULT_SWEEP_CONFIGS

# V3 Ablation: 4 algorithms, 2 hours each, all start from BC checkpoint
V3_ABLATION_CONFIGS: List[Dict[str, Any]] = [
    # A) Fixed PPO — TurnSolver combat, CombatNet eval, boss HP progress
    {
        "name": "v3_ppo_fixed",
        "lr": 3e-5,
        "lr_schedule": "cosine_warm_restarts",
        "lr_T_0": LR_T_0,
        "batch_size": 256,
        "entropy_coeff": ENTROPY_COEFF,
        "temperature": TEMPERATURE,
        "turn_solver_ms": 100.0,
        "collect_games": 500,
        "train_steps": 30,
        "max_hours": 2.0,
        "boss_hp_progress": True,
        "combat_net": True,
        "algorithm": "ppo",
    },
    # B) IQL Offline RL — train on all offline data, no collection
    {
        "name": "v3_iql_offline",
        "lr": 3e-4,
        "lr_schedule": "cosine",
        "lr_T_0": LR_T_0,
        "batch_size": 256,
        "max_hours": 2.0,
        "algorithm": "iql",
        "iql_expectile": 0.7,
        "iql_temperature": 3.0,
    },
    # C) GRPO — 5 rollouts per card pick, group-relative advantages
    {
        "name": "v3_grpo",
        "lr": 3e-5,
        "lr_schedule": "cosine_warm_restarts",
        "lr_T_0": LR_T_0,
        "batch_size": 256,
        "entropy_coeff": ENTROPY_COEFF,
        "temperature": TEMPERATURE,
        "turn_solver_ms": 100.0,
        "max_hours": 2.0,
        "algorithm": "grpo",
        "grpo_rollouts_card": 5,
        "grpo_rollouts_other": 2,
    },
    # D) BC -> PPO Hybrid — same as A but starts from BC checkpoint
    {
        "name": "v3_bc_ppo_hybrid",
        "lr": 3e-5,
        "lr_schedule": "cosine_warm_restarts",
        "lr_T_0": LR_T_0,
        "batch_size": 256,
        "entropy_coeff": ENTROPY_COEFF,
        "temperature": TEMPERATURE,
        "turn_solver_ms": 100.0,
        "collect_games": 500,
        "train_steps": 30,
        "max_hours": 2.0,
        "boss_hp_progress": True,
        "combat_net": True,
        "algorithm": "ppo",
        "bc_warmup": True,
    },
]

# Adaptive ascension breakpoints: (min_avg_floor, min_win_rate, target_ascension)
ASCENSION_BREAKPOINTS: List[Tuple[float, float, int]] = [
    (17, 0.05, 1),   # Clearing Act 1 somewhat reliably -> A1
    (17, 0.15, 3),   # 15% WR -> A3
    (17, 0.30, 5),   # 30% WR -> A5
    (33, 0.10, 7),   # Reaching Act 2 boss at 10% -> A7
    (33, 0.25, 10),  # 25% WR past Act 2 -> A10
]
