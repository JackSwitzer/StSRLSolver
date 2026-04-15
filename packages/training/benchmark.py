"""Benchmark planning and frontier scoring for combat-first training."""

from __future__ import annotations

from dataclasses import dataclass

from .contracts import BenchmarkReport


@dataclass(frozen=True)
class BenchmarkConfig:
    no_regression_required: bool = True
    solve_rate_weight: float = 100.0
    hp_loss_weight: float = 1.0
    turns_weight: float = 0.25
    oracle_agreement_weight: float = 25.0
    elapsed_penalty_weight: float = 0.02
    rss_penalty_weight: float = 5.0


def frontier_score(report: BenchmarkReport, config: BenchmarkConfig | None = None) -> float:
    cfg = config or BenchmarkConfig()
    total = 0.0
    for slice_result in report.slices:
        total += slice_result.solve_rate * cfg.solve_rate_weight
        total -= slice_result.expected_hp_loss * cfg.hp_loss_weight
        total -= slice_result.expected_turns * cfg.turns_weight
        total += slice_result.oracle_top_k_agreement * cfg.oracle_agreement_weight
        total -= slice_result.p95_elapsed_ms * cfg.elapsed_penalty_weight
        total -= slice_result.p95_rss_gb * cfg.rss_penalty_weight
    return total
