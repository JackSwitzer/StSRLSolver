"""Typed Python mirrors for the Rust combat-training contract."""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from enum import Enum
from typing import Any, Mapping


class RestrictionBuiltin(str, Enum):
    NO_CARD_REWARDS = "NoCardRewards"
    NO_CARD_ADDS = "NoCardAdds"
    UPGRADE_REMOVE_ONLY = "UpgradeRemoveOnly"


class CombatSearchStopReason(str, Enum):
    CONVERGED = "Converged"
    HARD_VISIT_CAP = "HardVisitCap"
    TIME_CAP = "TimeCap"
    TERMINAL_ROOT = "TerminalRoot"
    NO_LEGAL_ACTIONS = "NoLegalActions"


@dataclass(frozen=True)
class RestrictionPolicy:
    builtins: tuple[RestrictionBuiltin, ...] = ()

    def to_dict(self) -> dict[str, Any]:
        return {
            "builtins": [builtin.value for builtin in self.builtins],
        }


@dataclass(frozen=True)
class TrainingSchemaVersions:
    training_session_schema_version: int
    combat_observation_schema_version: int
    action_candidate_schema_version: int
    gameplay_export_schema_version: int
    replay_event_trace_schema_version: int


@dataclass(frozen=True)
class CombatObservationCaps:
    hand: int
    enemies: int
    player_effects: int
    enemy_effects_per_enemy: int
    orbs: int
    relic_counters: int
    choice_options: int


@dataclass(frozen=True)
class CombatGlobalToken:
    turn: int
    energy: int
    max_energy: int
    cards_played_this_turn: int
    attacks_played_this_turn: int
    hand_size: int
    draw_pile_size: int
    discard_pile_size: int
    exhaust_pile_size: int
    potion_slots: int
    orb_slot_count: int
    occupied_orb_slots: int
    player_hp: int
    player_max_hp: int
    player_block: int
    stance: str
    mantra: int
    mantra_gained: int
    skip_enemy_turn: bool
    blasphemy_active: bool
    combat_over: bool
    player_won: bool
    total_damage_dealt: int
    total_damage_taken: int
    total_cards_played: int


@dataclass(frozen=True)
class PlayerToken:
    hp: int
    max_hp: int
    block: int
    stance: str
    strength: int
    dexterity: int
    focus: int
    weak: int
    vulnerable: int
    frail: int
    relics: tuple[str, ...]


@dataclass(frozen=True)
class CardToken:
    hand_index: int
    card_id: str
    card_name: str
    card_type: str
    target: str
    cost_for_turn: int
    base_cost: int
    misc: int
    upgraded: bool
    free_to_play: bool
    retained: bool
    ethereal: bool
    runtime_only: bool
    x_cost: bool
    multi_hit: bool


@dataclass(frozen=True)
class EnemyToken:
    enemy_index: int
    enemy_id: str
    enemy_name: str
    hp: int
    max_hp: int
    block: int
    alive: bool
    targetable: bool
    back_attack: bool
    intent: str
    intent_damage: int
    intent_hits: int
    intent_block: int


@dataclass(frozen=True)
class StatusToken:
    status_id: int
    status_name: str
    amount: int


@dataclass(frozen=True)
class EnemyStatusToken:
    enemy_index: int
    status_id: int
    status_name: str
    amount: int


@dataclass(frozen=True)
class OrbToken:
    slot_index: int
    orb_type: str
    base_passive: int
    base_evoke: int
    evoke_amount: int


@dataclass(frozen=True)
class RelicCounterToken:
    counter_name: str
    value: int


@dataclass(frozen=True)
class CombatChoiceOption:
    choice_index: int
    kind: str
    source_index: int
    label: str
    selected: bool


@dataclass(frozen=True)
class CombatChoiceContext:
    active: bool
    reason: str | None
    min_picks: int
    max_picks: int
    selected: tuple[int, ...]
    options: tuple[CombatChoiceOption, ...]


@dataclass(frozen=True)
class CombatObservation:
    schema_version: int
    caps: CombatObservationCaps
    global_token: CombatGlobalToken
    player: PlayerToken
    hand: tuple[CardToken, ...]
    enemies: tuple[EnemyToken, ...]
    player_effects: tuple[StatusToken, ...]
    enemy_effects: tuple[EnemyStatusToken, ...]
    orbs: tuple[OrbToken, ...]
    relic_counters: tuple[RelicCounterToken, ...]
    choice: CombatChoiceContext


