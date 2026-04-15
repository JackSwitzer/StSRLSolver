"""Curated combat corpus planning for the Watcher A0 first milestone."""

from __future__ import annotations

from collections import Counter
from dataclasses import asdict, dataclass, field
from typing import Any, Mapping

from .combat_model import CombatStateSummary, LegalCombatCandidate
from .shared_memory import CombatSearchRequest


PHASE1_CORPUS_NAME = "watcher_a0_act1"
PHASE1_CORPUS_TARGET_CASES = 50_000


WATCHER_STARTER_DECK = (
    "Strike_P",
    "Strike_P",
    "Strike_P",
    "Strike_P",
    "Defend_P",
    "Defend_P",
    "Defend_P",
    "Defend_P",
    "Eruption",
    "Vigilance",
    "Miracle",
)


@dataclass(frozen=True)
class SeedProvenance:
    source: str
    label: str
    seed: int | None = None
    bucket: str | None = None
    notes: tuple[str, ...] = ()


@dataclass(frozen=True)
class NeowProvenance:
    source: str
    policy: str
    choice_index: int | None = None
    option_label: str | None = None
    all_options_available: bool = True
    notes: tuple[str, ...] = ()


@dataclass(frozen=True)
class DeckProvenance:
    family: str
    description: str
    base_deck: tuple[str, ...] = WATCHER_STARTER_DECK
    removed_cards: tuple[str, ...] = ()
    added_cards: tuple[str, ...] = ()
    upgraded_cards: tuple[str, ...] = ()
    potion_set: tuple[str, ...] = ()
    tags: tuple[str, ...] = ()

    @property
    def remove_count(self) -> int:
        return len(self.removed_cards)


@dataclass(frozen=True)
class CorpusFamilyPlan:
    family: str
    description: str
    deck: DeckProvenance
    seed_provenance: SeedProvenance
    neow_provenance: NeowProvenance
    focus_enemies: tuple[str, ...] = ()
    tags: tuple[str, ...] = ()


@dataclass(frozen=True)
class CorpusCasePlan:
    case_id: str
    description: str
    floor: int
    enemy: str
    deck: DeckProvenance
    seed_provenance: SeedProvenance
    neow_provenance: NeowProvenance
    potion_set: tuple[str, ...] = ()
    tags: tuple[str, ...] = ()

    @property
    def deck_family(self) -> str:
        return self.deck.family

    @property
    def remove_count(self) -> int:
        return self.deck.remove_count


@dataclass(frozen=True)
class BenchmarkSlicePlan:
    name: str
    description: str
    includes_opening_hand_enumeration: bool = False
    includes_potion_variants: bool = False
    includes_setup_counter_variants: bool = False
    family_names: tuple[str, ...] = ()
    cases: tuple[CorpusCasePlan, ...] = ()


@dataclass(frozen=True)
class CorpusPlan:
    character: str
    ascension: int
    families: tuple[CorpusFamilyPlan, ...] = field(default_factory=tuple)
    slices: tuple[BenchmarkSlicePlan, ...] = field(default_factory=tuple)


@dataclass(frozen=True)
class PreparedCorpusRequest:
    corpus_index: int
    corpus_id: str
    corpus_group: str
    collection_pass: int
    slice_name: str
    case: CorpusCasePlan
    variant_index: int
    opening_hand_bucket: str
    request: CombatSearchRequest
    preferred_action_id: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "corpus_index": self.corpus_index,
            "corpus_id": self.corpus_id,
            "corpus_group": self.corpus_group,
            "collection_pass": self.collection_pass,
            "slice_name": self.slice_name,
            "case": _case_to_dict(self.case),
            "variant_index": self.variant_index,
            "opening_hand_bucket": self.opening_hand_bucket,
            "request": self.request.to_dict(),
            "preferred_action_id": self.preferred_action_id,
        }

    @classmethod
    def from_dict(cls, payload: Mapping[str, Any]) -> "PreparedCorpusRequest":
        return cls(
            corpus_index=int(payload["corpus_index"]),
            corpus_id=str(payload["corpus_id"]),
            corpus_group=str(payload["corpus_group"]),
            collection_pass=int(payload["collection_pass"]),
            slice_name=str(payload["slice_name"]),
            case=_case_from_dict(payload["case"]),
            variant_index=int(payload["variant_index"]),
            opening_hand_bucket=str(payload["opening_hand_bucket"]),
            request=CombatSearchRequest.from_dict(payload["request"]),
            preferred_action_id=str(payload["preferred_action_id"]),
        )


