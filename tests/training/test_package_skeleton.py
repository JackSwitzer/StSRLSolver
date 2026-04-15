from __future__ import annotations

from packages.training import (
    CombatInferenceService,
    CombatSearchConfig,
    CombatSearchRequest,
    CombatSharedMemoryBatcher,
    CombatStateSummary,
    LegalCombatCandidate,
    LinearCombatModel,
    TrainingConfig,
)


def test_training_config_defaults_are_combat_first() -> None:
    config = TrainingConfig()

    assert config.model_backend == "mlx"
    assert config.shared_memory.max_batch_size == 64
    assert config.combat_search.top_k == 4
    assert config.combat_search.require_legal_candidates is True


def test_shared_memory_batcher_packs_legal_candidates() -> None:
    batcher = CombatSharedMemoryBatcher(max_batch_size=4)
    request = CombatSearchRequest(
        request_id="combat-1",
        state=CombatStateSummary(
            combat_id="combat-1",
            turn=2,
            hp=42,
            block=7,
            energy=3,
            hand_size=5,
            draw_pile_size=20,
            discard_pile_size=4,
            exhaust_pile_size=1,
            stance="Wrath",
        ),
        candidates=(
            LegalCombatCandidate("play-bash", "card", target_idx=0, features=(0.2, 1.0), legal=True),
            LegalCombatCandidate("use-potion", "potion", target_idx=0, features=(0.9,), legal=False),
        ),
    )

    packed = batcher.pack((request,))

    assert packed.request_ids == ("combat-1",)
    assert packed.state_matrix.shape == (1, 9)
    assert packed.candidate_matrix.shape == (1, 2, 2)
    assert packed.legal_mask.tolist() == [[True, False]]
    assert packed.candidate_ids == (("play-bash", "use-potion"),)


def test_inference_service_scores_only_legal_candidates() -> None:
    service = CombatInferenceService.build(
        LinearCombatModel(state_scale=0.0, candidate_scale=1.0, legal_bias=0.0),
        CombatSearchConfig(top_k=4),
    )
    request = CombatSearchRequest(
        request_id="combat-2",
        state=CombatStateSummary(
            combat_id="combat-2",
            turn=1,
            hp=30,
            block=0,
            energy=2,
            hand_size=3,
            draw_pile_size=10,
            discard_pile_size=8,
            exhaust_pile_size=0,
        ),
        candidates=(
            LegalCombatCandidate("illegal", "card", features=(100.0,), legal=False),
            LegalCombatCandidate("attack", "card", features=(1.0,), legal=True),
            LegalCombatCandidate("defend", "card", features=(0.5,), legal=True),
        ),
    )

    result = service.choose_action(request)

    assert result.request_id == "combat-2"
    assert result.chosen_action_id == "attack"
    assert result.ranked_action_ids == ("attack", "defend")
    assert result.ranked_scores[0] > result.ranked_scores[1]

