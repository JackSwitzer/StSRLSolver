"""Deterministic combat encounter definitions for Watcher Act 1 training."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class EncounterEnemySpec:
    enemy_id: str
    hp: int
    max_hp: int
    move_damage: int
    move_hits: int = 1

    def to_tuple(self) -> tuple[str, int, int, int, int]:
        return (self.enemy_id, self.hp, self.max_hp, self.move_damage, self.move_hits)


@dataclass(frozen=True)
class EncounterSpec:
    name: str
    room_kind: str
    enemies: tuple[EncounterEnemySpec, ...]
    floor_hint: int

    def to_engine_enemies(self) -> list[tuple[str, int, int, int, int]]:
        return [enemy.to_tuple() for enemy in self.enemies]


ENCOUNTER_CATALOG: dict[str, EncounterSpec] = {
    "Cultist": EncounterSpec(
        name="Cultist",
        room_kind="hallway",
        floor_hint=1,
        enemies=(EncounterEnemySpec("Cultist", 48, 48, 6, 1),),
    ),
    "Jaw Worm": EncounterSpec(
        name="Jaw Worm",
        room_kind="hallway",
        floor_hint=1,
        enemies=(EncounterEnemySpec("JawWorm", 44, 44, 11, 1),),
    ),
    "2 Louse": EncounterSpec(
        name="2 Louse",
        room_kind="hallway",
        floor_hint=2,
        enemies=(
            EncounterEnemySpec("RedLouse", 12, 12, 7, 1),
            EncounterEnemySpec("GreenLouse", 13, 13, 7, 1),
        ),
    ),
    "Small Slimes": EncounterSpec(
        name="Small Slimes",
        room_kind="hallway",
        floor_hint=2,
        enemies=(
            EncounterEnemySpec("AcidSlimeM", 35, 35, 8, 1),
            EncounterEnemySpec("SpikeSlimeM", 34, 34, 8, 1),
        ),
    ),
    "Gremlin Gang": EncounterSpec(
        name="Gremlin Gang",
        room_kind="hallway",
        floor_hint=5,
        enemies=(
            EncounterEnemySpec("MadGremlin", 20, 20, 5, 1),
            EncounterEnemySpec("SneakyGremlin", 15, 15, 10, 1),
            EncounterEnemySpec("FatGremlin", 14, 14, 4, 1),
            EncounterEnemySpec("ShieldGremlin", 17, 17, 6, 1),
        ),
    ),
    "2 Fungi Beasts": EncounterSpec(
        name="2 Fungi Beasts",
        room_kind="hallway",
        floor_hint=8,
        enemies=(
            EncounterEnemySpec("FungiBeast", 24, 24, 6, 1),
            EncounterEnemySpec("FungiBeast", 24, 24, 6, 1),
        ),
    ),
    "Red Slaver": EncounterSpec(
        name="Red Slaver",
        room_kind="hallway",
        floor_hint=5,
        enemies=(EncounterEnemySpec("RedSlaver", 50, 50, 13, 1),),
    ),
    "Blue Slaver": EncounterSpec(
        name="Blue Slaver",
        room_kind="hallway",
        floor_hint=14,
        enemies=(EncounterEnemySpec("BlueSlaver", 48, 48, 13, 1),),
    ),
    "Looter": EncounterSpec(
        name="Looter",
        room_kind="hallway",
        floor_hint=12,
        enemies=(EncounterEnemySpec("Looter", 46, 46, 10, 1),),
    ),
    "Gremlin Nob": EncounterSpec(
        name="Gremlin Nob",
        room_kind="elite",
        floor_hint=8,
        enemies=(EncounterEnemySpec("GremlinNob", 85, 85, 16, 1),),
    ),
    "Lagavulin": EncounterSpec(
        name="Lagavulin",
        room_kind="elite",
        floor_hint=8,
        enemies=(EncounterEnemySpec("Lagavulin", 112, 112, 18, 1),),
    ),
    "3 Sentries": EncounterSpec(
        name="3 Sentries",
        room_kind="elite",
        floor_hint=14,
        enemies=(
            EncounterEnemySpec("Sentry", 39, 39, 9, 1),
            EncounterEnemySpec("Sentry", 39, 39, 9, 1),
            EncounterEnemySpec("Sentry", 39, 39, 9, 1),
        ),
    ),
    "Sentries": EncounterSpec(
        name="Sentries",
        room_kind="elite",
        floor_hint=14,
        enemies=(
            EncounterEnemySpec("Sentry", 39, 39, 9, 1),
            EncounterEnemySpec("Sentry", 39, 39, 9, 1),
            EncounterEnemySpec("Sentry", 39, 39, 9, 1),
        ),
    ),
    "Slime Boss": EncounterSpec(
        name="Slime Boss",
        room_kind="boss",
        floor_hint=16,
        enemies=(EncounterEnemySpec("SlimeBoss", 140, 140, 35, 1),),
    ),
    "Hexaghost": EncounterSpec(
        name="Hexaghost",
        room_kind="boss",
        floor_hint=16,
        enemies=(EncounterEnemySpec("Hexaghost", 264, 264, 6, 6),),
    ),
    "The Guardian": EncounterSpec(
        name="The Guardian",
        room_kind="boss",
        floor_hint=16,
        enemies=(EncounterEnemySpec("TheGuardian", 240, 240, 9, 2),),
    ),
}


def encounter_spec(name: str) -> EncounterSpec:
    try:
        return ENCOUNTER_CATALOG[name]
    except KeyError as exc:
        raise KeyError(f"missing encounter spec for {name!r}") from exc
