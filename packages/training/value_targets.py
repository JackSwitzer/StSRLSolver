"""Canonical policy/value targets for phase-1 combat training."""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from typing import Any, Mapping, Sequence


PHASE1_POTION_VOCAB: tuple[str, ...] = (
    "BlockPotion",
    "BottledMiracle",
    "DexterityPotion",
    "DistilledChaos",
    "EnergyPotion",
    "ExplosivePotion",
    "FearPotion",
    "FirePotion",
    "FlexPotion",
    "FruitJuice",
    "SpeedPotion",
    "StancePotion",
    "SwiftPotion",
)

BASE_VALUE_HEAD_NAMES: tuple[str, ...] = (
    "solve_probability",
    "expected_hp_loss",
    "expected_turns",
    "potion_spend_count",
    "setup_delta",
    "persistent_scaling_delta",
)

PHASE1_VALUE_HEAD_NAMES: tuple[str, ...] = BASE_VALUE_HEAD_NAMES + tuple(
    f"potion::{potion_id}" for potion_id in PHASE1_POTION_VOCAB
)


@dataclass(frozen=True)
class CombatValueTarget:
    """Multi-head policy/value supervision attached to one combat root state."""

    solve_probability: float
    expected_hp_loss: float
    expected_turns: float
    potion_spend_count: float
    setup_delta: float
    persistent_scaling_delta: float
    potion_spend_by_id: Mapping[str, float] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        payload = asdict(self)
        payload["potion_spend_by_id"] = dict(sorted(self.potion_spend_by_id.items()))
        return payload

    @classmethod
    def from_dict(cls, payload: Mapping[str, Any]) -> "CombatValueTarget":
        return cls(
            solve_probability=float(payload.get("solve_probability", 0.0)),
            expected_hp_loss=float(payload.get("expected_hp_loss", 0.0)),
            expected_turns=float(payload.get("expected_turns", 0.0)),
            potion_spend_count=float(payload.get("potion_spend_count", 0.0)),
            setup_delta=float(payload.get("setup_delta", 0.0)),
            persistent_scaling_delta=float(payload.get("persistent_scaling_delta", 0.0)),
            potion_spend_by_id={
                str(key): float(value)
                for key, value in dict(payload.get("potion_spend_by_id", {})).items()
            },
        )

    def to_vector(self, head_names: Sequence[str] = PHASE1_VALUE_HEAD_NAMES) -> tuple[float, ...]:
        values: list[float] = []
        for name in head_names:
            if name == "solve_probability":
                values.append(float(self.solve_probability))
            elif name == "expected_hp_loss":
                values.append(float(self.expected_hp_loss))
            elif name == "expected_turns":
                values.append(float(self.expected_turns))
            elif name == "potion_spend_count":
                values.append(float(self.potion_spend_count))
            elif name == "setup_delta":
                values.append(float(self.setup_delta))
            elif name == "persistent_scaling_delta":
                values.append(float(self.persistent_scaling_delta))
            elif name.startswith("potion::"):
                values.append(float(self.potion_spend_by_id.get(name.split("::", 1)[1], 0.0)))
            else:
                raise KeyError(f"unknown value head {name!r}")
        return tuple(values)

    @classmethod
    def from_vector(
        cls,
        head_names: Sequence[str],
        values: Sequence[float],
    ) -> "CombatValueTarget":
        if len(head_names) != len(values):
            raise ValueError("head name count must match value count")
        payload: dict[str, Any] = {"potion_spend_by_id": {}}
        for name, value in zip(head_names, values):
            if name.startswith("potion::"):
                payload["potion_spend_by_id"][name.split("::", 1)[1]] = float(value)
            else:
                payload[name] = float(value)
        return cls.from_dict(payload)
