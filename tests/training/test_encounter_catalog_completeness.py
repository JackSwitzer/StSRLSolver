"""Audit §5.11 -- assert ENCOUNTER_CATALOG covers every encounter that appears
in the recorded golden run, so future runs do not regress to the unsupported
state we hit before commit 88234d5a.

If a new encounter shows up in the .run file (e.g., a different Watcher run
gets added as a golden seed), this test fails until the catalog is extended.
"""

from __future__ import annotations

import json
import os
from pathlib import Path

import pytest

from packages.training.encounters import ENCOUNTER_CATALOG
from packages.training.run_parser import parse_run_file


GOLDEN_RUN_PATH = Path(
    "/Users/jackswitzer/Library/Application Support/Steam/steamapps/"
    "common/SlayTheSpire/SlayTheSpire.app/Contents/Resources/runs/"
    "WATCHER/1776347657.run"
)


def test_catalog_covers_every_combat_in_golden_run():
    """Every `damage_taken[].enemies` string in the golden run must have a
    matching ENCOUNTER_CATALOG key."""
    if not GOLDEN_RUN_PATH.exists():
        pytest.skip(f"golden run not available at {GOLDEN_RUN_PATH}")

    run = parse_run_file(GOLDEN_RUN_PATH)
    encounters_in_run = sorted({c.encounter for c in run.combat_cases})

    missing = [e for e in encounters_in_run if e not in ENCOUNTER_CATALOG]
    assert not missing, (
        f"ENCOUNTER_CATALOG missing {len(missing)} encounter(s) "
        f"present in golden run {GOLDEN_RUN_PATH.name}: {missing}. "
        f"Add EncounterSpec entries in packages/training/encounters.py."
    )


def test_catalog_room_kinds_are_canonical():
    """Every catalog entry must declare a room_kind that the engine's
    CombatPuctConfig presets understand (hallway/elite/boss). Anything else
    falls through to the hallway preset and is silently mis-budgeted."""
    valid = {"hallway", "elite", "boss"}
    bad = [
        (name, spec.room_kind)
        for name, spec in ENCOUNTER_CATALOG.items()
        if spec.room_kind not in valid
    ]
    assert not bad, (
        f"non-canonical room_kind values: {bad}. Use one of {sorted(valid)}."
    )


def test_catalog_enemies_are_non_empty_and_consistent():
    """Every spec must declare at least one enemy with hp>=max_hp>=1."""
    bad: list[tuple[str, str]] = []
    for name, spec in ENCOUNTER_CATALOG.items():
        if not spec.enemies:
            bad.append((name, "no enemies"))
            continue
        for enemy in spec.enemies:
            if enemy.hp < 1 or enemy.max_hp < 1:
                bad.append((name, f"enemy {enemy.enemy_id} hp/max_hp <1"))
            if enemy.hp > enemy.max_hp:
                bad.append((name, f"enemy {enemy.enemy_id} hp>max_hp"))
            if enemy.move_hits < 1:
                bad.append((name, f"enemy {enemy.enemy_id} move_hits <1"))
    assert not bad, f"malformed catalog entries: {bad}"


def test_catalog_engine_ids_use_no_spaces():
    """Rust engine IDs in roll_next_move are CamelCase without spaces (e.g.
    `BookOfStabbing`, not `Book of Stabbing`). The encounter NAME may have
    spaces but the per-enemy ID must not, otherwise it falls through the
    match arm to the default-AI path."""
    bad: list[tuple[str, str]] = []
    for name, spec in ENCOUNTER_CATALOG.items():
        for enemy in spec.enemies:
            if " " in enemy.enemy_id:
                bad.append((name, enemy.enemy_id))
    assert not bad, (
        f"encounter enemy_ids must not contain spaces (got {bad}). "
        f"Use the canonical Rust ID from packages/engine-rs/src/enemies/mod.rs."
    )
