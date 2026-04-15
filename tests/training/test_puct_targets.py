from __future__ import annotations

import math

from packages.training.combat_model import LinearCombatModel
from packages.training.inference_service import CombatInferenceService, CombatPreferenceExample, CombatSearchConfig
from packages.training.shared_memory import CombatPuctTargetExample, CombatSearchRequest, CombatSharedMemoryBatcher, CombatStateSummary, LegalCombatCandidate


def _combat_request() -> CombatSearchRequest:
    return CombatSearchRequest(
        request_id="combat-puct-1",
        state=CombatStateSummary(
            combat_id="combat-puct-1",
            turn=3,
            hp=41,
            block=5,
            energy=2,
            hand_size=4,
            draw_pile_size=11,
            discard_pile_size=3,
            exhaust_pile_size=0,
            stance="Calm",
        ),
        candidates=(
            LegalCombatCandidate("attack", "card", features=(1.2, 0.2), legal=True),
            LegalCombatCandidate("block", "card", features=(0.8, 0.7), legal=True),
            LegalCombatCandidate("setup", "card", features=(0.1, 0.1), legal=True),
        ),
    )


def test_puct_target_example_prefers_visit_counts_and_packs_cleanly() -> None:
    request = _combat_request()
    target = CombatPuctTargetExample(
        request=request,
        policy_action_ids=("attack", "block", "setup"),
        policy_scores=(2.0, 1.0, 0.0),
        value_target=0.5,
        chosen_action_id="attack",
        visit_counts=(6, 3, 1),
        temperature=1.0,
        sample_weight=2.0,
    )

    distribution = target.policy_distribution()
    batch = CombatSharedMemoryBatcher(max_batch_size=4).pack_puct_targets((target,))

    assert math.isclose(sum(distribution), 1.0, rel_tol=1e-6, abs_tol=1e-6)
    assert tuple(round(value, 6) for value in distribution) == (0.6, 0.3, 0.1)
    assert batch.request_ids == ("combat-puct-1",)
    assert batch.target_action_ids == (("attack", "block", "setup"),)
    assert tuple(round(value, 6) for value in batch.policy_target_matrix[0].tolist()) == (0.6, 0.3, 0.1)
    assert batch.policy_target_mask.tolist() == [[True, True, True]]
    assert batch.chosen_action_indices.tolist() == [0]
    assert batch.value_targets.tolist() == [0.5]
    assert batch.sample_weights.tolist() == [2.0]


def test_inference_service_builds_puct_targets_from_preference_examples() -> None:
    service = CombatInferenceService.build(
        LinearCombatModel(state_scale=0.0, candidate_scale=1.0, legal_bias=0.0),
        CombatSearchConfig(top_k=3, puct_target_temperature=0.5),
    )
    example = CombatPreferenceExample(
        request=_combat_request(),
        preferred_action_id="block",
        weight=1.5,
    )

    puct_examples = service.build_puct_target_examples((example,), value_targets=(0.75,))
    puct_batch = service.build_puct_target_batch((example,), value_targets=(0.75,))

    assert service.batcher.max_batch_size == 128
    assert len(puct_examples) == 1
    assert puct_examples[0].chosen_action_id == "block"
    assert puct_examples[0].sample_weight == 1.5
    assert puct_examples[0].value_target == 0.75
    assert puct_examples[0].policy_action_ids == ("block", "attack", "setup")
    assert math.isclose(sum(puct_examples[0].policy_distribution()), 1.0, rel_tol=1e-6, abs_tol=1e-6)
    assert puct_batch.value_targets.tolist() == [0.75]
    assert puct_batch.sample_weights.tolist() == [1.5]
    assert puct_batch.policy_target_matrix.shape == (1, 3)
