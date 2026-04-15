"""Deterministic local selector for frontier lines before the meta model exists."""

from __future__ import annotations

from .contracts import CombatFrontierLine


def frontier_sort_key(line: CombatFrontierLine) -> tuple[float, float, float, float, float]:
    outcome = line.outcome
    return (
        -outcome.solve_probability,
        outcome.expected_hp_loss,
        outcome.potion_cost,
        -outcome.setup_value_delta - outcome.persistent_scaling_delta,
        outcome.expected_turns,
    )


def select_frontier_line(lines: tuple[CombatFrontierLine, ...]) -> CombatFrontierLine:
    if not lines:
        raise ValueError("cannot select from an empty frontier")
    return min(lines, key=frontier_sort_key)
