"""Stage 2 snapshot-backed Rust-PUCT collection and validation helpers."""

from __future__ import annotations

from collections import Counter
import json
from dataclasses import asdict, dataclass
from pathlib import Path
from statistics import mean
from typing import Any, Callable, Iterable, Mapping

from .benchmarking import BenchmarkFrontierPoint, build_frontier_report
from .contracts import CombatOutcomeVector, CombatPuctConfig, CombatPuctResult, CombatSearchStopReason
from .corpus import WATCHER_STARTER_DECK
from .encounters import ENCOUNTER_CATALOG, encounter_spec
from .entity_catalog import canonicalize_relic_id, canonicalize_watcher_card_id
from .engine_adapter import (
    action_id_for_candidate,
    build_model_evaluator,
    build_search_request_from_training_state,
    should_promote_collection_result,
)
from .engine_module import build_engine_extension, load_engine_module
from .inference_service import CombatInferenceService, CombatSearchConfig
from .run_logging import TrainingArtifacts, TrainingRunLogger
from .seed_imports import ImportedCombatCase, default_imported_act1_scripts, build_imported_combat_cases
from .shared_memory import CombatPuctTargetExample, CombatSearchRequest
from .value_targets import CombatValueTarget, PHASE1_POTION_VOCAB


PHASE2_CORPUS_NAME = "watcher_a0_act1_snapshot"
POTION_VARIANTS: tuple[tuple[str, ...], ...] = (
    (),
    ("FlexPotion",),
    ("StancePotion",),
    ("SwiftPotion",),
    ("FearPotion",),
    ("FirePotion",),
    ("DexterityPotion",),
)


@dataclass(frozen=True)
class SnapshotCase:
    case_id: str
    source_kind: str
    slice_name: str
    deck_family: str
    enemy: str
    room_kind: str
    remove_count: int
    potion_set: tuple[str, ...]
    relic_profile: str
    seed_label: str | None
    act1_floor: int
    opening_hand_bucket: str
    snapshot: Mapping[str, Any]
    metadata: Mapping[str, Any]

    def to_dict(self) -> dict[str, Any]:
        return {
            "case_id": self.case_id,
            "source_kind": self.source_kind,
            "slice_name": self.slice_name,
            "deck_family": self.deck_family,
            "enemy": self.enemy,
            "room_kind": self.room_kind,
            "remove_count": self.remove_count,
            "potion_set": list(self.potion_set),
            "relic_profile": self.relic_profile,
            "seed_label": self.seed_label,
            "act1_floor": self.act1_floor,
            "opening_hand_bucket": self.opening_hand_bucket,
            "snapshot": dict(self.snapshot),
            "metadata": dict(self.metadata),
        }

    @classmethod
    def from_dict(cls, payload: Mapping[str, Any]) -> "SnapshotCase":
        return cls(
            case_id=str(payload["case_id"]),
            source_kind=str(payload["source_kind"]),
            slice_name=str(payload["slice_name"]),
            deck_family=str(payload["deck_family"]),
            enemy=str(payload["enemy"]),
            room_kind=str(payload["room_kind"]),
            remove_count=int(payload["remove_count"]),
            potion_set=tuple(payload.get("potion_set", ())),
            relic_profile=str(payload.get("relic_profile", "starting_only")),
            seed_label=payload.get("seed_label"),
            act1_floor=int(payload["act1_floor"]),
            opening_hand_bucket=str(payload["opening_hand_bucket"]),
            snapshot=dict(payload["snapshot"]),
            metadata=dict(payload.get("metadata", {})),
        )


@dataclass(frozen=True)
class PuctCollectionRecord:
    case: SnapshotCase
    collection_pass: int
    request: CombatSearchRequest
    puct_result: CombatPuctResult

    def to_dict(self) -> dict[str, Any]:
        puct_payload = asdict(self.puct_result)
        puct_payload["stop_reason"] = self.puct_result.stop_reason.value
        return {
            "case": self.case.to_dict(),
            "collection_pass": self.collection_pass,
            "request": self.request.to_dict(),
            "puct_result": puct_payload,
        }

    @classmethod
    def from_dict(cls, payload: Mapping[str, Any]) -> "PuctCollectionRecord":
        from .contracts import parse_combat_puct_result

        return cls(
            case=SnapshotCase.from_dict(payload["case"]),
            collection_pass=int(payload["collection_pass"]),
            request=CombatSearchRequest.from_dict(payload["request"]),
            puct_result=parse_combat_puct_result(payload["puct_result"]),
        )


