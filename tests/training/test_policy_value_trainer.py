from __future__ import annotations

from packages.training import (
    CombatInferenceService,
    CombatPolicyValueTrainer,
    CombatPuctTargetExample,
    CombatSearchConfig,
    CombatSearchRequest,
    CombatStateSummary,
    CombatValueTarget,
    LegalCombatCandidate,
    MLXCombatModel,
)


def _example(idx: int, *, preferred_strength: float = 1.5, aggressive_strength: float = 3.0) -> CombatPuctTargetExample:
    return CombatPuctTargetExample(
        request=CombatSearchRequest(
            request_id=f"request-{idx}",
            state=CombatStateSummary(
                combat_id=f"combat-{idx}",
                turn=1,
                hp=50,
                block=0,
                energy=3,
                hand_size=4,
                draw_pile_size=12,
                discard_pile_size=3,
                exhaust_pile_size=0,
            ),
            candidates=(
                LegalCombatCandidate(f"attack-{idx}", "card", features=(aggressive_strength, 0.0), legal=True),
                LegalCombatCandidate(f"scale-{idx}", "card", features=(0.0, preferred_strength), legal=True),
                LegalCombatCandidate(f"end-{idx}", "end_turn", features=(0.1, 0.1), legal=True),
            ),
        ),
        policy_action_ids=(f"scale-{idx}", f"attack-{idx}", f"end-{idx}"),
        policy_scores=(0.7, 0.2, 0.1),
        visit_counts=(7, 2, 1),
        chosen_action_id=f"scale-{idx}",
        value_target=CombatValueTarget(
            solve_probability=0.95,
            expected_hp_loss=2.0,
            expected_turns=3.0,
            potion_spend_count=0.0,
            setup_delta=0.4,
            persistent_scaling_delta=0.0,
        ),
    )


def test_policy_value_trainer_improves_chosen_action_mass() -> None:
    examples = [_example(idx) for idx in range(8)]
    model = MLXCombatModel(candidate_scale=1.0, legal_bias=0.0, default_learning_rate=0.2)
    service = CombatInferenceService.build(model=model, config=CombatSearchConfig(top_k=3))
    trainer = CombatPolicyValueTrainer(service=service, learning_rate=0.2, batch_size=4)

    before = trainer.run_epoch(examples, epoch_index=0, update=False)
    trainer.run_epoch(examples, epoch_index=1, update=True)
    after = trainer.run_epoch(examples, epoch_index=2, update=False)

    assert before.mean_chosen_action_mass < after.mean_chosen_action_mass
    assert after.policy_loss < before.policy_loss
    assert after.mean_frontier_size == 3.0


def test_inference_service_exposes_multi_head_value_predictions() -> None:
    service = CombatInferenceService.build(
        MLXCombatModel(candidate_scale=1.0, legal_bias=0.0),
        CombatSearchConfig(top_k=3),
    )
    result = service.choose_action(_example(0).request)

    assert result.frontier_action_ids
    assert result.predicted_value is not None
    assert isinstance(result.predicted_value.solve_probability, float)
