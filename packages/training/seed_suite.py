"""External validation seeds for phase-1 Watcher combat training."""

from __future__ import annotations

from dataclasses import asdict, dataclass
from enum import Enum
from typing import Any


class SeedSource(str, Enum):
    BAALORLORD = "Baalorlord"
    STEAM = "Steam"


@dataclass(frozen=True)
class ValidationSeed:
    label: str
    seed: str
    character: str
    source: SeedSource
    source_url: str
    source_ascension: int
    suggested_eval_ascension: int
    neow_bonus: str
    intended_use: str
    path_capture_status: str = "seed+neow captured, floor decisions pending"
    tags: tuple[str, ...] = ()
    notes: tuple[str, ...] = ()

    def to_dict(self) -> dict[str, Any]:
        payload = asdict(self)
        payload["source"] = self.source.value
        return payload


@dataclass(frozen=True)
class ValidationSeedSuiteReport:
    suite_name: str
    seeds: tuple[ValidationSeed, ...]
    issues: tuple[str, ...]
    notes: tuple[str, ...] = ()

    def to_dict(self) -> dict[str, Any]:
        source_counts: dict[str, int] = {}
        tag_counts: dict[str, int] = {}
        for seed in self.seeds:
            source_counts[seed.source.value] = source_counts.get(seed.source.value, 0) + 1
            for tag in seed.tags:
                tag_counts[tag] = tag_counts.get(tag, 0) + 1
        return {
            "suite_name": self.suite_name,
            "seed_count": len(self.seeds),
            "labels": [seed.label for seed in self.seeds],
            "seeds": [seed.to_dict() for seed in self.seeds],
            "source_counts": dict(sorted(source_counts.items())),
            "tag_counts": dict(sorted(tag_counts.items())),
            "issues": list(self.issues),
            "notes": list(self.notes),
            "all_watcher": all(seed.character == "Watcher" for seed in self.seeds),
            "all_eval_ascension_zero": all(seed.suggested_eval_ascension == 0 for seed in self.seeds),
        }

    def to_markdown(self) -> str:
        lines = [
            "# Watcher Validation Seed Suite Report",
            "",
            "| label | seed | source | eval_asc | neow_bonus | intended_use |",
            "| --- | --- | --- | ---: | --- | --- |",
        ]
        for seed in self.seeds:
            lines.append(
                "| "
                f"{seed.label} | {seed.seed} | {seed.source.value} | {seed.suggested_eval_ascension} | "
                f"{seed.neow_bonus} | {seed.intended_use} |"
            )
        lines.extend(
            (
                "",
                f"Issues: {', '.join(self.issues) if self.issues else 'none'}",
            )
        )
        return "\n".join(lines)


def default_watcher_validation_seed_suite() -> tuple[ValidationSeed, ...]:
    return (
        ValidationSeed(
            label="minimalist_remove",
            seed="4AWM3ECVQDEWJ",
            character="Watcher",
            source=SeedSource.BAALORLORD,
            source_url="https://baalorlord.tv/runs/1736881318",
            source_ascension=20,
            suggested_eval_ascension=0,
            neow_bonus="Lose 6 Max HP, then remove Defend and Defend",
            intended_use="primary remove-heavy minimalist-style validation seed",
            tags=("remove-heavy", "minimalist-style", "watcher"),
            notes=(
                "Source run ultimately removed four Defends and four Strikes.",
                "Good proof-of-concept seed for upgrades/removes-only style evaluation.",
            ),
        ),
        ValidationSeed(
            label="lesson_learned_shell",
            seed="4VM6JKC3KR3TD",
            character="Watcher",
            source=SeedSource.BAALORLORD,
            source_url="https://baalorlord.tv/runs/1744916840",
            source_ascension=20,
            suggested_eval_ascension=0,
            neow_bonus="Lose 6 Max HP, then pick Lesson Learned over Establishment and Wish",
            intended_use="stance-dance / lesson-learned validation seed",
            tags=("lesson-learned", "stance-dance", "watcher"),
            notes=(
                "Source run is Watcher victory on an A20 run with a recognizable Lesson Learned shell.",
                "Useful for checking whether the solver handles premium skill-heavy lines.",
            ),
        ),
        ValidationSeed(
            label="icecream_runic_pyramid",
            seed="1TPMUARFP690B",
            character="Watcher",
            source=SeedSource.STEAM,
            source_url="https://steamcommunity.com/app/646570/discussions/0/3667553591708386502/?l=schinese",
            source_ascension=20,
            suggested_eval_ascension=0,
            neow_bonus="Take Ice Cream and later first boss gives Runic Pyramid",
            intended_use="retain / control validation seed from a community thread",
            tags=("retain", "control", "steam", "watcher"),
            notes=(
                "Community-reported Watcher seed with Ice Cream into Runic Pyramid.",
                "Good for testing retain-heavy control loops and hand-size planning.",
            ),
        ),
    )


def validate_watcher_validation_seed_suite(
    seeds: tuple[ValidationSeed, ...] | None = None,
) -> ValidationSeedSuiteReport:
    active_seeds = seeds or default_watcher_validation_seed_suite()
    issues: list[str] = []
    if len(active_seeds) != 3:
        issues.append(f"expected 3 seeds, found {len(active_seeds)}")
    if len({seed.label for seed in active_seeds}) != len(active_seeds):
        issues.append("duplicate labels found")
    if len({seed.seed for seed in active_seeds}) != len(active_seeds):
        issues.append("duplicate seed strings found")
    if not all(seed.character == "Watcher" for seed in active_seeds):
        issues.append("all seeds must be Watcher")
    if not all(seed.suggested_eval_ascension == 0 for seed in active_seeds):
        issues.append("all seeds must evaluate at ascension 0")
    if not all(seed.source_url for seed in active_seeds):
        issues.append("all seeds must carry a source_url")
    return ValidationSeedSuiteReport(
        suite_name="watcher_validation_suite",
        seeds=active_seeds,
        issues=tuple(issues),
        notes=(
            "Fixed 3-seed validation suite for A0 Watcher phase-1 evaluation.",
        ),
    )
