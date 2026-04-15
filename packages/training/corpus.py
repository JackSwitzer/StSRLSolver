"""Curated combat corpus planning for the Watcher A0 first milestone."""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass(frozen=True)
class BenchmarkSlicePlan:
    name: str
    description: str
    includes_opening_hand_enumeration: bool = False
    includes_potion_variants: bool = False
    includes_setup_counter_variants: bool = False


@dataclass(frozen=True)
class CorpusPlan:
    character: str
    ascension: int
    slices: tuple[BenchmarkSlicePlan, ...] = field(default_factory=tuple)


def default_watcher_a0_act1_corpus_plan() -> CorpusPlan:
    return CorpusPlan(
        character="Watcher",
        ascension=0,
        slices=(
            BenchmarkSlicePlan(
                name="curated-core",
                description="Curated hallway and elite combat states for baseline solver quality.",
                includes_potion_variants=True,
                includes_setup_counter_variants=True,
            ),
            BenchmarkSlicePlan(
                name="opening-hand-buckets",
                description="Opening-hand enumeration from the same pre-draw combat state.",
                includes_opening_hand_enumeration=True,
                includes_potion_variants=True,
            ),
            BenchmarkSlicePlan(
                name="frontier-harvest-hard",
                description="Hard states mined from search disagreement and high-entropy roots.",
                includes_potion_variants=True,
                includes_setup_counter_variants=True,
            ),
        ),
    )
