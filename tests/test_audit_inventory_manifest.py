"""Audit tests for inventory manifest generation robustness (AUD-GEN-*)."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
SCRIPT_PATH = REPO_ROOT / "scripts/generate_parity_manifests.py"


spec = importlib.util.spec_from_file_location("generate_parity_manifests", SCRIPT_PATH)
assert spec is not None and spec.loader is not None
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)


def test_power_hook_parser_handles_multiline_and_single_quote_calls(tmp_path: Path) -> None:
    sample = tmp_path / "runtime.py"
    sample.write_text(
        """
from packages.engine.registry import execute_power_triggers

execute_power_triggers(
    \"onCardDraw\",
    state,
    owner,
)
execute_power_triggers('onScry', state, owner)
""",
        encoding="utf-8",
    )

    hooks = mod.parse_runtime_power_dispatch_hooks([sample])
    assert "onCardDraw" in hooks
    assert "onScry" in hooks


def test_parse_java_potions_falls_back_to_class_artifacts(tmp_path: Path) -> None:
    java_root = tmp_path / "decompiled" / "java-src" / "com" / "megacrit" / "cardcrawl"
    java_root.mkdir(parents=True)

    fallback_potions = (
        tmp_path
        / "decompiled"
        / "game-source"
        / "com"
        / "megacrit"
        / "cardcrawl"
        / "potions"
    )
    fallback_potions.mkdir(parents=True)

    # Include one real potion class and one inner/non-potion class to filter out.
    (fallback_potions / "AttackPotion.class").write_text("", encoding="utf-8")
    (fallback_potions / "PotionSlot.class").write_text("", encoding="utf-8")
    (fallback_potions / "AbstractPotion$1.class").write_text("", encoding="utf-8")

    parsed = mod.parse_java_potions(java_root)
    assert parsed["source_missing"] is False

    ids = [row["java_id"] for row in parsed["items"]]
    assert "AttackPotion" in ids
    assert "PotionSlot" not in ids


def test_parse_java_events_excludes_spire_heart(tmp_path: Path) -> None:
    java_root = tmp_path / "com" / "megacrit" / "cardcrawl"
    events_root = java_root / "events" / "beyond"
    events_root.mkdir(parents=True)

    (events_root / "SpireHeart.java").write_text("class SpireHeart {}", encoding="utf-8")
    (events_root / "MindBloom.java").write_text("class MindBloom {}", encoding="utf-8")

    parsed = mod.parse_java_events(java_root)
    ids = [row["java_id"] for row in parsed["items"]]
    assert "MindBloom" in ids
    assert "SpireHeart" not in ids


def test_generated_parity_diff_has_no_missing_cards_events_or_potions() -> None:
    parity_path = (
        REPO_ROOT
        / "docs/audits/2026-02-22-full-game-parity/traceability/parity-diff.json"
    )
    payload = json.loads(parity_path.read_text(encoding="utf-8"))

    assert payload["cards"]["summary"]["missing"] == 0
    assert payload["events"]["summary"]["missing"] == 0
    assert payload["potions"]["summary"]["missing"] == 0


def test_generated_power_hook_coverage_has_no_undispatched_registry_hooks() -> None:
    hooks_path = (
        REPO_ROOT
        / "docs/audits/2026-02-22-full-game-parity/traceability/power-hook-coverage.json"
    )
    payload = json.loads(hooks_path.read_text(encoding="utf-8"))
    assert payload["registered_not_dispatched"] == []