def _seed_to_dict(seed: SeedProvenance) -> dict[str, Any]:
    return asdict(seed)


def _neow_to_dict(neow: NeowProvenance) -> dict[str, Any]:
    return asdict(neow)


def _deck_to_dict(deck: DeckProvenance) -> dict[str, Any]:
    return asdict(deck)


def _case_to_dict(case: CorpusCasePlan) -> dict[str, Any]:
    return {
        "case_id": case.case_id,
        "description": case.description,
        "floor": case.floor,
        "enemy": case.enemy,
        "deck": _deck_to_dict(case.deck),
        "seed_provenance": _seed_to_dict(case.seed_provenance),
        "neow_provenance": _neow_to_dict(case.neow_provenance),
        "potion_set": list(case.potion_set),
        "tags": list(case.tags),
    }


def _case_from_dict(payload: Mapping[str, Any]) -> CorpusCasePlan:
    return CorpusCasePlan(
        case_id=str(payload["case_id"]),
        description=str(payload["description"]),
        floor=int(payload["floor"]),
        enemy=str(payload["enemy"]),
        deck=DeckProvenance(
            family=str(payload["deck"]["family"]),
            description=str(payload["deck"]["description"]),
            base_deck=tuple(payload["deck"].get("base_deck", WATCHER_STARTER_DECK)),
            removed_cards=tuple(payload["deck"].get("removed_cards", ())),
            added_cards=tuple(payload["deck"].get("added_cards", ())),
            upgraded_cards=tuple(payload["deck"].get("upgraded_cards", ())),
            potion_set=tuple(payload["deck"].get("potion_set", ())),
            tags=tuple(payload["deck"].get("tags", ())),
        ),
        seed_provenance=SeedProvenance(
            source=str(payload["seed_provenance"]["source"]),
            label=str(payload["seed_provenance"]["label"]),
            seed=payload["seed_provenance"].get("seed"),
            bucket=payload["seed_provenance"].get("bucket"),
            notes=tuple(payload["seed_provenance"].get("notes", ())),
        ),
        neow_provenance=NeowProvenance(
            source=str(payload["neow_provenance"]["source"]),
            policy=str(payload["neow_provenance"]["policy"]),
            choice_index=payload["neow_provenance"].get("choice_index"),
            option_label=payload["neow_provenance"].get("option_label"),
            all_options_available=bool(payload["neow_provenance"].get("all_options_available", True)),
            notes=tuple(payload["neow_provenance"].get("notes", ())),
        ),
        potion_set=tuple(payload.get("potion_set", ())),
        tags=tuple(payload.get("tags", ())),
    )


def _corpus_id(slice_name: str, case: CorpusCasePlan, corpus_index: int) -> str:
    return f"{PHASE1_CORPUS_NAME}::{slice_name}::{case.case_id}::{corpus_index:05d}"


def _corpus_group(slice_name: str, case: CorpusCasePlan, opening_hand_bucket: str) -> str:
    potion_label = "+".join(case.potion_set) if case.potion_set else "none"
    return "|".join(
        (
            slice_name,
            case.deck.family,
            case.enemy,
            f"remove={case.remove_count}",
            f"potion={potion_label}",
            f"bucket={opening_hand_bucket}",
        )
    )


def summarize_phase1_corpus_requests(requests: tuple[PreparedCorpusRequest, ...]) -> dict[str, Any]:
    slice_counts = Counter(request.slice_name for request in requests)
    family_counts = Counter(request.case.deck.family for request in requests)
    group_counts = Counter(request.corpus_group for request in requests)
    bucket_counts = Counter(request.opening_hand_bucket for request in requests)
    return {
        "corpus_name": PHASE1_CORPUS_NAME,
        "total_cases": len(requests),
        "unique_corpus_groups": len(group_counts),
        "slice_counts": dict(sorted(slice_counts.items())),
        "family_counts": dict(sorted(family_counts.items())),
        "group_counts": dict(sorted(group_counts.items())),
        "opening_hand_bucket_counts": dict(sorted(bucket_counts.items())),
        "collection_passes": max((request.collection_pass for request in requests), default=-1) + 1,
        "first_corpus_id": requests[0].corpus_id if requests else None,
        "last_corpus_id": requests[-1].corpus_id if requests else None,
    }


def build_phase1_corpus_requests(
    plan: CorpusPlan,
    *,
    total_cases: int = PHASE1_CORPUS_TARGET_CASES,
) -> tuple[PreparedCorpusRequest, ...]:
    return build_phase1_requests(plan, target_requests=total_cases)


