"""Audit §D10/D11 regression tests -- assert that the run_parser
reconciliation pass produces per-combat entry decks that match the
recorded master_deck composition, including Pandora's Box transform
outputs (both surviving and later-purged) and untracked relic upgrades.
"""

from __future__ import annotations

from pathlib import Path

import pytest

from packages.training.run_parser import parse_run_file


GOLDEN_RUN_PATH = Path(
    "/Users/jackswitzer/Library/Application Support/Steam/steamapps/"
    "common/SlayTheSpire/SlayTheSpire.app/Contents/Resources/runs/"
    "WATCHER/1776347657.run"
)


@pytest.fixture(scope="module")
def golden_run():
    if not GOLDEN_RUN_PATH.exists():
        pytest.skip(f"golden run not available at {GOLDEN_RUN_PATH}")
    return parse_run_file(GOLDEN_RUN_PATH)


def _combat_at(run, floor: int):
    for c in run.combat_cases:
        if c.floor == floor:
            return c
    raise AssertionError(f"no combat case at F{floor}")


def test_adaptation_is_upgraded_in_combats_after_pickup(golden_run):
    """Adaptation was picked at F4 (NoteForYourself) and ends up as
    `Adaptation+1` in master_deck. Reconciliation should upgrade it in
    every combat case from F5 onward (the next combat after it was added)."""
    for case in golden_run.combat_cases:
        if case.floor < 5:
            continue
        assert "Adaptation" not in case.entry_deck, (
            f"F{case.floor} entry deck still has un-upgraded Adaptation; "
            f"reconciliation should have upgraded it. Deck: {case.entry_deck}"
        )


def test_pandora_outputs_in_post_pickup_combats(golden_run):
    """Pandora's Box at F16 transformed leftover Strike_P + Defend_P into
    2 random commons. The first becomes Establishment (which survives to
    master_deck); the second becomes WaveOfTheHand (purged at F21).

    Combats F18..F20 should have BOTH cards in entry deck.
    Combats F22+ should have only Establishment (WaveOfTheHand purged).
    """
    f18 = _combat_at(golden_run, 18)
    assert "Establishment" in f18.entry_deck, (
        f"F18 missing Establishment (Pandora output); deck: {f18.entry_deck}"
    )
    assert "WaveOfTheHand" in f18.entry_deck, (
        f"F18 missing WaveOfTheHand (transient Pandora output); "
        f"deck: {f18.entry_deck}"
    )

    f22 = _combat_at(golden_run, 22)
    assert "Establishment" in f22.entry_deck, (
        f"F22 missing Establishment (Pandora output); deck: {f22.entry_deck}"
    )
    assert "WaveOfTheHand" not in f22.entry_deck, (
        f"F22 still has WaveOfTheHand even though F21 purged it; "
        f"deck: {f22.entry_deck}"
    )


def test_strike_and_defend_removed_post_pandora(golden_run):
    """Pre-Pandora combats keep their basic Strike/Defend; post-Pandora
    combats should not (Pandora transforms ALL remaining basics)."""
    f5 = _combat_at(golden_run, 5)
    # F5 Jaw Worm is pre-Pandora; should still have basics from starter.
    assert (
        "Strike" in f5.entry_deck or "Defend" in f5.entry_deck
    ), f"F5 (pre-Pandora) should still have starter basics; deck: {f5.entry_deck}"

    # F18+ Acts 2-4 combats should have NO Strike_P or Defend_P (Pandora
    # transformed the leftover basics at F16). NOTE: this assertion is
    # specific to the golden run's exact Neow + shop state where exactly
    # 1 Strike + 1 Defend remained at F16. If the assumption breaks, the
    # reconciliation diff was misapplied.
    for floor in (18, 22, 33, 50, 55):
        case = _combat_at(golden_run, floor)
        assert "Strike" not in case.entry_deck, (
            f"F{floor} (post-Pandora) still has Strike_P; "
            f"reconciliation should have removed it. Deck: {case.entry_deck}"
        )
        assert "Defend" not in case.entry_deck, (
            f"F{floor} (post-Pandora) still has Defend_P; "
            f"reconciliation should have removed it. Deck: {case.entry_deck}"
        )


def test_failed_removes_tracked(golden_run):
    """The forward-sim should have hit at least one failed remove on
    this run (WaveOfTheHand purge at F21 against a deck that doesn't
    contain it because Pandora outputs are not tracked in forward-sim)."""
    failed = [name for _, name in golden_run.failed_removes]
    assert "WaveOfTheHand" in failed, (
        f"expected at least one failed remove for WaveOfTheHand; "
        f"actual failed_removes: {golden_run.failed_removes}"
    )


def test_final_combat_uses_full_developed_deck(golden_run):
    """F55 (The Heart) entry deck should match the recorded master_deck size
    exactly (11 cards on this run) -- a developed late-game deck, not a
    near-starter approximation."""
    f55 = _combat_at(golden_run, 55)
    assert len(f55.entry_deck) == len(golden_run.final_master_deck), (
        f"F55 entry deck has {len(f55.entry_deck)} cards but recorded "
        f"master_deck has {len(golden_run.final_master_deck)}. Deck: "
        f"{f55.entry_deck}"
    )
    # Every Watcher pick should be in there (upgraded where the player
    # campfire-upgraded it).
    must_include = (
        "Eruption+",
        "Vigilance+",
        "Adaptation+",
        "Scrawl+",
        "MentalFortress+",
        "Vault+",
        "FearNoEvil+",
        "Meditate+",
        "Master of Strategy+",
        "Establishment",
        "WreathOfFlame",
    )
    missing = [c for c in must_include if c not in f55.entry_deck]
    assert not missing, (
        f"F55 entry deck missing {missing}. Full deck: {f55.entry_deck}"
    )
