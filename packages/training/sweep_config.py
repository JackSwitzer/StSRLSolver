"""Hyperparameter sweep configuration templates and ascension breakpoints."""

from __future__ import annotations

from typing import Any, Dict, List, Tuple

from .training_config import ENTROPY_COEFF, LR_BASE, LR_SCHEDULE, LR_T_0, TEMPERATURE

DEFAULT_SWEEP_CONFIGS: List[Dict[str, Any]] = [
    # Single focused config -- no weight forking, all budget goes to learning.
    # Pure on-policy: model makes decisions, log_prob from unscaled policy.
    {"name": "focused_b1024",
     "lr": LR_BASE, "lr_schedule": LR_SCHEDULE, "lr_T_0": LR_T_0,
     "batch_size": 512, "entropy_coeff": ENTROPY_COEFF, "temperature": TEMPERATURE,
     "turn_solver_ms": 30.0},
    {"name": "reward_tuned_v12",
     "lr": 1e-4, "lr_schedule": "cosine_warm_restarts", "lr_T_0": 10000,
     "batch_size": 256, "entropy_coeff": 0.05, "temperature": 0.9,
     "turn_solver_ms": 100.0},
]

# Adaptive ascension breakpoints: (min_avg_floor, min_win_rate, target_ascension)
ASCENSION_BREAKPOINTS: List[Tuple[float, float, int]] = [
    (17, 0.05, 1),   # Clearing Act 1 somewhat reliably -> A1
    (17, 0.15, 3),   # 15% WR -> A3
    (17, 0.30, 5),   # 30% WR -> A5
    (33, 0.10, 7),   # Reaching Act 2 boss at 10% -> A7
    (33, 0.25, 10),  # 25% WR past Act 2 -> A10
]