@dataclass(frozen=True)
class CandidateCardFeatures:
    hand_index: int
    card_id: str
    card_name: str
    card_type: str
    cost_for_turn: int
    base_cost: int
    upgraded: bool
    x_cost: bool
    multi_hit: bool
    free_to_play: bool


@dataclass(frozen=True)
class CandidateTargetFeatures:
    enemy_index: int
    enemy_name: str
    hp: int
    block: int
    targetable: bool
    back_attack: bool


@dataclass(frozen=True)
class CandidatePotionFeatures:
    slot: int
    potion_id: str
    target_required: bool


@dataclass(frozen=True)
class CandidateChoiceFeatures:
    choice_index: int
    label: str
    kind: str
    source_index: int


@dataclass(frozen=True)
class LegalActionCandidate:
    schema_version: int
    dense_index: int
    execution_id: int
    action_kind: str
    description: str
    card: CandidateCardFeatures | None = None
    target: CandidateTargetFeatures | None = None
    potion: CandidatePotionFeatures | None = None
    choice: CandidateChoiceFeatures | None = None


@dataclass(frozen=True)
class CombatOutcomeVector:
    solve_probability: float
    expected_hp_loss: float
    expected_turns: float
    potion_cost: float
    setup_value_delta: float
    persistent_scaling_delta: float


@dataclass(frozen=True)
class CombatFrontierLine:
    line_index: int
    action_prefix: tuple[int, ...]
    visits: int
    expanded_nodes: int
    elapsed_ms: int
    outcome: CombatOutcomeVector


@dataclass(frozen=True)
class CombatFrontierSummary:
    capacity: int
    lines: tuple[CombatFrontierLine, ...] = ()


@dataclass(frozen=True)
class CombatPuctConfig:
    cpuct: float = 1.35
    frontier_capacity: int = 8
    min_visits: int = 1024
    visit_window: int = 256
    hard_visit_cap: int = 4096
    time_cap_ms: int = 1500
    max_rollout_depth: int = 48
    stable_windows_required: int = 3
    best_visit_share_lead_threshold: float = 0.08
    root_value_delta_threshold: float = 0.01

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(frozen=True)
class CombatPuctLine:
    line_index: int
    action_prefix: tuple[int, ...]
    visits: int
    visit_share: float
    prior: float
    expanded_nodes: int
    elapsed_ms: int
    outcome: CombatOutcomeVector


@dataclass(frozen=True)
class CombatPuctResult:
    chosen_action_id: int | None
    root_action_ids: tuple[int, ...]
    root_visits: tuple[int, ...]
    root_visit_shares: tuple[float, ...]
    root_priors: tuple[float, ...]
    frontier: tuple[CombatPuctLine, ...]
    root_outcome: CombatOutcomeVector
    root_total_visits: int
    stable_windows: int
    nodes_expanded: int
    leaf_evaluations: int
    max_depth_reached: int
    elapsed_ms: int
    stop_reason: CombatSearchStopReason


@dataclass(frozen=True)
class CombatTrainingContext:
    runtime_scope: str
    decision_kind: str
    phase_label: str
    terminal: bool
    floor: int | None
    ascension: int | None
    seed: int | None


@dataclass(frozen=True)
class CombatTrainingState:
    schema_versions: TrainingSchemaVersions
    context: CombatTrainingContext
    observation: CombatObservation
    legal_candidates: tuple[LegalActionCandidate, ...]


@dataclass(frozen=True)
class CardSnapshot:
    card_id: str
    cost_for_turn: int
    base_cost: int
    misc: int
    upgraded: bool
    free_to_play: bool
    retained: bool
    ethereal: bool


@dataclass(frozen=True)
class EnemySnapshot:
    enemy_index: int
    enemy_id: str
    enemy_name: str
    hp: int
    max_hp: int
    block: int
    back_attack: bool
    move_id: int
    intent_damage: int
    intent_hits: int
    intent_block: int
    first_turn: bool
    is_escaping: bool
    statuses: tuple[StatusToken, ...] = ()


