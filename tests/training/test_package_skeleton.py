from __future__ import annotations

from packages.training import (
    CombatInferenceService,
    CombatSearchConfig,
    CombatSearchRequest,
    CombatSharedMemoryBatcher,
    CombatStateSummary,
    LegalCombatCandidate,
    LinearCombatModel,
    SharedMemoryConfig,
    TrainingConfig,
)


def test_training_config_defaults_are_combat_first() -> None:
    config = TrainingConfig()

    assert config.model_backend == "mlx"
    assert config.shared_memory.max_batch_size == 128
    assert config.shared_memory.max_candidates_per_request == 64
    assert config.combat_search.top_k == 4
    assert config.combat_search.require_legal_candidates is True
    assert config.combat_search.puct_target_temperature == 1.0


def test_training_stack_defaults_are_scaled_for_puct_topology() -> None:
    from packages.training import TrainingStackConfig

    stack = TrainingStackConfig()

    assert stack.topology.actor_workers == 12
    assert stack.topology.inference_workers == 2
    assert stack.topology.target_memory_gb == 19.0
    assert stack.shared_memory.max_batch_size == 128
    assert stack.shared_memory.max_candidates_per_request == 64
    assert stack.search.root_simulations == 384
    assert stack.search.frontier_capacity == 12
    assert stack.model.d_model == 256
    assert stack.model.token_dim == 256
    assert stack.model.trunk_layers == 8
    assert stack.model.attention_heads == 8


def test_shared_memory_batcher_packs_legal_candidates() -> None:
    batcher = CombatSharedMemoryBatcher.from_config(SharedMemoryConfig(max_batch_size=4))
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


def test_inference_service_scores_only_legal_candidates_and_preserves_frontier() -> None:
    service = CombatInferenceService.build(
        LinearCombatModel(state_scale=0.0, candidate_scale=1.0, legal_bias=0.0),
        CombatSearchConfig(top_k=2),
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
            LegalCombatCandidate("setup", "card", features=(0.25,), legal=True),
        ),
    )

    result = service.choose_action(request)

    assert result.request_id == "combat-2"
    assert result.chosen_action_id == "attack"
    assert result.ranked_action_ids == ("attack", "defend")
    assert result.frontier_action_ids == ("attack", "defend", "setup")
    assert result.ranked_scores[0] > result.ranked_scores[1]
