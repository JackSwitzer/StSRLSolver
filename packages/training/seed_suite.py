"""External validation seeds for phase-1 Watcher combat training."""

from __future__ import annotations

from dataclasses import asdict, dataclass
from enum import Enum
from typing import Any


class SeedSource(str, Enum):
    BAALORLORD = "Baalorlord"


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


def default_watcher_validation_seed_suite() -> tuple[ValidationSeed, ...]:
    return (
        ValidationSeed(
            label="double_defend_remove_easy_1",
            seed="9YGUT28YGS0U",
            character="Watcher",
            source=SeedSource.BAALORLORD,
            source_url="https://baalorlord.tv/runs/1660337485",
            source_ascension=20,
            suggested_eval_ascension=0,
            neow_bonus="Lose all gold, then remove Defend and Defend",
            intended_use="easy remove-heavy validation seed with strong minimalist trajectory",
            tags=("easy", "remove-heavy", "minimalist-style", "watcher"),
            notes=(
                "Source run removed two Defends at Neow and ended with a very slim list after many later removals.",
                "Good candidate for testing whether our A0 solver values early removes and low-card-count combat lines.",
            ),
        ),
        ValidationSeed(
            label="double_defend_remove_easy_2",
            seed="2XHHFJ3P7FIZ",
            character="Watcher",
            source=SeedSource.BAALORLORD,
            source_url="https://baalorlord.tv/runs/1731611785",
            source_ascension=20,
            suggested_eval_ascension=0,
            neow_bonus="Lose all gold, then remove Defend and Defend",
            intended_use="second remove-heavy easy seed for replay and variance checks",
            tags=("easy", "remove-heavy", "watcher"),
            notes=(
                "Another clean double-remove opening with different downstream rewards and elites.",
                "Useful as a paired seed against 9YGUT28YGS0U so we do not overfit one remove-heavy run.",
            ),
        ),
        ValidationSeed(
            label="double_defend_remove_minimalist",
            seed="4AWM3ECVQDEWJ",
            character="Watcher",
            source=SeedSource.BAALORLORD,
            source_url="https://baalorlord.tv/runs/1736881318",
            source_ascension=20,
            suggested_eval_ascension=0,
            neow_bonus="Lose 6 Max HP, then remove Defend and Defend",
            intended_use="minimalist-style seed with especially aggressive remove trajectory",
            tags=("easy", "remove-heavy", "minimalist-style", "watcher"),
            notes=(
                "Source run ultimately removed four Defends and four Strikes.",
                "Good proof-of-concept seed for 'upgrades/removes only' style evaluation.",
            ),
        ),
        ValidationSeed(
            label="double_defend_transform_sanctity_worship",
            seed="3FRQITS2NMXWS",
            character="Watcher",
            source=SeedSource.BAALORLORD,
            source_url="https://baalorlord.tv/runs/1661364756",
            source_ascension=20,
            suggested_eval_ascension=0,
            neow_bonus="Lose all gold, then transform Defend and Defend into Sanctity and Worship",
            intended_use="high-roll transform validation seed",
            tags=("easy", "transform", "high-roll", "watcher"),
            notes=(
                "The transformed opening is directly relevant to path/value attribution from Neow and early elites.",
            ),
        ),
        ValidationSeed(
            label="double_defend_transform_fasting_consecrate",
            seed="4KSQS5JHKT5QI",
            character="Watcher",
            source=SeedSource.BAALORLORD,
            source_url="https://baalorlord.tv/runs/1655411177",
            source_ascension=20,
            suggested_eval_ascension=0,
            neow_bonus="Lose all gold, then transform Defend and Defend into Fasting and Consecrate",
            intended_use="second transform seed with more mixed card quality than the Sanctity/Worship line",
            tags=("transform", "high-roll", "watcher"),
            notes=(
                "Useful for checking whether the model can correctly price premium transforms versus plain removals.",
            ),
        ),
        ValidationSeed(
            label="rare_card_omniscience",
            seed="1AS4LGHSY0GFL",
            character="Watcher",
            source=SeedSource.BAALORLORD,
            source_url="https://baalorlord.tv/runs/1686947698",
            source_ascension=20,
            suggested_eval_ascension=0,
            neow_bonus="Pick a random Rare card and receive Omniscience",
            intended_use="obvious high-roll seed for easy-run validation",
            tags=("easy", "rare-card", "high-roll", "watcher"),
            notes=(
                "A clean high-roll example that should be easier at A0 than in the source run.",
                "Useful for checking whether premium early rares are reflected in frontier quality.",
            ),
        ),
        ValidationSeed(
            label="neows_lament_easy",
            seed="10CWNH9IJ279B",
            character="Watcher",
            source=SeedSource.BAALORLORD,
            source_url="https://baalorlord.tv/runs/1752695764",
            source_ascension=20,
            suggested_eval_ascension=0,
            neow_bonus="Take Neow's Lament for three 1 HP fights",
            intended_use="easy-route seed for validating early combat/resource snowballing",
            tags=("easy", "neows-lament", "route", "watcher"),
            notes=(
                "Good for route/value tests where free early combats should change potion and HP economics.",
            ),
        ),
        ValidationSeed(
            label="pandoras_pressure_points",
            seed="794YZS4F0DPR",
            character="Watcher",
            source=SeedSource.BAALORLORD,
            source_url="https://baalorlord.tv/runs/1717106043",
            source_ascension=20,
            suggested_eval_ascension=0,
            neow_bonus="Swap starter relic for Pandora's Box",
            intended_use="extreme archetype seed with immediate path-defining deck mutation",
            tags=("easy", "pandoras-box", "archetype", "watcher"),
            notes=(
                "The source run opened with eight Pressure Points from Pandora's Box.",
                "Excellent for validating monitor visuals and external-seed replay because the deck identity is obvious.",
            ),
        ),
        ValidationSeed(
            label="black_star_loss_control_1",
            seed="2Q148N2V0TK80",
            character="Watcher",
            source=SeedSource.BAALORLORD,
            source_url="https://baalorlord.tv/runs/1753468869",
            source_ascension=20,
            suggested_eval_ascension=0,
            neow_bonus="Swap starter relic for Black Star",
            intended_use="loss control seed for hard-negative validation",
            tags=("loss-control", "starter-swap", "black-star", "watcher"),
            notes=(
                "Killed by Red Slaver on floor 11 in the source run.",
                "Useful as a sanity check that not every externally sourced seed is trivially strong at A0.",
            ),
        ),
        ValidationSeed(
            label="black_star_loss_control_2",
            seed="588H4D1XVAAIG",
            character="Watcher",
            source=SeedSource.BAALORLORD,
            source_url="https://baalorlord.tv/runs/1696621608",
            source_ascension=20,
            suggested_eval_ascension=0,
            neow_bonus="Swap starter relic for Black Star",
            intended_use="second loss control seed for negative-control pairing",
            tags=("loss-control", "starter-swap", "black-star", "watcher"),
            notes=(
                "Killed by 3 Byrds on floor 18 in the source run.",
                "Useful when comparing seed-suite progress across checkpoints so we can track both easy and awkward starts.",
            ),
        ),
    )