def _write_json(path: Path, payload: Mapping[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(dict(payload), indent=2, sort_keys=True) + "\n", encoding="utf-8")


def _write_jsonl(path: Path, rows: Iterable[Mapping[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        for row in rows:
            handle.write(json.dumps(dict(row), sort_keys=True))
            handle.write("\n")


def _load_jsonl(path: Path) -> tuple[dict[str, Any], ...]:
    rows: list[dict[str, Any]] = []
    with path.open(encoding="utf-8") as handle:
        for raw_line in handle:
            line = raw_line.strip()
            if not line:
                continue
            rows.append(json.loads(line))
    return tuple(rows)


def _normalize_deck(base_deck: Iterable[str], *, removed_cards: Iterable[str], added_cards: Iterable[str], upgraded_cards: Iterable[str]) -> list[str]:
    deck = [canonicalize_watcher_card_id(card) for card in base_deck]
    for removed in removed_cards:
        normalized_removed = canonicalize_watcher_card_id(removed)
        if normalized_removed in deck:
            deck.remove(normalized_removed)
    deck.extend(added_cards)
    for upgraded in upgraded_cards:
        normalized = canonicalize_watcher_card_id(upgraded)
        if not normalized.endswith("+"):
            normalized = f"{normalized}+"
        base_id = normalized[:-1]
        for index, card in enumerate(deck):
            if card == base_id:
                deck[index] = normalized
                break
        else:
            deck.append(normalized)
    return [canonicalize_watcher_card_id(card) for card in deck]


def _starter_remove_count(deck: Iterable[str]) -> int:
    starter = Counter(card.removesuffix("+") for card in WATCHER_STARTER_DECK)
    current = Counter(canonicalize_watcher_card_id(card).removesuffix("+") for card in deck)
    return sum(max(0, count - current.get(card_id, 0)) for card_id, count in starter.items())


def _synthetic_families() -> tuple[dict[str, Any], ...]:
    return (
        {
            "family": "starting_only",
            "removed_cards": (),
            "added_cards": (),
            "upgraded_cards": (),
            "relics": ("PureWater",),
            "relic_profile": "starting_only",
            "weight": 6,
        },
        {
            "family": "starting_only_remove_heavy",
            "removed_cards": ("Defend_P", "Defend_P", "Strike_P"),
            "added_cards": ("CutThroughFate", "ThirdEye"),
            "upgraded_cards": ("Eruption",),
            "relics": ("PureWater",),
            "relic_profile": "starting_only",
            "weight": 6,
        },
        {
            "family": "starting_only_upgrade_heavy",
            "removed_cards": ("Strike_P",),
            "added_cards": ("ThirdEye", "TalkToTheHand"),
            "upgraded_cards": ("Vigilance", "LessonLearned"),
            "relics": ("PureWater",),
            "relic_profile": "starting_only",
            "weight": 6,
        },
        {
            "family": "starting_only_stance_shell",
            "removed_cards": ("Defend_P",),
            "added_cards": ("Tantrum", "Rushdown", "MentalFortress"),
            "upgraded_cards": ("BowlingBash",),
            "relics": ("PureWater",),
            "relic_profile": "starting_only",
            "weight": 6,
        },
        {
            "family": "starting_only_retain_control",
            "removed_cards": ("Strike_P", "Strike_P"),
            "added_cards": ("DeusExMachina", "ThirdEye", "Perseverance"),
            "upgraded_cards": ("Vigilance",),
            "relics": ("PureWater",),
            "relic_profile": "starting_only",
            "weight": 6,
        },
        {
            "family": "extra_relic_ablation_akabeko",
            "removed_cards": ("Defend_P", "Defend_P", "Strike_P"),
            "added_cards": ("CutThroughFate", "ThirdEye"),
            "upgraded_cards": ("Eruption",),
            "relics": ("PureWater", "Akabeko"),
            "relic_profile": "extra_relic",
            "weight": 1,
        },
        {
            "family": "extra_relic_ablation_frozen_eye",
            "removed_cards": ("Strike_P",),
            "added_cards": ("ThirdEye", "TalkToTheHand"),
            "upgraded_cards": ("Vigilance", "LessonLearned"),
            "relics": ("PureWater", "FrozenEye"),
            "relic_profile": "extra_relic",
            "weight": 1,
        },
        {
            "family": "extra_relic_ablation_pocketwatch",
            "removed_cards": ("Defend_P",),
            "added_cards": ("Tantrum", "Rushdown", "MentalFortress"),
            "upgraded_cards": ("BowlingBash",),
            "relics": ("PureWater", "Pocketwatch"),
            "relic_profile": "extra_relic",
            "weight": 1,
        },
        {
            "family": "extra_relic_ablation_ice_cream",
            "removed_cards": ("Strike_P", "Strike_P"),
            "added_cards": ("DeusExMachina", "ThirdEye", "Perseverance"),
            "upgraded_cards": ("Vigilance",),
            "relics": ("PureWater", "IceCream"),
            "relic_profile": "extra_relic",
            "weight": 1,
        },
    )


def _weighted_family_pool() -> tuple[dict[str, Any], ...]:
    pool: list[dict[str, Any]] = []
    for family in _synthetic_families():
        pool.extend(family for _ in range(int(family.get("weight", 1))))
    return tuple(pool)


def _mutate_snapshot_for_bucket(snapshot: dict[str, Any], bucket_index: int, potion_set: tuple[str, ...]) -> dict[str, Any]:
    mutated = json.loads(json.dumps(snapshot))
    combined = list(mutated["hand"]) + list(mutated["draw_pile"])
    if combined:
        rotation = bucket_index % len(combined)
        combined = combined[rotation:] + combined[:rotation]
        mutated["hand"] = combined[: min(5, len(combined))]
        mutated["draw_pile"] = combined[min(5, len(combined)) :]
    mutated["potions"] = list(potion_set)
    while len(mutated["potions"]) < 5:
        mutated["potions"].append("")
    mutated["potions"] = mutated["potions"][:5]
    mutated["player_hp"] = max(12, int(mutated["player_hp"]) - (bucket_index % 4))
    mutated["player_block"] = bucket_index % 3
    mutated["rng_counter"] = int(mutated["rng_counter"]) + bucket_index
    return mutated


def _build_synthetic_snapshot_cases(total_cases: int) -> tuple[SnapshotCase, ...]:
    engine_mod = load_engine_module()
    encounter_names = tuple(ENCOUNTER_CATALOG.keys())
    families = _weighted_family_pool()
    buckets = 8
    cases: list[SnapshotCase] = []
    for case_index in range(total_cases):
        family = families[case_index % len(families)]
        encounter_name = encounter_names[(case_index // len(families)) % len(encounter_names)]
        encounter = encounter_spec(encounter_name)
        potion_set = POTION_VARIANTS[(case_index // (len(families) * len(encounter_names))) % len(POTION_VARIANTS)]
        bucket_index = case_index % buckets
        deck = _normalize_deck(
            WATCHER_STARTER_DECK,
            removed_cards=family["removed_cards"],
            added_cards=family["added_cards"],
            upgraded_cards=family["upgraded_cards"],
        )
        engine = engine_mod.RustCombatEngine(
            72,
            72,
            3,
            deck,
            encounter.to_engine_enemies(),
            42 + case_index,
            [canonicalize_relic_id(relic) for relic in family["relics"]],
        )
        engine.start_combat()
        snapshot = _mutate_snapshot_for_bucket(engine.get_combat_snapshot(), bucket_index, potion_set)
        floor = encounter.floor_hint
        room_kind = encounter.room_kind
        opening_bucket = f"{room_kind}-bucket-{bucket_index:02d}"
        cases.append(
            SnapshotCase(
                case_id=f"{PHASE2_CORPUS_NAME}::synthetic::{case_index:05d}",
                source_kind="synthetic",
                slice_name=f"synthetic-{room_kind}",
                deck_family=family["family"],
                enemy=encounter_name,
                room_kind=room_kind,
                remove_count=len(family["removed_cards"]),
                potion_set=potion_set,
                relic_profile=str(family["relic_profile"]),
                seed_label=None,
                act1_floor=floor,
                opening_hand_bucket=opening_bucket,
                snapshot=snapshot,
                metadata={
                    "character": "Watcher",
                    "ascension": 0,
                    "relics": list(family["relics"]),
                    "relic_profile": family["relic_profile"],
                    "removed_cards": list(family["removed_cards"]),
                    "added_cards": list(family["added_cards"]),
                    "upgraded_cards": list(family["upgraded_cards"]),
                    "bucket_index": bucket_index,
                },
            )
        )
    return tuple(cases)


def _build_imported_snapshot_case(
    imported_case: ImportedCombatCase,
    *,
    case_index: int,
    bucket_index: int,
    potion_variant_index: int,
) -> SnapshotCase:
    engine_mod = load_engine_module()
    encounter = encounter_spec(imported_case.encounter)
    engine = engine_mod.RustCombatEngine(
        imported_case.current_hp,
        imported_case.max_hp,
        3,
        list(imported_case.deck),
        encounter.to_engine_enemies(),
        7_000 + case_index,
        list(imported_case.relics),
    )
    engine.start_combat()
    base_snapshot = engine.get_combat_snapshot()
    if imported_case.potions:
        potion_pool = list(imported_case.potions)
    else:
        potion_pool = []
    if potion_variant_index == 1 and potion_pool:
        potion_pool = potion_pool[:-1]
    snapshot = _mutate_snapshot_for_bucket(base_snapshot, bucket_index, tuple(potion_pool))
    snapshot["player_hp"] = imported_case.current_hp
    snapshot["player_max_hp"] = imported_case.max_hp
    return SnapshotCase(
        case_id=f"{PHASE2_CORPUS_NAME}::imported::{imported_case.seed_label}::{imported_case.floor:02d}::{bucket_index:02d}::{potion_variant_index:02d}",
        source_kind="imported_seed",
        slice_name="imported-seed",
        deck_family=imported_case.seed_label,
        enemy=imported_case.encounter,
        room_kind=encounter.room_kind,
        remove_count=_starter_remove_count(imported_case.deck),
        potion_set=tuple(potion_pool),
        relic_profile=("starting_only" if tuple(imported_case.relics) == ("PureWater",) else "extra_relic"),
        seed_label=imported_case.seed_label,
        act1_floor=imported_case.floor,
        opening_hand_bucket=f"imported-{bucket_index:02d}",
        snapshot=snapshot,
        metadata={
            "seed": imported_case.seed,
            "source_url": imported_case.source_url,
            "gold": imported_case.gold,
            "relics": list(imported_case.relics),
            "relic_profile": "starting_only" if tuple(imported_case.relics) == ("PureWater",) else "extra_relic",
            "notes": list(imported_case.notes),
        },
    )


def build_phase2_snapshot_corpus(*, total_cases: int) -> tuple[SnapshotCase, ...]:
    synthetic_target = max(0, min(total_cases, round(total_cases * 0.84)))
    imported_target = max(0, total_cases - synthetic_target)
    cases = list(_build_synthetic_snapshot_cases(synthetic_target))
    imported_cases = build_imported_combat_cases()
    if imported_cases and imported_target > 0:
        for case_index in range(imported_target):
            imported_case = imported_cases[case_index % len(imported_cases)]
            bucket_index = case_index % 8
            potion_variant_index = case_index % 2
            cases.append(
                _build_imported_snapshot_case(
                    imported_case,
                    case_index=case_index,
                    bucket_index=bucket_index,
                    potion_variant_index=potion_variant_index,
                )
            )
    return tuple(cases[:total_cases])


def write_snapshot_corpus(output_dir: Path, *, total_cases: int) -> dict[str, Any]:
    cases = build_phase2_snapshot_corpus(total_cases=total_cases)
    _write_jsonl(output_dir / "corpus.jsonl", (case.to_dict() for case in cases))
    summary = {
        "corpus_name": PHASE2_CORPUS_NAME,
        "total_cases": len(cases),
        "source_counts": {
            "synthetic": sum(1 for case in cases if case.source_kind == "synthetic"),
            "imported_seed": sum(1 for case in cases if case.source_kind == "imported_seed"),
        },
        "relic_profile_counts": {
            name: sum(1 for case in cases if case.relic_profile == name)
            for name in sorted({case.relic_profile for case in cases})
        },
        "family_counts": {
            name: sum(1 for case in cases if case.deck_family == name)
            for name in sorted({case.deck_family for case in cases})
        },
        "slice_counts": {
            name: sum(1 for case in cases if case.slice_name == name)
            for name in sorted({case.slice_name for case in cases})
        },
    }
    _write_json(output_dir / "corpus_summary.json", summary)
    return summary


def load_snapshot_corpus(path: Path) -> tuple[SnapshotCase, ...]:
    if path.is_dir():
        path = path / "corpus.jsonl"
    return tuple(SnapshotCase.from_dict(row) for row in _load_jsonl(path))


def _request_from_solver(solver: Any, *, case: SnapshotCase) -> CombatSearchRequest:
    from .bridge import load_combat_training_state

    state = load_combat_training_state(solver)
    return build_search_request_from_training_state(
        state,
        request_id=case.case_id,
        metadata={
            **case.metadata,
            "source_kind": case.source_kind,
            "slice_name": case.slice_name,
            "deck_family": case.deck_family,
            "enemy": case.enemy,
            "remove_count": case.remove_count,
            "potion_set": list(case.potion_set),
            "seed_label": case.seed_label,
            "act1_floor": case.act1_floor,
            "opening_hand_bucket": case.opening_hand_bucket,
        },
    )


def _config_for_room(room_kind: str, multiplier: int = 1) -> CombatPuctConfig:
    if room_kind == "boss":
        return CombatPuctConfig(
            min_visits=4096 * multiplier,
            visit_window=1024 * multiplier,
            hard_visit_cap=16384 * multiplier,
            time_cap_ms=10000 * multiplier,
        )
    if room_kind == "elite":
        return CombatPuctConfig(
            min_visits=2048 * multiplier,
            visit_window=512 * multiplier,
            hard_visit_cap=8192 * multiplier,
            time_cap_ms=4000 * multiplier,
        )
    return CombatPuctConfig(
        min_visits=1024 * multiplier,
        visit_window=256 * multiplier,
        hard_visit_cap=4096 * multiplier,
        time_cap_ms=1500 * multiplier,
    )


def collect_rust_puct_records(
    *,
    cases: tuple[SnapshotCase, ...],
    collection_passes: int,
    checkpoint_path: Path | None = None,
    on_record: Callable[[PuctCollectionRecord, int], None] | None = None,
) -> tuple[PuctCollectionRecord, ...]:
    from .combat_model import MLXCombatModel

    engine_mod = load_engine_module()
    model = MLXCombatModel(checkpoint_path=str(checkpoint_path) if checkpoint_path else None)
    service = CombatInferenceService.build(model=model, config=CombatSearchConfig(top_k=8))
    records: list[PuctCollectionRecord] = []
    active_cases = list(cases)
    for pass_index in range(collection_passes):
        next_cases: list[SnapshotCase] = []
        multiplier = 1 if pass_index == 0 else 2 if pass_index == 1 else 4
        for case in active_cases:
            snapshot_json = json.dumps(case.snapshot)
            solver = engine_mod.CombatSolver.from_snapshot_json(snapshot_json)
            request = _request_from_solver(solver, case=case)
            evaluator = build_model_evaluator(
                service,
                metadata_factory=lambda _: {"request_id": case.case_id},
            )
            puct_result = solver.run_combat_puct(evaluator, json.dumps(_config_for_room(case.room_kind, multiplier).to_dict()))
            from .bridge import parse_combat_puct_result

            parsed = parse_combat_puct_result(puct_result)
            record = PuctCollectionRecord(
                case=case,
                collection_pass=pass_index,
                request=request,
                puct_result=parsed,
            )
            records.append(record)
            if on_record is not None:
                on_record(record, len(records))
            if pass_index + 1 < collection_passes and should_promote_collection_result(
                stop_reason=parsed.stop_reason,
                root_visit_shares=parsed.root_visit_shares,
                root_outcome=parsed.root_outcome,
                room_kind=case.room_kind,
            ):
                next_cases.append(case)
        active_cases = next_cases
        if not active_cases:
            break
    return tuple(records)


def write_puct_collection(
    output_dir: Path,
    *,
    cases: tuple[SnapshotCase, ...],
    collection_passes: int,
    checkpoint_path: Path | None = None,
    on_record: Callable[[PuctCollectionRecord, int], None] | None = None,
) -> tuple[PuctCollectionRecord, ...]:
    records = collect_rust_puct_records(
        cases=cases,
        collection_passes=collection_passes,
        checkpoint_path=checkpoint_path,
        on_record=on_record,
    )
    _write_jsonl(output_dir / "puct_collection.jsonl", (record.to_dict() for record in records))
    pass_counts = {
        f"pass_{index:03d}": sum(1 for record in records if record.collection_pass == index)
        for index in range(collection_passes)
    }
    _write_json(
        output_dir / "puct_targets_report.json",
        {
            "corpus_name": PHASE2_CORPUS_NAME,
            "total_records": len(records),
            "collection_passes": collection_passes,
            "backend_requested": "mlx",
            "backend_loaded": "mlx",
            "pass_counts": pass_counts,
        },
    )
    return records


def _potion_target_for_record(record: PuctCollectionRecord) -> CombatValueTarget:
    candidate_lookup = {candidate.action_id: candidate for candidate in record.request.candidates}
    total_visits = sum(record.puct_result.root_visits)
    potion_weights: dict[str, float] = {}
    if total_visits > 0:
        for action_id, visit_count in zip(record.puct_result.root_action_ids, record.puct_result.root_visits):
            candidate = candidate_lookup.get(str(action_id))
            if candidate is None or not candidate.potion_id:
                continue
            if candidate.potion_id not in PHASE1_POTION_VOCAB:
                continue
            potion_weights[candidate.potion_id] = potion_weights.get(candidate.potion_id, 0.0) + (
                float(visit_count) / float(total_visits)
            )

    potion_spend_count = float(record.puct_result.root_outcome.potion_cost)
    if potion_spend_count > 0.0 and potion_weights:
        scale = potion_spend_count / max(sum(potion_weights.values()), 1e-6)
        potion_weights = {
            potion_id: float(weight * scale) for potion_id, weight in potion_weights.items()
        }
    elif potion_spend_count == 0.0 and potion_weights:
        potion_spend_count = float(sum(potion_weights.values()))

    return CombatValueTarget(
        solve_probability=float(record.puct_result.root_outcome.solve_probability),
        expected_hp_loss=float(record.puct_result.root_outcome.expected_hp_loss),
        expected_turns=float(record.puct_result.root_outcome.expected_turns),
        potion_spend_count=potion_spend_count,
        setup_delta=float(record.puct_result.root_outcome.setup_value_delta),
        persistent_scaling_delta=float(record.puct_result.root_outcome.persistent_scaling_delta),
        potion_spend_by_id=potion_weights,
    )


def records_to_puct_targets(records: Iterable[PuctCollectionRecord]) -> list[CombatPuctTargetExample]:
    examples: list[CombatPuctTargetExample] = []
    for record in records:
        chosen_action = None
        if record.puct_result.root_visits:
            best_index = max(range(len(record.puct_result.root_visits)), key=record.puct_result.root_visits.__getitem__)
            if best_index < len(record.puct_result.root_action_ids):
                chosen_action = str(record.puct_result.root_action_ids[best_index])
        if chosen_action is None and record.puct_result.chosen_action_id is not None:
            chosen_action = str(record.puct_result.chosen_action_id)
        if chosen_action is None:
            continue
        examples.append(
            CombatPuctTargetExample(
                request=record.request,
                policy_action_ids=tuple(str(action_id) for action_id in record.puct_result.root_action_ids),
                policy_scores=tuple(float(value) for value in record.puct_result.root_visit_shares),
                value_target=_potion_target_for_record(record),
                chosen_action_id=chosen_action,
                visit_counts=tuple(int(value) for value in record.puct_result.root_visits),
                sample_weight=1.0 + float(record.collection_pass),
                metadata={
                    **dict(record.request.metadata),
                    "source_kind": record.case.source_kind,
                    "collection_pass": record.collection_pass,
                    "root_visits": list(record.puct_result.root_visits),
                    "root_action_ids": list(record.puct_result.root_action_ids),
                    "stop_reason": record.puct_result.stop_reason.value,
                    "target_source": "rust_puct_snapshot",
                    "relic_profile": record.case.relic_profile,
                },
            )
        )
    return examples


def frontier_points_from_records(records: Iterable[PuctCollectionRecord]) -> list[BenchmarkFrontierPoint]:
    points: list[BenchmarkFrontierPoint] = []
    for record in records:
        outcome = record.puct_result.root_outcome
        points.append(
            BenchmarkFrontierPoint(
                label=record.case.case_id,
                win_rate=outcome.solve_probability,
                avg_floor=50.0 - outcome.expected_hp_loss,
                throughput_gpm=60_000.0 / max(1.0, float(record.puct_result.elapsed_ms)),
                deck_family=record.case.deck_family,
                remove_count=record.case.remove_count,
                potion_set=record.case.potion_set,
                enemy=record.case.enemy,
            )
        )
    return points


def build_seed_validation_report(
    *,
    checkpoint: str,
) -> dict[str, Any]:
    scripts = default_imported_act1_scripts()
    imported_cases = build_imported_combat_cases(scripts)
    seed_rows = []
    stop_reason_counts: dict[str, int] = {}
    required_seed_labels = {script.label for script in scripts if script.exact_available}
    records = collect_rust_puct_records(
        cases=tuple(
            _build_imported_snapshot_case(case, case_index=index, bucket_index=0, potion_variant_index=0)
            for index, case in enumerate(imported_cases)
        ),
        collection_passes=1,
        checkpoint_path=Path(checkpoint) if checkpoint else None,
    )
    by_seed: dict[str, list[PuctCollectionRecord]] = {}
    for record in records:
        by_seed.setdefault(record.case.seed_label or "unknown", []).append(record)
        stop_reason_counts[record.puct_result.stop_reason.value] = stop_reason_counts.get(record.puct_result.stop_reason.value, 0) + 1

    for script in scripts:
        if not script.exact_available:
            seed_rows.append(
                {
                    "seed": script.seed,
                    "label": script.label,
                    "status": "metadata_only",
                    "stop_reason": "ImportBlocked",
                    "note": script.exact_issue,
                    "source_url": script.source_url,
                }
            )
            continue
        seed_records = by_seed.get(script.label, [])
        if not seed_records:
            seed_rows.append(
                {
                    "seed": script.seed,
                    "label": script.label,
                    "status": "reconstructed_with_uncertainty",
                    "stop_reason": "ImportBlocked",
                    "source_url": script.source_url,
                }
            )
            continue
        mean_visits = mean(record.puct_result.root_total_visits for record in seed_records)
        mean_frontier = mean(len(record.puct_result.frontier) for record in seed_records)
        mean_solve = mean(record.puct_result.root_outcome.solve_probability for record in seed_records)
        mean_hp_loss = mean(record.puct_result.root_outcome.expected_hp_loss for record in seed_records)
        stop_reason = max(seed_records, key=lambda record: record.puct_result.root_total_visits).puct_result.stop_reason.value
        seed_rows.append(
            {
                "seed": script.seed,
                "label": script.label,
                "status": "reconstructed_validated",
                "stop_reason": stop_reason,
                "root_visits": round(mean_visits, 2),
                "frontier_width": round(mean_frontier, 2),
                "solve_rate": round(mean_solve, 4),
                "expected_hp_loss": round(mean_hp_loss, 4),
                "combats": len(seed_records),
                "boss_cleared": any(record.case.act1_floor == 16 and record.puct_result.root_outcome.solve_probability >= 0.5 for record in seed_records),
                "checkpoint": checkpoint,
                "source_url": script.source_url,
                "reconstructed_floors": [floor.floor for floor in script.floors if floor.is_combat],
                "act1_boundary_floor": 16,
            }
        )

    return {
        "suite_name": "watcher_validation_suite",
        "checkpoint": checkpoint,
        "benchmark_config": PHASE2_CORPUS_NAME,
        "backend_requested": "mlx",
        "backend_loaded": "mlx",
        "seed_count": len(scripts),
        "validated_seeds": sum(1 for row in seed_rows if row["status"] == "reconstructed_validated"),
        "failed_seeds": sum(
            1
            for row in seed_rows
            if row["label"] in required_seed_labels and row["status"] != "reconstructed_validated"
        ),
        "required_seed_count": len(required_seed_labels),
        "metadata_only_count": sum(1 for row in seed_rows if row["status"] == "metadata_only"),
        "seeds": seed_rows,
        "stop_reason_counts": stop_reason_counts,
        "checkpoint_comparisons": [
            {
                "from_checkpoint": "baseline",
                "to_checkpoint": checkpoint,
                "seed_count": sum(
                    1
                    for row in seed_rows
                    if row["label"] in required_seed_labels and row["status"] == "reconstructed_validated"
                ),
                "note": "Stage 2 reconstructed Act 1 validation over the 2 required seeds",
            }
        ],
    }
