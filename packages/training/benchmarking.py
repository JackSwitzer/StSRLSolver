"""Frontier-style benchmark summaries for comparing training snapshots."""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass, field
from statistics import fmean


def _potion_set_label(potion_set: tuple[str, ...]) -> str:
    return "+".join(potion_set) if potion_set else "none"


@dataclass(frozen=True)
class BenchmarkFrontierPoint:
    label: str
    win_rate: float
    avg_floor: float
    throughput_gpm: float
    deck_family: str | None = None
    remove_count: int | None = None
    potion_set: tuple[str, ...] = ()
    enemy: str | None = None


@dataclass(frozen=True)
class FrontierWeights:
    win_rate: float = 0.5
    avg_floor: float = 0.3
    throughput_gpm: float = 0.2


@dataclass(frozen=True)
class FrontierGroupKey:
    deck_family: str
    remove_count: int
    potion_set: tuple[str, ...]
    enemy: str

    @property
    def potion_set_label(self) -> str:
        return _potion_set_label(self.potion_set)


@dataclass(frozen=True)
class FrontierGroupSummary:
    key: FrontierGroupKey
    labels: tuple[str, ...]
    count: int
    mean_win_rate: float
    mean_avg_floor: float
    mean_throughput_gpm: float


@dataclass(frozen=True)
class FrontierReport:
    points: tuple[BenchmarkFrontierPoint, ...]
    frontier: tuple[BenchmarkFrontierPoint, ...]
    ranking: tuple[str, ...]
    best_by_metric: dict[str, str]
    weights: FrontierWeights
    groups: tuple[FrontierGroupSummary, ...] = field(default_factory=tuple)

    def to_dict(self) -> dict[str, object]:
        return {
            "points": [asdict(point) for point in self.points],
            "frontier": [point.label for point in self.frontier],
            "ranking": list(self.ranking),
            "best_by_metric": dict(self.best_by_metric),
            "weights": asdict(self.weights),
            "groups": [
                {
                    "key": {
                        "deck_family": group.key.deck_family,
                        "remove_count": group.key.remove_count,
                        "potion_set": list(group.key.potion_set),
                        "enemy": group.key.enemy,
                        "potion_set_label": group.key.potion_set_label,
                    },
                    "labels": list(group.labels),
                    "count": group.count,
                    "mean_win_rate": group.mean_win_rate,
                    "mean_avg_floor": group.mean_avg_floor,
                    "mean_throughput_gpm": group.mean_throughput_gpm,
                }
                for group in self.groups
            ],
        }

    def to_markdown(self) -> str:
        lines = [
            "# Benchmark Frontier Report",
            "",
            "| label | win_rate | avg_floor | throughput_gpm | family | removes | potions | enemy |",
            "| --- | ---: | ---: | ---: | --- | ---: | --- | --- |",
        ]
        for label in self.ranking:
            point = next(point for point in self.points if point.label == label)
            lines.append(
                "| "
                f"{point.label} | {point.win_rate:.3f} | {point.avg_floor:.2f} | {point.throughput_gpm:.2f} | "
                f"{point.deck_family or 'n/a'} | "
                f"{'' if point.remove_count is None else point.remove_count} | "
                f"{_potion_set_label(point.potion_set)} | "
                f"{point.enemy or 'n/a'} |"
            )
        lines.append("")
        lines.append(f"Frontier: {', '.join(point.label for point in self.frontier)}")
        if self.groups:
            lines.extend(
                (
                    "",
                    "## Benchmark Groups",
                    "",
                    "| family | removes | potions | enemy | count | mean_win_rate | mean_avg_floor | mean_throughput_gpm |",
                    "| --- | ---: | --- | --- | ---: | ---: | ---: | ---: |",
                )
            )
            for group in self.groups:
                lines.append(
                    "| "
                    f"{group.key.deck_family} | "
                    f"{group.key.remove_count} | "
                    f"{group.key.potion_set_label} | "
                    f"{group.key.enemy} | "
                    f"{group.count} | "
                    f"{group.mean_win_rate:.3f} | "
                    f"{group.mean_avg_floor:.2f} | "
                    f"{group.mean_throughput_gpm:.2f} |"
                )
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


def group_frontier_points(points: list[BenchmarkFrontierPoint]) -> tuple[FrontierGroupSummary, ...]:
    buckets: dict[FrontierGroupKey, list[BenchmarkFrontierPoint]] = {}
    for point in points:
        if point.deck_family is None or point.remove_count is None or point.enemy is None:
            continue
        key = FrontierGroupKey(
            deck_family=point.deck_family,
            remove_count=point.remove_count,
            potion_set=point.potion_set,
            enemy=point.enemy,
        )
        buckets.setdefault(key, []).append(point)
    return tuple(
        FrontierGroupSummary(
            key=key,
            labels=tuple(sorted(point.label for point in grouped_points)),
            count=len(grouped_points),
            mean_win_rate=fmean(point.win_rate for point in grouped_points),
            mean_avg_floor=fmean(point.avg_floor for point in grouped_points),
            mean_throughput_gpm=fmean(point.throughput_gpm for point in grouped_points),
        )
        for key, grouped_points in sorted(
            buckets.items(),
            key=lambda item: (
                item[0].deck_family,
                item[0].remove_count,
                item[0].potion_set,
                item[0].enemy,
            ),
        )
    )


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
    groups = group_frontier_points(points)
    return FrontierReport(
        points=tuple(points),
        frontier=frontier,
        ranking=ranking,
        best_by_metric=best_by_metric,
        weights=active_weights,
        groups=groups,
    )