def _watcher_easy_seed(label: str) -> SeedProvenance:
    return SeedProvenance(
        source="easy_seed_placeholder",
        label=label,
        bucket="watcher_a0_act1_easy_seed_pool",
        notes=("Replace with mined easy-seed inventory after overnight harvesting.",),
    )


def _watcher_neow(label: str, *, choice_index: int | None = None) -> NeowProvenance:
    return NeowProvenance(
        source="neow_placeholder",
        policy="always_four_choices",
        choice_index=choice_index,
        option_label=label,
        notes=("Training contract keeps the full four-option Neow surface available per seed.",),
    )


def default_watcher_a0_act1_corpus_plan() -> CorpusPlan:
    starter_family = CorpusFamilyPlan(
        family="starter-vanilla",
        description="Untouched Watcher starter deck for baseline hallway solves.",
        deck=DeckProvenance(
            family="starter-vanilla",
            description="Pure starter deck with no removals or additions.",
            tags=("synthetic", "starter", "watcher", "a0", "act1"),
        ),
        seed_provenance=_watcher_easy_seed("watcher_a0_act1_starter"),
        neow_provenance=_watcher_neow("placeholder_opening_bonus"),
        focus_enemies=("Jaw Worm", "Cultist"),
        tags=("baseline", "synthetic"),
    )
    remove_family = CorpusFamilyPlan(
        family="single-strike-remove",
        description="Single Strike remove to test slimmer Watcher act 1 lines.",
        deck=DeckProvenance(
            family="single-strike-remove",
            description="Starter deck with one Strike removed and a light premium add.",
            removed_cards=("Strike_P",),
            added_cards=("Cut Through Fate",),
            tags=("synthetic", "remove1", "watcher", "a0", "act1"),
        ),
        seed_provenance=_watcher_easy_seed("watcher_a0_act1_remove1"),
        neow_provenance=_watcher_neow("remove_card"),
        focus_enemies=("Gremlin Nob", "Lagavulin"),
        tags=("remove", "synthetic"),
    )
    calm_family = CorpusFamilyPlan(
        family="calm-setup-upgrade",
        description="Light setup/upgrade family for opening-hand and potion planning.",
        deck=DeckProvenance(
            family="calm-setup-upgrade",
            description="Starter deck with an upgraded Vigilance and one premium setup add.",
            added_cards=("Third Eye",),
            upgraded_cards=("Vigilance",),
            tags=("synthetic", "upgrade", "watcher", "a0", "act1"),
        ),
        seed_provenance=_watcher_easy_seed("watcher_a0_act1_calm_setup"),
        neow_provenance=_watcher_neow("upgrade_card"),
        focus_enemies=("Sentries", "Lagavulin", "Hexaghost"),
        tags=("setup", "synthetic"),
    )

    curated_core_cases = (
        CorpusCasePlan(
            case_id="jaw-worm-starter",
            description="Baseline hallway state for starter-deck Watcher.",
            floor=1,
            enemy="Jaw Worm",
            deck=starter_family.deck,
            seed_provenance=starter_family.seed_provenance,
            neow_provenance=starter_family.neow_provenance,
            potion_set=(),
            tags=("synthetic", "hallway", "baseline"),
        ),
        CorpusCasePlan(
            case_id="cultist-remove",
            description="Single-remove hallway case with a slightly cleaner deck.",
            floor=3,
            enemy="Cultist",
            deck=remove_family.deck,
            seed_provenance=remove_family.seed_provenance,
            neow_provenance=remove_family.neow_provenance,
            potion_set=(),
            tags=("synthetic", "hallway", "remove"),
        ),
        CorpusCasePlan(
            case_id="gremlin-nob-fire-potion",
            description="Elite pressure case with a deterministic fire potion loadout.",
            floor=7,
            enemy="Gremlin Nob",
            deck=remove_family.deck,
            seed_provenance=remove_family.seed_provenance,
            neow_provenance=remove_family.neow_provenance,
            potion_set=("Fire Potion",),
            tags=("synthetic", "elite", "potion"),
        ),
    )
    opening_hand_cases = (
        CorpusCasePlan(
            case_id="sentries-opening-hand-stance",
            description="Opening-hand enumeration for Sentries with stance-potion variance.",
            floor=7,
            enemy="Sentries",
            deck=calm_family.deck,
            seed_provenance=calm_family.seed_provenance,
            neow_provenance=calm_family.neow_provenance,
            potion_set=("Stance Potion",),
            tags=("synthetic", "opening-hand", "elite"),
        ),
        CorpusCasePlan(
            case_id="lagavulin-opening-hand-swift",
            description="Lagavulin opening-hand bucket with a Swift Potion placeholder.",
            floor=8,
            enemy="Lagavulin",
            deck=calm_family.deck,
            seed_provenance=calm_family.seed_provenance,
            neow_provenance=calm_family.neow_provenance,
            potion_set=("Swift Potion",),
            tags=("synthetic", "opening-hand", "elite"),
        ),
    )
    frontier_hard_cases = (
        CorpusCasePlan(
            case_id="lagavulin-hard-remove",
            description="Hard elite setup case used for disagreement and frontier harvesting.",
            floor=9,
            enemy="Lagavulin",
            deck=remove_family.deck,
            seed_provenance=remove_family.seed_provenance,
            neow_provenance=remove_family.neow_provenance,
            potion_set=("Fear Potion",),
            tags=("synthetic", "frontier", "hard"),
        ),
        CorpusCasePlan(
            case_id="hexaghost-calm-burst",
            description="Act 1 boss placeholder for overnight search frontier quality checks.",
            floor=17,
            enemy="Hexaghost",
            deck=calm_family.deck,
            seed_provenance=calm_family.seed_provenance,
            neow_provenance=calm_family.neow_provenance,
            potion_set=("Dexterity Potion", "Stance Potion"),
            tags=("synthetic", "boss", "frontier", "hard"),
        ),
    )

    return CorpusPlan(
        character="Watcher",
        ascension=0,
        families=(starter_family, remove_family, calm_family),
        slices=(
            BenchmarkSlicePlan(
                name="curated-core",
                description="Curated hallway and elite combat states for baseline solver quality.",
                includes_potion_variants=True,
                includes_setup_counter_variants=True,
                family_names=(starter_family.family, remove_family.family),
                cases=curated_core_cases,
            ),
            BenchmarkSlicePlan(
                name="opening-hand-buckets",
                description="Opening-hand enumeration from the same pre-draw combat state.",
                includes_opening_hand_enumeration=True,
                includes_potion_variants=True,
                family_names=(calm_family.family,),
                cases=opening_hand_cases,
            ),
            BenchmarkSlicePlan(
                name="frontier-harvest-hard",
                description="Hard states mined from search disagreement and high-entropy roots.",
                includes_potion_variants=True,
                includes_setup_counter_variants=True,
                family_names=(remove_family.family, calm_family.family),
                cases=frontier_hard_cases,
            ),
        ),
    )


