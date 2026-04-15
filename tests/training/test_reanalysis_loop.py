from __future__ import annotations

from packages.training import (
    CombatInferenceService,
    CombatPreferenceExample,
    CombatSearchConfig,
    CombatSearchRequest,
    CombatStateSummary,
    LegalCombatCandidate,
    LinearCombatModel,
    OvernightReanalysisLoop,
)


def _example(idx: int, preferred_strength: float = 1.5, aggressive_strength: float = 3.0) -> CombatPreferenceExample:
    return CombatPreferenceExample(
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
        preferred_action_id=f"scale-{idx}",
    )


def test_reanalysis_loop_improves_accuracy_with_preference_updates() -> None:
    examples = [_example(idx) for idx in range(6)]
    model = LinearCombatModel(state_scale=0.0, candidate_scale=1.0, legal_bias=0.0, default_learning_rate=0.2)
    service = CombatInferenceService.build(model=model, config=CombatSearchConfig(top_k=3))
    loop = OvernightReanalysisLoop(service=service, learning_rate=0.2, batch_size=3)

    _, before = loop.run_epoch(examples, epoch_index=0, update=False)
    _, after = loop.run_epoch(examples, epoch_index=1, update=True)
    _, final = loop.run_epoch(examples, epoch_index=2, update=False)

    assert before.accuracy == 0.0
    assert after.updated_examples > 0
    assert final.accuracy == 1.0
    assert final.mean_frontier_size == 3.0


def test_reanalysis_loop_keeps_full_frontier_ordering() -> None:
    example = _example(0, preferred_strength=2.5, aggressive_strength=1.0)
    service = CombatInferenceService.build(
        LinearCombatModel(state_scale=0.0, candidate_scale=1.0, legal_bias=0.0),
        CombatSearchConfig(top_k=2),
    )
    loop = OvernightReanalysisLoop(service=service, learning_rate=0.1, batch_size=1)

    results, summary = loop.run_epoch([example], epoch_index=0, update=False)

    assert summary.accuracy == 1.0
    assert results[0].ranked_action_ids == (example.preferred_action_id, "attack-0")
    assert results[0].frontier_action_ids == (example.preferred_action_id, "attack-0", "end-0")
