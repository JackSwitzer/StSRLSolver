"""Adapters between the Rust training contract and the canonical policy/value model."""

from __future__ import annotations

from math import exp
from typing import Any, Mapping

from .combat_model import CombatStateSummary, LegalCombatCandidate
from .contracts import (
    CombatOutcomeVector,
    CombatPuctConfig,
    CombatSearchStopReason,
    CombatTrainingState,
    LegalActionCandidate,
    parse_combat_training_state,
)
from .inference_service import CombatInferenceService
from .shared_memory import CombatSearchRequest


def _sigmoid(value: float) -> float:
    return 1.0 / (1.0 + exp(-value))


def _enemy_pressure(state: CombatTrainingState) -> float:
    pressure = 0.0
    for enemy in state.observation.enemies:
        pressure += float(enemy.intent_damage * max(enemy.intent_hits, 1))
        pressure += float(enemy.intent_block) * 0.35
    return pressure


def _player_stance(state: CombatTrainingState) -> str:
    return state.observation.global_token.stance or state.observation.player.stance or "Neutral"


def _action_kind(candidate: LegalActionCandidate) -> str:
    return candidate.action_kind


def _target_hp(candidate: LegalActionCandidate) -> float:
    if candidate.target is None:
        return 0.0
    return float(candidate.target.hp + candidate.target.block)


def _candidate_features(
    state: CombatTrainingState,
    candidate: LegalActionCandidate,
) -> tuple[float, ...]:
    global_token = state.observation.global_token
    card = candidate.card
    target = candidate.target
    potion = candidate.potion
    choice = candidate.choice

    action_kind = _action_kind(candidate)
    card_cost = float(card.cost_for_turn) if card is not None else 0.0
    base_cost = float(card.base_cost) if card is not None else 0.0
    upgraded = 1.0 if card is not None and card.upgraded else 0.0
    x_cost = 1.0 if card is not None and card.x_cost else 0.0
    multi_hit = 1.0 if card is not None and card.multi_hit else 0.0
    target_hp = float(target.hp) if target is not None else 0.0
    target_block = float(target.block) if target is not None else 0.0
    targetable = 1.0 if target is not None and target.targetable else 0.0
    potion_flag = 1.0 if potion is not None else 0.0
    choice_flag = 1.0 if choice is not None else 0.0
    end_turn_flag = 1.0 if action_kind == "end_turn" else 0.0
    play_card_flag = 1.0 if action_kind == "play_card" else 0.0
    use_potion_flag = 1.0 if action_kind == "use_potion" else 0.0
    choose_flag = 1.0 if action_kind == "choose_option" else 0.0

    return (
        float(global_token.energy),
        float(global_token.hand_size),
        float(global_token.player_hp),
        float(global_token.player_block),
        card_cost,
        base_cost,
        upgraded,
        x_cost,
        multi_hit,
        target_hp,
        target_block,
        targetable,
        potion_flag,
        choice_flag,
        end_turn_flag + choose_flag,
        play_card_flag + use_potion_flag,
    )


def action_id_for_candidate(candidate: LegalActionCandidate) -> str:
    return str(candidate.execution_id)


def build_search_request_from_training_state(
    state: CombatTrainingState,
    *,
    request_id: str,
    metadata: Mapping[str, Any] | None = None,
) -> CombatSearchRequest:
    global_token = state.observation.global_token
    candidates = tuple(
        LegalCombatCandidate(
            action_id=action_id_for_candidate(candidate),
            action_type=candidate.action_kind,
            target_idx=(-1 if candidate.target is None else int(candidate.target.enemy_index)),
            features=_candidate_features(state, candidate),
            legal=True,
            legality_reason="legal",
            card_id=(None if candidate.card is None else candidate.card.card_id),
            potion_id=(None if candidate.potion is None else candidate.potion.potion_id),
            label=candidate.description,
        )
        for candidate in state.legal_candidates
    )
    return CombatSearchRequest(
        request_id=request_id,
        state=CombatStateSummary(
            combat_id=request_id,
            turn=int(global_token.turn),
            hp=int(global_token.player_hp),
            block=int(global_token.player_block),
            energy=int(global_token.energy),
            hand_size=int(global_token.hand_size),
            draw_pile_size=int(global_token.draw_pile_size),
            discard_pile_size=int(global_token.discard_pile_size),
            exhaust_pile_size=int(global_token.exhaust_pile_size),
            stance=_player_stance(state),
        ),
        candidates=candidates,
        metadata=dict(metadata or {}),
    )