def iter_slice_cases(plan: CorpusPlan) -> tuple[tuple[str, CorpusCasePlan], ...]:
    pairs: list[tuple[str, CorpusCasePlan]] = []
    for slice_plan in plan.slices:
        for case in slice_plan.cases:
            pairs.append((slice_plan.name, case))
    return tuple(pairs)


def build_phase1_requests(
    plan: CorpusPlan,
    *,
    target_requests: int = 5_000,
) -> tuple[PreparedCorpusRequest, ...]:
    if target_requests <= 0:
        return ()

    slice_cases = iter_slice_cases(plan)
    if not slice_cases:
        return ()

    prepared: list[PreparedCorpusRequest] = []
    for request_index in range(target_requests):
        slice_name, case = slice_cases[request_index % len(slice_cases)]
        variant_index = request_index // len(slice_cases)
        prepared.append(_build_case_request(slice_name, case, request_index, variant_index))
    return tuple(prepared)


def _build_case_request(
    slice_name: str,
    case: CorpusCasePlan,
    corpus_index: int,
    variant_index: int,
) -> PreparedCorpusRequest:
    deck_size = (
        len(case.deck.base_deck)
        + len(case.deck.added_cards)
        - len(case.deck.removed_cards)
    )
    upgraded_cards = len(case.deck.upgraded_cards)
    enemy_pressure = _enemy_pressure(case.enemy)
    hp_offset = variant_index % 7
    stance = "Calm" if ("calm" in case.deck.family or "setup" in case.tags) and variant_index % 2 == 0 else "Neutral"
    player_hp = max(18, 72 - case.floor - hp_offset + case.remove_count + upgraded_cards)
    block = 0 if enemy_pressure >= 3.5 else (variant_index % 6)
    draw_pile_size = max(deck_size - 5 - (variant_index % 3), 0)
    discard_pile_size = variant_index % 4
    exhaust_pile_size = 1 if "hard" in case.tags and variant_index % 3 == 0 else 0

    attack_strength = 1.8 + case.remove_count * 0.35 + upgraded_cards * 0.25 + enemy_pressure * 0.1
    setup_strength = 0.8 + upgraded_cards * 0.45 + (0.6 if "setup" in case.tags else 0.0)
    defend_strength = 0.6 + enemy_pressure * 0.45 + max(0, 28 - player_hp) * 0.03
    potion_strength = 1.5 + enemy_pressure * 0.55 + len(case.potion_set) * 0.25

    attack_action_id = f"attack::{case.case_id}::{variant_index}"
    setup_action_id = f"setup::{case.case_id}::{variant_index}"
    defend_action_id = f"defend::{case.case_id}::{variant_index}"
    potion_action_id = f"potion::{case.case_id}::{variant_index}"
    end_action_id = f"end::{case.case_id}::{variant_index}"

    candidates = [
        LegalCombatCandidate(
            attack_action_id,
            "card",
            features=(attack_strength, 0.0, enemy_pressure, 0.0),
            legal=True,
        ),
        LegalCombatCandidate(
            setup_action_id,
            "card",
            features=(0.2, setup_strength, enemy_pressure * 0.5, 1.0 + upgraded_cards * 0.25),
            legal=True,
        ),
        LegalCombatCandidate(
            defend_action_id,
            "card",
            features=(0.0, 0.1, defend_strength, max(0.0, 36.0 - player_hp) * 0.05),
            legal=True,
        ),
    ]
    if case.potion_set:
        candidates.append(
            LegalCombatCandidate(
                potion_action_id,
                "potion",
                features=(potion_strength, setup_strength * 0.5, enemy_pressure, 1.5),
                legal=True,
            )
        )
    candidates.append(
        LegalCombatCandidate(
            end_action_id,
            "end_turn",
            features=(0.05, 0.05, 0.05, 0.0),
            legal=True,
        )
    )

    if case.potion_set and ("elite" in case.tags or "boss" in case.tags) and variant_index % 4 == 0:
        preferred_action_id = potion_action_id
    elif "opening-hand" in case.tags or "setup" in case.tags or "calm" in case.deck.family:
        preferred_action_id = setup_action_id if variant_index % 3 != 1 else attack_action_id
    elif player_hp <= 24 and variant_index % 3 == 2:
        preferred_action_id = defend_action_id
    else:
        preferred_action_id = attack_action_id

    opening_hand_bucket = f"{slice_name}-variant-{variant_index % 8:02d}"
    corpus_id = _corpus_id(slice_name, case, corpus_index)
    corpus_group = _corpus_group(slice_name, case, opening_hand_bucket)
    request = CombatSearchRequest(
        request_id=f"{slice_name}:{case.case_id}:{variant_index:04d}",
        state=CombatStateSummary(
            combat_id=case.case_id,
            turn=1 + (variant_index % 3),
            hp=player_hp,
            block=block,
            energy=3,
            hand_size=5,
            draw_pile_size=draw_pile_size,
            discard_pile_size=discard_pile_size,
            exhaust_pile_size=exhaust_pile_size,
            stance=stance,
        ),
        candidates=tuple(candidates),
        metadata={
            "character": "Watcher",
            "ascension": 0,
            "corpus_slice": slice_name,
            "corpus_case": case.case_id,
            "deck_family": case.deck_family,
            "remove_count": case.remove_count,
            "removed_cards": list(case.deck.removed_cards),
            "added_cards": list(case.deck.added_cards),
            "upgraded_cards": list(case.deck.upgraded_cards),
            "potion_set": list(case.potion_set),
            "enemy": case.enemy,
            "seed_source": case.seed_provenance.source,
            "seed_label": case.seed_provenance.label,
            "neow_source": case.neow_provenance.source,
            "neow_option_label": case.neow_provenance.option_label,
            "opening_hand_bucket": opening_hand_bucket,
            "variant_index": variant_index,
            "corpus_index": corpus_index,
            "collection_pass": variant_index,
            "corpus_id": corpus_id,
            "corpus_group": corpus_group,
            "tags": list(case.tags),
        },
    )
    return PreparedCorpusRequest(
        corpus_index=corpus_index,
        corpus_id=corpus_id,
        corpus_group=corpus_group,
        collection_pass=variant_index,
        slice_name=slice_name,
        case=case,
        variant_index=variant_index,
        opening_hand_bucket=opening_hand_bucket,
        request=request,
        preferred_action_id=preferred_action_id,
    )


def _enemy_pressure(enemy: str) -> float:
    return {
        "Jaw Worm": 1.4,
        "Cultist": 1.1,
        "Gremlin Nob": 3.8,
        "Sentries": 3.0,
        "Lagavulin": 3.4,
        "Hexaghost": 4.2,
    }.get(enemy, 2.0)
