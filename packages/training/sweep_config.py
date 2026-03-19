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

# Weekend sweep: only D+E (A-C completed 2026-03-19 afternoon)
WEEKEND_SWEEP_CONFIGS: List[Dict[str, Any]] = [
    cfg for cfg in DEFAULT_SWEEP_CONFIGS
    if cfg["name"] in ("reward_tuned_bc", "full_mcts_ucb")
]

# Overnight ablation: full 5-config sweep on 8 fixed seeds, 18M model + Wrath fix
OVERNIGHT_SWEEP_CONFIGS = DEFAULT_SWEEP_CONFIGS

# Adaptive ascension breakpoints: (min_avg_floor, min_win_rate, target_ascension)
ASCENSION_BREAKPOINTS: List[Tuple[float, float, int]] = [
    (17, 0.05, 1),   # Clearing Act 1 somewhat reliably -> A1
    (17, 0.15, 3),   # 15% WR -> A3
    (17, 0.30, 5),   # 30% WR -> A5
    (33, 0.10, 7),   # Reaching Act 2 boss at 10% -> A7
    (33, 0.25, 10),  # 25% WR past Act 2 -> A10
]