@dataclass(frozen=True)
class CombatSnapshot:
    schema_version: int
    player_hp: int
    player_max_hp: int
    player_block: int
    energy: int
    max_energy: int
    turn: int
    cards_played_this_turn: int
    attacks_played_this_turn: int
    stance: str
    mantra: int
    mantra_gained: int
    skip_enemy_turn: bool
    blasphemy_active: bool
    total_damage_dealt: int
    total_damage_taken: int
    total_cards_played: int
    player_effects: tuple[StatusToken, ...]
    hand: tuple[CardSnapshot, ...]
    draw_pile: tuple[CardSnapshot, ...]
    discard_pile: tuple[CardSnapshot, ...]
    exhaust_pile: tuple[CardSnapshot, ...]
    enemies: tuple[EnemySnapshot, ...]
    potions: tuple[str, ...]
    relics: tuple[str, ...]
    relic_counters: tuple[RelicCounterToken, ...]
    orb_slots: int
    rng_seed0: int
    rng_seed1: int
    rng_counter: int


@dataclass(frozen=True)
class RunManifest:
    git_sha: str
    git_dirty: bool
    combat_observation_schema_version: int
    action_candidate_schema_version: int
    gameplay_export_schema_version: int
    replay_event_trace_schema_version: int
    model_version: str
    benchmark_config: str
    seed: int
    restriction_policy: RestrictionPolicy
    hardware: str
    runtime: str

    def to_dict(self) -> dict[str, Any]:
        data = asdict(self)
        data["restriction_policy"] = self.restriction_policy.to_dict()
        return data


@dataclass(frozen=True)
class EpisodeStep:
    step_index: int
    action_id: int
    reward_delta: float
    done: bool
    search_frontier: CombatFrontierSummary | None = None
    value: CombatOutcomeVector | None = None


@dataclass(frozen=True)
class EpisodeLog:
    manifest: RunManifest | None
    steps: tuple[EpisodeStep, ...] = ()


@dataclass(frozen=True)
class BenchmarkSliceResult:
    slice_name: str
    cases: int
    solve_rate: float
    expected_hp_loss: float
    expected_turns: float
    oracle_top_k_agreement: float
    p95_elapsed_ms: float
    p95_rss_gb: float


@dataclass(frozen=True)
class BenchmarkReport:
    manifest: RunManifest | None
    slices: tuple[BenchmarkSliceResult, ...]


def _tuple_of(items: list[Any], ctor) -> tuple[Any, ...]:
    return tuple(ctor(item) for item in items)


def parse_training_schema_versions(payload: Mapping[str, Any]) -> TrainingSchemaVersions:
    return TrainingSchemaVersions(**payload)


def parse_combat_snapshot(payload: Mapping[str, Any]) -> CombatSnapshot:
    return CombatSnapshot(
        schema_version=payload["schema_version"],
        player_hp=payload["player_hp"],
        player_max_hp=payload["player_max_hp"],
        player_block=payload["player_block"],
        energy=payload["energy"],
        max_energy=payload["max_energy"],
        turn=payload["turn"],
        cards_played_this_turn=payload["cards_played_this_turn"],
        attacks_played_this_turn=payload["attacks_played_this_turn"],
        stance=payload["stance"],
        mantra=payload["mantra"],
        mantra_gained=payload["mantra_gained"],
        skip_enemy_turn=payload["skip_enemy_turn"],
        blasphemy_active=payload["blasphemy_active"],
        total_damage_dealt=payload["total_damage_dealt"],
        total_damage_taken=payload["total_damage_taken"],
        total_cards_played=payload["total_cards_played"],
        player_effects=_tuple_of(payload["player_effects"], lambda item: StatusToken(**item)),
        hand=_tuple_of(payload["hand"], lambda item: CardSnapshot(**item)),
        draw_pile=_tuple_of(payload["draw_pile"], lambda item: CardSnapshot(**item)),
        discard_pile=_tuple_of(payload["discard_pile"], lambda item: CardSnapshot(**item)),
        exhaust_pile=_tuple_of(payload["exhaust_pile"], lambda item: CardSnapshot(**item)),
        enemies=_tuple_of(
            payload["enemies"],
            lambda item: EnemySnapshot(
                **{
                    **item,
                    "statuses": _tuple_of(item["statuses"], lambda status: StatusToken(**status)),
                }
            ),
        ),
        potions=tuple(payload["potions"]),
        relics=tuple(payload["relics"]),
        relic_counters=_tuple_of(payload["relic_counters"], lambda item: RelicCounterToken(**item)),
        orb_slots=payload["orb_slots"],
        rng_seed0=payload["rng_seed0"],
        rng_seed1=payload["rng_seed1"],
        rng_counter=payload["rng_counter"],
    )


