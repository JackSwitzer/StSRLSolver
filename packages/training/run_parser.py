"""Parse Slay the Spire `.run` JSON files into combat-by-combat replay cases.

The Steam game writes one `.run` file per completed run under
`saves/runs/<character>/<timestamp>.run`. Each file is a single-line JSON
record containing per-floor arrays plus event-keyed lists (card_choices,
relics_obtained, event_choices, campfire_choices, potions_obtained,
items_purchased, items_purged, boss_relics, etc.).

This module reads such a file and produces:

- `RecordedRun` -- the parsed metadata plus a per-floor view that joins all
  the event lists onto their floor numbers.
- `RecordedCombatCase` -- one entry per combat encounter, with a
  forward-simulated entry deck/relics/potions snapshot, ready for the engine
  replayer to load.

Forward simulation is best-effort: where the `.run` schema is ambiguous (Neow
removes, shop item type discrimination, mid-combat potion timing), this module
applies a documented heuristic and emits a warning rather than failing. The
final reconstructed deck is sanity-checked against `master_deck`; mismatches
are surfaced via `RecordedRun.reconstruction_warnings`.
"""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from .entity_catalog import (
    canonicalize_potion_id,
    canonicalize_relic_id,
    canonicalize_watcher_card_id,
)


_WATCHER_RUN_STARTER_DECK: tuple[str, ...] = (
    "Strike",
    "Strike",
    "Strike",
    "Strike",
    "Defend",
    "Defend",
    "Defend",
    "Defend",
    "Eruption",
    "Vigilance",
)


def _try_canonical_card(value: str) -> str | None:
    try:
        return canonicalize_watcher_card_id(value)
    except KeyError:
        return None


def _try_canonical_relic(value: str) -> str | None:
    try:
        return canonicalize_relic_id(value)
    except KeyError:
        return None


def _try_canonical_potion(value: str) -> str | None:
    try:
        return canonicalize_potion_id(value)
    except KeyError:
        return None


@dataclass(frozen=True)
class RecordedFloor:
    """All data the `.run` schema records for a single floor, joined by floor number."""

    floor: int
    room_kind: str
    current_hp_post: int
    max_hp_post: int
    gold_post: int
    encounter: str | None = None
    damage_taken: int = 0
    turns: int | None = None
    card_picked: str | None = None
    relic_picked: str | None = None
    cards_obtained_event: tuple[str, ...] = ()
    cards_removed_event: tuple[str, ...] = ()
    cards_transformed_event: tuple[str, ...] = ()
    campfire_kind: str | None = None
    campfire_target: str | None = None
    items_purchased_here: tuple[str, ...] = ()
    items_purged_here: tuple[str, ...] = ()
    potions_obtained_here: tuple[str, ...] = ()
    boss_relic_picked: str | None = None


@dataclass(frozen=True)
class RecordedCombatCase:
    """One reconstructed combat encounter, ready for engine replay."""

    play_id: str
    floor: int
    encounter: str
    room_kind: str
    entry_hp: int
    max_hp: int
    entry_deck: tuple[str, ...]
    entry_relics: tuple[str, ...]
    entry_potions: tuple[str, ...]
    recorded_damage_taken: int
    recorded_turns: int | None


@dataclass
class RecordedRun:
    """Parsed `.run` file: metadata, per-floor records, and reconstruction notes."""

    play_id: str
    character: str
    seed_played: str
    ascension_level: int
    victory: bool
    floor_reached: int
    chose_seed: bool
    neow_bonus: str | None
    neow_cost: str | None
    starting_max_hp: int
    floors: tuple[RecordedFloor, ...]
    final_master_deck: tuple[str, ...]
    final_relics: tuple[str, ...]
    reconstruction_warnings: list[str] = field(default_factory=list)
    # (floor_num, card_name) pairs where a remove was attempted but the card
    # was not in the reconstructed deck. Often Pandora-output-then-purged
    # cases (e.g. WaveOfTheHand on the WATCHER A0 winning seed). The
    # reconciliation pass injects these cards into combat cases between the
    # Pandora pickup floor and the failed-remove floor.
    failed_removes: list[tuple[int, str]] = field(default_factory=list)
    combat_cases: tuple[RecordedCombatCase, ...] = ()


