"""Deterministic frontier selector for phase-1 combat search outputs."""

from __future__ import annotations

from dataclasses import dataclass

from .contracts import CombatFrontierLine


@dataclass(frozen=True)
class FrontierSelection:
    """Chosen line plus the fully ordered frontier for reviewer/debug output."""

    chosen: CombatFrontierLine
    ordered_frontier: tuple[CombatFrontierLine, ...]

    def to_dict(self) -> dict[str, object]:
        return {
            "chosen_line_index": self.chosen.line_index,
            "chosen_action_prefix": list(self.chosen.action_prefix),
            "ordered_line_indices": [line.line_index for line in self.ordered_frontier],
            "ordered_action_prefixes": [list(line.action_prefix) for line in self.ordered_frontier],
        }


def frontier_sort_key(line: CombatFrontierLine) -> tuple[float, float, float, float, float, tuple[int, ...], int, float, int, int]:
    outcome = line.outcome
    return (
        -outcome.solve_probability,
        outcome.expected_hp_loss,
        outcome.potion_cost,
        -(outcome.setup_value_delta + outcome.persistent_scaling_delta),
        outcome.expected_turns,
        tuple(line.action_prefix),
        line.line_index,
        -float(line.visits),
        -int(line.expanded_nodes),
        int(line.elapsed_ms),
    )


def rank_frontier_lines(lines: tuple[CombatFrontierLine, ...]) -> tuple[CombatFrontierLine, ...]:
    if not lines:
        return ()
    return tuple(sorted(lines, key=frontier_sort_key))


def select_frontier(lines: tuple[CombatFrontierLine, ...]) -> FrontierSelection:
    ordered = rank_frontier_lines(lines)
    if not ordered:
        raise ValueError("cannot select from an empty frontier")
    return FrontierSelection(chosen=ordered[0], ordered_frontier=ordered)


def select_frontier_line(lines: tuple[CombatFrontierLine, ...]) -> CombatFrontierLine:
    return select_frontier(lines).chosen
