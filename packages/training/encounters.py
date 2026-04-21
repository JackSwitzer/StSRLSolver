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

    # =========================================================================
    # Act 1 multi-slime hallway (referenced as "Lots of Slimes" in .run files).
    # =========================================================================
    "Lots of Slimes": EncounterSpec(
        name="Lots of Slimes",
        room_kind="hallway",
        floor_hint=12,
        enemies=(
            EncounterEnemySpec("AcidSlime_S", 8, 8, 3, 1),
            EncounterEnemySpec("AcidSlime_S", 8, 8, 3, 1),
            EncounterEnemySpec("SpikeSlime_S", 11, 11, 5, 1),
            EncounterEnemySpec("SpikeSlime_S", 11, 11, 5, 1),
        ),
    ),

    # =========================================================================
    # Act 2 hallways and elites. HP values are A0 base from StS reference; if
    # they drift from Rust engine constants the new completeness test (audit
    # §5.11) will surface the diff.
    # =========================================================================
    "Shell Parasite": EncounterSpec(
        name="Shell Parasite",
        room_kind="hallway",
        floor_hint=18,
        enemies=(EncounterEnemySpec("ShelledParasite", 68, 68, 10, 1),),
    ),
    "3 Byrds": EncounterSpec(
        name="3 Byrds",
        room_kind="hallway",
        floor_hint=20,
        enemies=(
            EncounterEnemySpec("Byrd", 28, 28, 1, 5),
            EncounterEnemySpec("Byrd", 28, 28, 1, 5),
            EncounterEnemySpec("Byrd", 28, 28, 1, 5),
        ),
    ),
    "Sentry and Sphere": EncounterSpec(
        name="Sentry and Sphere",
        room_kind="hallway",
        floor_hint=22,
        enemies=(
            EncounterEnemySpec("Sentry", 39, 39, 9, 1),
            EncounterEnemySpec("SphericGuardian", 18, 18, 10, 1),
        ),
    ),
    "Slavers": EncounterSpec(
        name="Slavers",
        room_kind="hallway",
        floor_hint=25,
        enemies=(
            EncounterEnemySpec("SlaverBlue", 46, 46, 12, 1),
            EncounterEnemySpec("SlaverRed", 46, 46, 13, 1),
        ),
    ),
    "Book of Stabbing": EncounterSpec(
        name="Book of Stabbing",
        room_kind="elite",
        floor_hint=29,
        enemies=(EncounterEnemySpec("BookOfStabbing", 168, 168, 6, 2),),
    ),
    "Centurion and Healer": EncounterSpec(
        name="Centurion and Healer",
        room_kind="hallway",
        floor_hint=31,
        enemies=(
            EncounterEnemySpec("Centurion", 76, 76, 12, 1),
            EncounterEnemySpec("Mystic", 48, 48, 8, 1),
        ),
    ),
    "Collector": EncounterSpec(
        name="Collector",
        room_kind="boss",
        floor_hint=33,
        enemies=(EncounterEnemySpec("TheCollector", 282, 282, 18, 1),),
    ),

    # =========================================================================
    # Act 3 hallways, elites, and boss.
    # =========================================================================
    "3 Shapes": EncounterSpec(
        name="3 Shapes",
        room_kind="hallway",
        floor_hint=35,
        enemies=(
            EncounterEnemySpec("Repulsor", 30, 30, 11, 1),
            EncounterEnemySpec("Spiker", 30, 30, 7, 1),
            EncounterEnemySpec("Exploder", 30, 30, 9, 1),
        ),
    ),
    "4 Shapes": EncounterSpec(
        name="4 Shapes",
        room_kind="hallway",
        floor_hint=47,
        enemies=(
            EncounterEnemySpec("Repulsor", 30, 30, 11, 1),
            EncounterEnemySpec("Spiker", 30, 30, 7, 1),
            EncounterEnemySpec("Exploder", 30, 30, 9, 1),
            EncounterEnemySpec("Repulsor", 30, 30, 11, 1),
        ),
    ),
    "3 Darklings": EncounterSpec(
        name="3 Darklings",
        room_kind="elite",
        floor_hint=36,
        enemies=(
            EncounterEnemySpec("Darkling", 48, 48, 9, 1),
            EncounterEnemySpec("Darkling", 48, 48, 9, 1),
            EncounterEnemySpec("Darkling", 48, 48, 9, 1),
        ),
    ),
    "Transient": EncounterSpec(
        name="Transient",
        room_kind="hallway",
        floor_hint=38,
        enemies=(EncounterEnemySpec("Transient", 999, 999, 99, 1),),
    ),
    "Reptomancer": EncounterSpec(
        name="Reptomancer",
        room_kind="elite",
        floor_hint=44,
        enemies=(EncounterEnemySpec("Reptomancer", 180, 180, 34, 1),),
    ),
    "Nemesis": EncounterSpec(
        name="Nemesis",
        room_kind="elite",
        floor_hint=46,
        enemies=(EncounterEnemySpec("Nemesis", 185, 185, 6, 4),),
    ),
    "Spire Growth": EncounterSpec(
        name="Spire Growth",
        room_kind="hallway",
        floor_hint=48,
        enemies=(EncounterEnemySpec("SpireGrowth", 170, 170, 16, 1),),
    ),
    "Time Eater": EncounterSpec(
        name="Time Eater",
        room_kind="boss",
        floor_hint=50,
        enemies=(EncounterEnemySpec("TimeEater", 456, 456, 7, 3),),
    ),

    # =========================================================================
    # Act 4: pre-Heart elite + Heart boss.
    # =========================================================================
    "Shield and Spear": EncounterSpec(
        name="Shield and Spear",
        room_kind="elite",
        floor_hint=54,
        enemies=(
            EncounterEnemySpec("SpireShield", 240, 240, 12, 1),
            EncounterEnemySpec("SpireSpear", 250, 250, 6, 2),
        ),
    ),
    "The Heart": EncounterSpec(
        name="The Heart",
        room_kind="boss",
        floor_hint=55,
        enemies=(EncounterEnemySpec("CorruptHeart", 750, 750, 40, 1),),
    ),
}


def encounter_spec(name: str) -> EncounterSpec:
    try:
        return ENCOUNTER_CATALOG[name]
    except KeyError as exc:
        raise KeyError(f"missing encounter spec for {name!r}") from exc