def parse_run_file(path: Path | str) -> RecordedRun:
    """Read a `.run` JSON file and produce a structured `RecordedRun`.

    Combat cases are reconstructed in `reconstruct_combat_cases` and stored on
    the returned object as `.combat_cases`.
    """

    raw = json.loads(Path(path).read_text())

    floor_reached = int(raw["floor_reached"])
    current_hp_per_floor: list[int] = list(raw.get("current_hp_per_floor", []))
    max_hp_per_floor: list[int] = list(raw.get("max_hp_per_floor", []))
    gold_per_floor: list[int] = list(raw.get("gold_per_floor", []))
    path_per_floor: list[str | None] = list(raw.get("path_per_floor", []))

    damage_by_floor = {int(d["floor"]): d for d in raw.get("damage_taken", [])}
    cards_by_floor = {int(c["floor"]): c for c in raw.get("card_choices", [])}
    relics_by_floor = {int(r["floor"]): str(r["key"]) for r in raw.get("relics_obtained", [])}
    events_by_floor = {int(e["floor"]): e for e in raw.get("event_choices", [])}
    campfire_by_floor = {int(c["floor"]): c for c in raw.get("campfire_choices", [])}

    potions_obtained_by_floor: dict[int, list[str]] = {}
    for entry in raw.get("potions_obtained", []):
        potions_obtained_by_floor.setdefault(int(entry["floor"]), []).append(str(entry["key"]))

    purchased_by_floor: dict[int, list[str]] = {}
    for floor_num, item in zip(
        raw.get("item_purchase_floors", []),
        raw.get("items_purchased", []),
        strict=False,
    ):
        purchased_by_floor.setdefault(int(floor_num), []).append(str(item))

    purged_by_floor: dict[int, list[str]] = {}
    for floor_num, item in zip(
        raw.get("items_purged_floors", []),
        raw.get("items_purged", []),
        strict=False,
    ):
        purged_by_floor.setdefault(int(floor_num), []).append(str(item))

    # `boss_relics` is an ordered list, one per boss floor visited (16, 33, 50).
    # The .run schema does not tag them with the floor; we infer by boss-floor
    # ordering from path_per_floor.
    boss_relic_choices: list[dict[str, Any]] = list(raw.get("boss_relics", []))
    boss_floors_visited = [
        idx + 1
        for idx, room in enumerate(path_per_floor)
        if isinstance(room, str) and room == "B"
    ]
    boss_relic_by_floor: dict[int, str] = {}
    for floor_num, choice in zip(boss_floors_visited, boss_relic_choices, strict=False):
        if isinstance(choice, dict) and choice.get("picked") and choice["picked"] != "SKIP":
            boss_relic_by_floor[floor_num] = str(choice["picked"])

    floors: list[RecordedFloor] = []
    for idx in range(floor_reached):
        floor_num = idx + 1
        room = path_per_floor[idx] if idx < len(path_per_floor) else None
        room_kind = room if isinstance(room, str) else "post-boss"

        current_hp = current_hp_per_floor[idx] if idx < len(current_hp_per_floor) else 0
        max_hp = max_hp_per_floor[idx] if idx < len(max_hp_per_floor) else 0
        gold = gold_per_floor[idx] if idx < len(gold_per_floor) else 0

        damage = damage_by_floor.get(floor_num, {})
        card_pick_record = cards_by_floor.get(floor_num, {})
        picked_card_raw = card_pick_record.get("picked")
        picked_card = (
            str(picked_card_raw)
            if picked_card_raw and picked_card_raw != "SKIP"
            else None
        )
        event_record = events_by_floor.get(floor_num, {})
        campfire = campfire_by_floor.get(floor_num, {})

        floors.append(
            RecordedFloor(
                floor=floor_num,
                room_kind=room_kind,
                current_hp_post=current_hp,
                max_hp_post=max_hp,
                gold_post=gold,
                encounter=damage.get("enemies"),
                damage_taken=int(damage.get("damage", 0)),
                turns=damage.get("turns"),
                card_picked=picked_card,
                relic_picked=relics_by_floor.get(floor_num),
                cards_obtained_event=tuple(str(c) for c in event_record.get("cards_obtained", [])),
                cards_removed_event=tuple(str(c) for c in event_record.get("cards_removed", [])),
                cards_transformed_event=tuple(str(c) for c in event_record.get("cards_transformed", [])),
                campfire_kind=campfire.get("key"),
                campfire_target=campfire.get("data"),
                items_purchased_here=tuple(purchased_by_floor.get(floor_num, ())),
                items_purged_here=tuple(purged_by_floor.get(floor_num, ())),
                potions_obtained_here=tuple(potions_obtained_by_floor.get(floor_num, ())),
                boss_relic_picked=boss_relic_by_floor.get(floor_num),
            )
        )

    starting_max_hp = max_hp_per_floor[0] if max_hp_per_floor else 72

    run = RecordedRun(
        play_id=str(raw.get("play_id", "")),
        character=str(raw.get("character_chosen", "")),
        seed_played=str(raw.get("seed_played", "")),
        ascension_level=int(raw.get("ascension_level", 0)),
        victory=bool(raw.get("victory", False)),
        floor_reached=floor_reached,
        chose_seed=bool(raw.get("chose_seed", False)),
        neow_bonus=raw.get("neow_bonus"),
        neow_cost=raw.get("neow_cost"),
        starting_max_hp=starting_max_hp,
        floors=tuple(floors),
        final_master_deck=tuple(str(c) for c in raw.get("master_deck", ())),
        final_relics=tuple(str(r) for r in raw.get("relics", ())),
    )

    run.combat_cases = reconstruct_combat_cases(run)
    return run


