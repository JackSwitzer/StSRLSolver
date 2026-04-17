from __future__ import annotations

import math

from packages.training import CombatPuctTargetExample, CombatSearchRequest, CombatStateSummary, CombatValueTarget, LegalCombatCandidate
from packages.training.shared_memory import CombatSharedMemoryBatcher
from packages.training.value_targets import PHASE1_VALUE_HEAD_NAMES


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


def test_puct_target_example_prefers_visit_counts_and_packs_multi_head_values() -> None:
    request = _combat_request()
    target = CombatPuctTargetExample(
        request=request,
        policy_action_ids=("attack", "block", "setup"),
        policy_scores=(2.0, 1.0, 0.0),
        value_target=CombatValueTarget(
            solve_probability=0.6,
            expected_hp_loss=3.0,
            expected_turns=4.0,
            potion_spend_count=0.5,
            setup_delta=0.25,
            persistent_scaling_delta=0.0,
            potion_spend_by_id={"FlexPotion": 0.5},
        ),
        chosen_action_id="attack",
        visit_counts=(6, 3, 1),
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
    assert batch.value_target_names == PHASE1_VALUE_HEAD_NAMES
    assert batch.sample_weights.tolist() == [2.0]
    flex_index = batch.value_target_names.index("potion::FlexPotion")
    assert round(float(batch.value_target_matrix[0, flex_index]), 6) == 0.5
