"""Frontier-style benchmark summaries for comparing training snapshots."""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass


@dataclass(frozen=True)
class BenchmarkFrontierPoint:
    label: str
    win_rate: float
    avg_floor: float
    throughput_gpm: float


@dataclass(frozen=True)
class FrontierWeights:
    win_rate: float = 0.5
    avg_floor: float = 0.3
    throughput_gpm: float = 0.2


@dataclass(frozen=True)
class FrontierReport:
    points: tuple[BenchmarkFrontierPoint, ...]
    frontier: tuple[BenchmarkFrontierPoint, ...]
    ranking: tuple[str, ...]
    best_by_metric: dict[str, str]
    weights: FrontierWeights

    def to_dict(self) -> dict[str, object]:
        return {
            "points": [asdict(point) for point in self.points],
            "frontier": [point.label for point in self.frontier],
            "ranking": list(self.ranking),
            "best_by_metric": dict(self.best_by_metric),
            "weights": asdict(self.weights),
        }

    def to_markdown(self) -> str:
        lines = [
            "# Benchmark Frontier Report",
            "",
            "| label | win_rate | avg_floor | throughput_gpm |",
            "| --- | ---: | ---: | ---: |",
        ]
        for label in self.ranking:
            point = next(point for point in self.points if point.label == label)
            lines.append(
                f"| {point.label} | {point.win_rate:.3f} | {point.avg_floor:.2f} | {point.throughput_gpm:.2f} |"
            )
        lines.append("")
        lines.append(f"Frontier: {', '.join(point.label for point in self.frontier)}")
        return "\n".join(lines)

    def to_json(self) -> str:
        return json.dumps(self.to_dict(), indent=2, sort_keys=True)


def pareto_frontier(points: list[BenchmarkFrontierPoint]) -> list[BenchmarkFrontierPoint]:
    frontier: list[BenchmarkFrontierPoint] = []
    for candidate in points:
        dominated = False
        for other in points:
            if other.label == candidate.label:
                continue
            better_or_equal = (
                other.win_rate >= candidate.win_rate
                and other.avg_floor >= candidate.avg_floor
                and other.throughput_gpm >= candidate.throughput_gpm
            )
            strictly_better = (
                other.win_rate > candidate.win_rate
                or other.avg_floor > candidate.avg_floor
                or other.throughput_gpm > candidate.throughput_gpm
            )
            if better_or_equal and strictly_better:
                dominated = True
                break
        if not dominated:
            frontier.append(candidate)
    return sorted(frontier, key=lambda point: point.label)


def build_frontier_report(
    points: list[BenchmarkFrontierPoint],
    *,
    weights: FrontierWeights | None = None,
) -> FrontierReport:
    active_weights = weights or FrontierWeights()
    ranking = tuple(
        point.label
        for point in sorted(
            points,
            key=lambda point: (
                point.win_rate * active_weights.win_rate
                + point.avg_floor * active_weights.avg_floor
                + point.throughput_gpm * active_weights.throughput_gpm
            ),
            reverse=True,
        )
    )
    frontier = tuple(pareto_frontier(points))
    best_by_metric = {
        "win_rate": max(points, key=lambda point: point.win_rate).label,
        "avg_floor": max(points, key=lambda point: point.avg_floor).label,
        "throughput_gpm": max(points, key=lambda point: point.throughput_gpm).label,
    }
    return FrontierReport(
        points=tuple(points),
        frontier=frontier,
        ranking=ranking,
        best_by_metric=best_by_metric,
        weights=active_weights,
    )