def reconstruct_combat_cases(run: RecordedRun) -> tuple[RecordedCombatCase, ...]:
    """Forward-simulate deck/relics/potions per floor, emit one case per combat.

    The pre-combat snapshot is taken at the start of each combat floor, before
    any rewards from that floor are applied. Sanity check at the end: the
    reconstructed master deck is compared to `run.final_master_deck` and any
    mismatch is logged to `run.reconstruction_warnings`.
    """

    if run.character != "WATCHER":
        raise ValueError(
            f"run_parser currently supports WATCHER only; got {run.character}"
        )

    deck: list[str] = list(_WATCHER_RUN_STARTER_DECK)
    relics: list[str] = ["PureWater"]
    potions: list[str] = []
    potion_slots = 3

    _apply_neow(deck, relics, run)

    # TODO: TEN_PERCENT_HP_LOSS Neow cost should drop entry HP for floor 1 from
    # full HP to 0.9 * max. Currently leaves it stale; floor 1 entry_hp matches
    # current_hp_post on floor 1 in practice so the error is bounded to one floor.
    current_hp = run.starting_max_hp
    max_hp = run.starting_max_hp

    cases: list[RecordedCombatCase] = []

    for floor in run.floors:
        if floor.encounter is not None:
            cases.append(
                RecordedCombatCase(
                    play_id=run.play_id,
                    floor=floor.floor,
                    encounter=floor.encounter,
                    room_kind=floor.room_kind,
                    entry_hp=current_hp,
                    max_hp=max_hp,
                    entry_deck=tuple(deck),
                    entry_relics=tuple(relics),
                    entry_potions=tuple(potions),
                    recorded_damage_taken=floor.damage_taken,
                    recorded_turns=floor.turns,
                )
            )

        # Apply post-floor changes.
        if floor.card_picked:
            _add_card(deck, floor.card_picked, run.reconstruction_warnings)
        for c in floor.cards_obtained_event:
            _add_card(deck, c, run.reconstruction_warnings)
        for c in floor.cards_removed_event:
            _remove_card(deck, c, run.reconstruction_warnings)
        for c in floor.cards_transformed_event:
            _remove_card(deck, c, run.reconstruction_warnings)

        if floor.campfire_kind == "SMITH" and floor.campfire_target:
            _upgrade_card(deck, floor.campfire_target, run.reconstruction_warnings)

        for picked in (floor.relic_picked, floor.boss_relic_picked):
            if not picked:
                continue
            _add_relic(relics, picked, run.reconstruction_warnings)
            if _try_canonical_relic(picked) == "PotionBelt":
                potion_slots = 5

        for item in floor.items_purchased_here:
            _apply_shop_purchase(item, deck, relics, potions, potion_slots, run.reconstruction_warnings)

        for item in floor.items_purged_here:
            if not _remove_card(deck, item, run.reconstruction_warnings):
                # Track for reconciliation: this card was probably added by an
                # untracked source (Pandora's Box, an event we don't simulate)
                # and needs to be re-injected into combats between that source
                # and this purge floor.
                run.failed_removes.append((floor.floor, item))

        for potion in floor.potions_obtained_here:
            _gain_potion(potions, potion, slots=potion_slots, warnings=run.reconstruction_warnings)

        current_hp = floor.current_hp_post
        max_hp = floor.max_hp_post

    _validate_final_deck(deck, run.final_master_deck, run.reconstruction_warnings)
    _validate_final_relics(relics, run.final_relics, run.reconstruction_warnings)

    cases_tuple = _reconcile_with_master_deck(
        tuple(cases),
        reconstructed_final_deck=tuple(deck),
        recorded_final_deck=run.final_master_deck,
        warnings=run.reconstruction_warnings,
        failed_removes=tuple(run.failed_removes),
    )

    return cases_tuple


