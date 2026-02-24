"""Audit tests for CRD-INV-003 alias closure rows."""

from __future__ import annotations

from packages.engine.content.cards import get_card, resolve_card_id


JAVA_CLASS_ID_ALIASES = {
    "Alchemize": "Venomology",
    "Apparition": "Ghostly",
    "Defend_Blue": "Defend_B",
    "Defend_Green": "Defend_G",
    "Defend_Red": "Defend_R",
    "Defend_Watcher": "Defend_P",
    "Equilibrium": "Undo",
    "Fasting": "Fasting2",
    "Nightmare": "Night Terror",
    "Overclock": "Steam Power",
    "PressurePoints": "PathToVictory",
    "Recursion": "Redo",
    "SimmeringFury": "Vengeance",
    "SneakyStrike": "Underhanded Strike",
    "SteamBarrier": "Steam",
    "Strike_Blue": "Strike_B",
    "Strike_Green": "Strike_G",
    "Strike_Purple": "Strike_P",
    "Strike_Red": "Strike_R",
    "Tranquility": "ClearTheMind",
    "VoidCard": "Void",
}


def test_java_class_ids_resolve_to_engine_card_ids() -> None:
    for java_id, canonical_id in JAVA_CLASS_ID_ALIASES.items():
        assert resolve_card_id(java_id) == canonical_id


def test_java_class_ids_are_loadable_via_get_card() -> None:
    for java_id, canonical_id in JAVA_CLASS_ID_ALIASES.items():
        card = get_card(java_id)
        assert card.id == canonical_id