def build_model_evaluator(
    service: CombatInferenceService,
    *,
    metadata_factory=None,
):
    """Return the Python callback expected by the Rust PUCT bridge."""

    def evaluator(payload: Mapping[str, Any]) -> dict[str, Any]:
        state = parse_combat_training_state(payload)
        metadata = {} if metadata_factory is None else dict(metadata_factory(state))
        request = build_search_request_from_training_state(
            state,
            request_id=metadata.get("request_id", f"runtime::{state.context.phase_label}::{state.context.floor}"),
            metadata=metadata,
        )
        result = service.choose_action(request)
        score_map = {
            action_id: score
            for action_id, score in zip(result.frontier_action_ids, result.frontier_scores)
        }
        priors = [max(0.0, float(score_map.get(candidate.action_id, 0.0))) for candidate in request.candidates]

        enemy_pressure = _enemy_pressure(state)
        total_enemy_hp = sum(
            float(enemy.hp + enemy.block) for enemy in state.observation.enemies if enemy.alive
        )
        best_score = float(result.chosen_score or 0.0)
        chosen_action = next(
            (candidate for candidate in request.candidates if candidate.action_id == result.chosen_action_id),
            None,
        )
        chosen_kind = None if chosen_action is None else chosen_action.action_type
        solve_probability = max(
            0.01,
            min(
                0.995,
                _sigmoid(
                    best_score * 0.4
                    + float(state.observation.global_token.player_hp) / 18.0
                    - enemy_pressure / 12.0
                    - total_enemy_hp / 120.0
                    + float(state.observation.global_token.energy) * 0.1
                ),
            ),
        )
        expected_hp_loss = max(
            0.0,
            enemy_pressure * 0.55
            + total_enemy_hp / 30.0
            - best_score * 0.75
            - float(state.observation.global_token.player_block) * 0.2
            - (2.0 if chosen_kind == "use_potion" else 0.0),
        )
        expected_turns = max(
            1.0,
            total_enemy_hp / 18.0
            + enemy_pressure / 20.0
            - best_score * 0.25
            + (0.6 if chosen_kind == "end_turn" else 0.0),
        )
        potion_cost = 1.0 if chosen_kind == "use_potion" else 0.0
        setup_value_delta = 1.0 if chosen_kind == "play_card" and chosen_action and "setup" in chosen_action.action_id else 0.0
        persistent_scaling_delta = 0.0
        if chosen_action and any(
            keyword in chosen_action.action_id.lower()
            for keyword in ("lesson", "ritual", "deus", "rushdown")
        ):
            persistent_scaling_delta = 0.4

        predicted_value = result.predicted_value
        if predicted_value is not None and any(
            abs(value) > 1e-6 for value in predicted_value.to_vector()[:6]
        ):
            outcome = CombatOutcomeVector(
                solve_probability=float(predicted_value.solve_probability),
                expected_hp_loss=float(predicted_value.expected_hp_loss),
                expected_turns=float(predicted_value.expected_turns),
                potion_cost=float(predicted_value.potion_spend_count),
                setup_value_delta=float(predicted_value.setup_delta),
                persistent_scaling_delta=float(predicted_value.persistent_scaling_delta),
            )
        else:
            outcome = CombatOutcomeVector(
                solve_probability=float(solve_probability),
                expected_hp_loss=float(expected_hp_loss),
                expected_turns=float(expected_turns),
                potion_cost=float(potion_cost),
                setup_value_delta=float(setup_value_delta),
                persistent_scaling_delta=float(persistent_scaling_delta),
            )
        return {
            "priors": priors,
            "outcome": {
                "solve_probability": outcome.solve_probability,
                "expected_hp_loss": outcome.expected_hp_loss,
                "expected_turns": outcome.expected_turns,
                "potion_cost": outcome.potion_cost,
                "setup_value_delta": outcome.setup_value_delta,
                "persistent_scaling_delta": outcome.persistent_scaling_delta,
            },
        }

    return evaluator


def should_promote_collection_result(
    *,
    stop_reason: CombatSearchStopReason,
    root_visit_shares: tuple[float, ...],
    root_outcome: CombatOutcomeVector,
    room_kind: str,
) -> bool:
    if stop_reason is not CombatSearchStopReason.CONVERGED:
        return True
    if not root_visit_shares:
        return True
    ordered = sorted(root_visit_shares, reverse=True)
    if len(ordered) >= 2 and (ordered[0] - ordered[1]) < 0.08:
        return True
    if root_outcome.solve_probability < (0.92 if room_kind == "boss" else 0.8):
        return True
    return False