def _reconcile_with_master_deck(
    cases: tuple["RecordedCombatCase", ...],
    *,
    reconstructed_final_deck: tuple[str, ...],
    recorded_final_deck: tuple[str, ...],
    warnings: list[str],
    failed_removes: tuple[tuple[int, str], ...] = (),
) -> tuple["RecordedCombatCase", ...]:
    """Patch per-combat entry decks so they match the recorded master_deck.

    The forward-sim cannot reproduce Pandora's Box random transforms or
    other relic-driven upgrades whose outputs the `.run` schema does not
    record. After forward-sim we know exactly which cards are extra in
    our reconstruction (should be removed) and which are missing (should
    be added). This pass:

    1. Pairs upgrade mismatches first (`X` extra, `X+` missing) and applies
       the upgrade to every combat case that has the un-upgraded card in
       its entry deck.
    2. Treats remaining basics (`Strike_P`/`Defend_P`) as Pandora's Box
       leftovers and removes them from combats AFTER the typical F16 boss
       pickup. Treats remaining missing cards as Pandora's transform
       outputs and adds them to combats from F17 onward.

    The result is that combats from Act 2+ use the player's actual late
    game deck composition rather than a near-starter approximation. Earlier
    combats (Act 1 pre-boss) are untouched.
    """
    from dataclasses import replace

    if not recorded_final_deck:
        return cases

    canon_recorded = sorted(_normalize_upgrade_suffix(c) for c in recorded_final_deck)
    canon_reconstructed = sorted(_normalize_upgrade_suffix(c) for c in reconstructed_final_deck)
    if canon_recorded == canon_reconstructed:
        return cases

    # Multiset-style diffs.
    from collections import Counter

    expected = Counter(canon_recorded)
    actual = Counter(canon_reconstructed)
    extra = list((actual - expected).elements())
    missing = list((expected - actual).elements())

    out: list = list(cases)

    # Step 1: pair upgrade mismatches.
    pandora_floor_default = 16
    upgraded_pairs: list[tuple[str, str]] = []
    for unu in list(extra):
        upg = f"{unu}+"
        if upg in missing:
            extra.remove(unu)
            missing.remove(upg)
            upgraded_pairs.append((unu, upg))

    for unu, upg in upgraded_pairs:
        for i, case in enumerate(out):
            if unu in case.entry_deck:
                new_deck = list(case.entry_deck)
                idx = new_deck.index(unu)
                new_deck[idx] = upg
                out[i] = replace(case, entry_deck=tuple(new_deck))
        warnings.append(
            f"reconciled upgrade {unu!r} -> {upg!r} (likely relic-driven, "
            f"e.g. Frozen Eye / Astrolabe / Apotheosis); applied to all "
            f"combats containing {unu!r}"
        )

    # Step 2: remaining diffs are Pandora-style transforms (or other
    # untracked relic effects). Apply at the boss floor where Pandora is
    # canonically picked.
    for c in extra:
        # Remove ONE instance of `c` from EVERY post-Pandora combat (the
        # forward-sim propagated this card into every snapshot). If a combat
        # already lacks the card, it is silently skipped.
        for i, case in enumerate(out):
            if case.floor > pandora_floor_default and c in case.entry_deck:
                new_deck = list(case.entry_deck)
                new_deck.remove(c)
                out[i] = replace(case, entry_deck=tuple(new_deck))
        warnings.append(
            f"reconciled removal of {c!r} from combats after F{pandora_floor_default} "
            f"(likely Pandora's Box transform)"
        )

    for c in missing:
        # Add to entry_deck of combats AFTER the pandora floor.
        for i, case in enumerate(out):
            if case.floor > pandora_floor_default:
                out[i] = replace(case, entry_deck=case.entry_deck + (c,))
        warnings.append(
            f"reconciled addition of {c!r} to combats after F{pandora_floor_default} "
            f"(likely Pandora's Box output)"
        )

    # Step 3: Pandora-output-then-purged. Each (purge_floor, card) pair means
    # the player had `card` in their deck between the Pandora pickup floor
    # (default F16) and `purge_floor`. Inject `card` into combats in that
    # window so the bot plays those combats with the same deck the player had.
    for purge_floor, raw_name in failed_removes:
        canon = _try_canonical_card(raw_name) or raw_name
        if purge_floor <= pandora_floor_default:
            continue  # purged before Pandora; would be a different scenario
        injected_floors: list[int] = []
        for i, case in enumerate(out):
            if pandora_floor_default < case.floor < purge_floor:
                out[i] = replace(case, entry_deck=case.entry_deck + (canon,))
                injected_floors.append(case.floor)
        if injected_floors:
            warnings.append(
                f"reconciled transient {canon!r}: injected into combats "
                f"F{injected_floors[0]}..F{injected_floors[-1]} "
                f"(Pandora-output picked at F{pandora_floor_default}, "
                f"purged at F{purge_floor})"
            )

    return tuple(out)