def parse_combat_training_state(payload: Mapping[str, Any]) -> CombatTrainingState:
    schema_versions = parse_training_schema_versions(payload["schema_versions"])
    observation_payload = payload["observation"]
    observation = CombatObservation(
        schema_version=observation_payload["schema_version"],
        caps=CombatObservationCaps(**observation_payload["caps"]),
        global_token=CombatGlobalToken(**observation_payload["global"]),
        player=PlayerToken(
            **{
                **observation_payload["player"],
                "relics": tuple(observation_payload["player"]["relics"]),
            }
        ),
        hand=_tuple_of(observation_payload["hand"], lambda item: CardToken(**item)),
        enemies=_tuple_of(observation_payload["enemies"], lambda item: EnemyToken(**item)),
        player_effects=_tuple_of(
            observation_payload["player_effects"], lambda item: StatusToken(**item)
        ),
        enemy_effects=_tuple_of(
            observation_payload["enemy_effects"], lambda item: EnemyStatusToken(**item)
        ),
        orbs=_tuple_of(observation_payload["orbs"], lambda item: OrbToken(**item)),
        relic_counters=_tuple_of(
            observation_payload["relic_counters"], lambda item: RelicCounterToken(**item)
        ),
        choice=CombatChoiceContext(
            active=observation_payload["choice"]["active"],
            reason=observation_payload["choice"]["reason"],
            min_picks=observation_payload["choice"]["min_picks"],
            max_picks=observation_payload["choice"]["max_picks"],
            selected=tuple(observation_payload["choice"]["selected"]),
            options=_tuple_of(
                observation_payload["choice"]["options"],
                lambda item: CombatChoiceOption(**item),
            ),
        ),
    )
    candidates = []
    for candidate in payload["legal_candidates"]:
        candidates.append(
            LegalActionCandidate(
                schema_version=candidate["schema_version"],
                dense_index=candidate["dense_index"],
                execution_id=candidate["execution_id"],
                action_kind=candidate["action_kind"],
                description=candidate["description"],
                card=(
                    CandidateCardFeatures(**candidate["card"])
                    if candidate["card"] is not None
                    else None
                ),
                target=(
                    CandidateTargetFeatures(**candidate["target"])
                    if candidate["target"] is not None
                    else None
                ),
                potion=(
                    CandidatePotionFeatures(**candidate["potion"])
                    if candidate["potion"] is not None
                    else None
                ),
                choice=(
                    CandidateChoiceFeatures(**candidate["choice"])
                    if candidate["choice"] is not None
                    else None
                ),
            )
        )
    return CombatTrainingState(
        schema_versions=schema_versions,
        context=CombatTrainingContext(**payload["context"]),
        observation=observation,
        legal_candidates=tuple(candidates),
    )


def parse_combat_puct_result(payload: Mapping[str, Any]) -> CombatPuctResult:
    return CombatPuctResult(
        chosen_action_id=payload.get("chosen_action_id"),
        root_action_ids=tuple(int(value) for value in payload.get("root_action_ids", ())),
        root_visits=tuple(int(value) for value in payload.get("root_visits", ())),
        root_visit_shares=tuple(float(value) for value in payload.get("root_visit_shares", ())),
        root_priors=tuple(float(value) for value in payload.get("root_priors", ())),
        frontier=_tuple_of(
            payload.get("frontier", []),
            lambda item: CombatPuctLine(
                line_index=int(item["line_index"]),
                action_prefix=tuple(int(value) for value in item.get("action_prefix", ())),
                visits=int(item.get("visits", 0)),
                visit_share=float(item.get("visit_share", 0.0)),
                prior=float(item.get("prior", 0.0)),
                expanded_nodes=int(item.get("expanded_nodes", 0)),
                elapsed_ms=int(item.get("elapsed_ms", 0)),
                outcome=CombatOutcomeVector(**item["outcome"]),
            ),
        ),
        root_outcome=CombatOutcomeVector(**payload["root_outcome"]),
        root_total_visits=int(payload.get("root_total_visits", 0)),
        stable_windows=int(payload.get("stable_windows", 0)),
        nodes_expanded=int(payload.get("nodes_expanded", 0)),
        leaf_evaluations=int(payload.get("leaf_evaluations", 0)),
        max_depth_reached=int(payload.get("max_depth_reached", 0)),
        elapsed_ms=int(payload.get("elapsed_ms", 0)),
        stop_reason=CombatSearchStopReason(payload["stop_reason"]),
    )