def _apply_neow(deck: list[str], relics: list[str], run: RecordedRun) -> None:
    """Apply the documented Neow bonus to the starting deck/relics."""

    bonus = run.neow_bonus
    if bonus is None:
        return

    if bonus == "REMOVE_TWO":
        # Remove 1 Strike + 1 Defend (Baalorlord-pattern default).
        # The exact pair is not recorded in `.run`; the final-deck sanity check
        # will surface a mismatch if a different pair was actually removed.
        if "Strike" in deck:
            deck.remove("Strike")
        if "Defend" in deck:
            deck.remove("Defend")
        run.reconstruction_warnings.append(
            "neow REMOVE_TWO defaulted to 1× Strike_P + 1× Defend_P; "
            "verify against final master_deck"
        )
    elif bonus == "ONE_RANDOM_RARE_CARD":
        # The random card lands in `master_deck` but isn't recorded as a pick;
        # we cannot reconstruct without diffing — skip and let the validator flag.
        run.reconstruction_warnings.append("neow ONE_RANDOM_RARE_CARD not reconstructed")
    elif bonus == "BOSS_RELIC":
        # Adds an extra boss relic at start and removes starter relic. Schema
        # records starter relic loss implicitly; surface a warning.
        run.reconstruction_warnings.append("neow BOSS_RELIC not reconstructed")
    # Other Neow bonuses (HUNDRED_GOLD, MAX_HP, REMOVE_CARD, etc.) do not affect deck.


def _add_card(deck: list[str], raw_name: str, warnings: list[str]) -> None:
    canon = _try_canonical_card(raw_name)
    if canon is None:
        warnings.append(f"unknown card on add: {raw_name!r}")
        deck.append(raw_name)
        return
    deck.append(canon)


def _remove_card(deck: list[str], raw_name: str, warnings: list[str]) -> bool:
    """Remove `raw_name` from deck (trying canonical/base/upgraded variants).

    Returns True on success, False if the card was not present (in which
    case a warning is logged). Callers that need to handle failure
    distinctly (e.g. shop purges of Pandora outputs whose original add was
    not simulated) can branch on the return value.
    """
    canon = _try_canonical_card(raw_name) or raw_name
    base = canon.removesuffix("+")
    upgraded = f"{base}+"
    for candidate in (canon, base, upgraded):
        if candidate in deck:
            deck.remove(candidate)
            return True
    warnings.append(f"remove failed (not in deck): {raw_name!r}")
    return False


def _upgrade_card(deck: list[str], raw_name: str, warnings: list[str]) -> None:
    canon = _try_canonical_card(raw_name) or raw_name
    base = canon.removesuffix("+")
    upgraded = f"{base}+"
    for i, deck_card in enumerate(deck):
        if deck_card == base:
            deck[i] = upgraded
            return
    warnings.append(f"upgrade failed (base not in deck): {raw_name!r}")


def _add_relic(relics: list[str], raw_name: str, warnings: list[str]) -> None:
    canon = _try_canonical_relic(raw_name)
    if canon is None:
        warnings.append(f"unknown relic on add: {raw_name!r}")
        if raw_name not in relics:
            relics.append(raw_name)
        return
    if canon not in relics:
        relics.append(canon)


def _gain_potion(
    potions: list[str], raw_name: str, *, slots: int, warnings: list[str]
) -> None:
    canon = _try_canonical_potion(raw_name)
    if canon is None:
        warnings.append(f"unknown potion on gain: {raw_name!r}")
        canon = raw_name
    if len(potions) < slots:
        potions.append(canon)


def _apply_shop_purchase(
    item: str,
    deck: list[str],
    relics: list[str],
    potions: list[str],
    potion_slots: int,
    warnings: list[str],
) -> None:
    """Discriminate a shop purchase between card/relic/potion by trying each catalog."""

    relic = _try_canonical_relic(item)
    if relic is not None:
        if relic not in relics:
            relics.append(relic)
        return
    potion = _try_canonical_potion(item)
    if potion is not None:
        if len(potions) < potion_slots:
            potions.append(potion)
        return
    card = _try_canonical_card(item)
    if card is not None:
        deck.append(card)
        return
    warnings.append(f"unknown shop item (skipped): {item!r}")


def _normalize_upgrade_suffix(card: str) -> str:
    """Normalize `.run` upgrade suffixes (`+1`, `+2`, ...) to engine `+` form.

    Watcher cards never upgrade past `+`, so any numeric tail collapses to `+`.
    Searing Blow (Ironclad) is the only multi-upgrade card; we don't expect it
    here.
    """
    if "+" not in card:
        return card
    base, _, tail = card.partition("+")
    return f"{base}+" if (tail == "" or tail.isdigit()) else card


def _validate_final_deck(
    reconstructed: list[str], expected: tuple[str, ...], warnings: list[str]
) -> None:
    if not expected:
        return
    canon_expected = []
    for c in expected:
        normalized = _normalize_upgrade_suffix(c)
        canon = _try_canonical_card(normalized) or normalized
        canon_expected.append(canon)
    canon_reconstructed = sorted(_normalize_upgrade_suffix(c) for c in reconstructed)
    canon_expected_sorted = sorted(canon_expected)
    if canon_reconstructed == canon_expected_sorted:
        return
    only_in_recon = sorted(set(canon_reconstructed) - set(canon_expected_sorted))
    only_in_expected = sorted(set(canon_expected_sorted) - set(canon_reconstructed))
    warnings.append(
        f"final deck mismatch: reconstructed={len(canon_reconstructed)}, "
        f"expected={len(canon_expected_sorted)}, "
        f"only_in_reconstructed={only_in_recon[:8]}, "
        f"only_in_expected={only_in_expected[:8]}"
    )


def _validate_final_relics(
    reconstructed: list[str], expected: tuple[str, ...], warnings: list[str]
) -> None:
    if not expected:
        return
    canon_expected = []
    for r in expected:
        canon = _try_canonical_relic(r) or r
        canon_expected.append(canon)
    if sorted(reconstructed) == sorted(canon_expected):
        return
    only_in_recon = sorted(set(reconstructed) - set(canon_expected))
    only_in_expected = sorted(set(canon_expected) - set(reconstructed))
    warnings.append(
        f"final relics mismatch: only_in_reconstructed={only_in_recon}, "
        f"only_in_expected={only_in_expected}"
    )


__all__ = [
    "RecordedFloor",
    "RecordedCombatCase",
    "RecordedRun",
    "parse_run_file",
    "reconstruct_combat_cases",
]


def _print_summary(run: RecordedRun) -> None:
    print(f"play_id={run.play_id}")
    print(f"character={run.character}  ascension={run.ascension_level}  victory={run.victory}")
    print(f"seed_played={run.seed_played}  chose_seed={run.chose_seed}")
    print(f"floor_reached={run.floor_reached}  starting_max_hp={run.starting_max_hp}")
    print(f"neow_bonus={run.neow_bonus}  neow_cost={run.neow_cost}")
    print(f"final_master_deck size={len(run.final_master_deck)}")
    print(f"final_relics size={len(run.final_relics)}")
    print(f"combats reconstructed: {len(run.combat_cases)}")
    print()
    print("Combats:")
    for case in run.combat_cases:
        print(
            f"  F{case.floor:2d} [{case.room_kind}] {case.encounter:30s} "
            f"hp={case.entry_hp:3d}/{case.max_hp:3d}  "
            f"deck={len(case.entry_deck):2d}  relics={len(case.entry_relics):2d}  "
            f"potions={len(case.entry_potions):1d}  "
            f"recorded_dmg={case.recorded_damage_taken}"
        )
    if run.reconstruction_warnings:
        print()
        print(f"Warnings ({len(run.reconstruction_warnings)}):")
        for w in run.reconstruction_warnings:
            print(f"  - {w}")


if __name__ == "__main__":
    import sys

    if len(sys.argv) != 2:
        print("usage: python -m packages.training.run_parser <path-to-.run>")
        sys.exit(2)
    parsed = parse_run_file(sys.argv[1])
    _print_summary(parsed)
