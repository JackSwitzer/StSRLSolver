"""
Agent-readiness tests for action/observation/map specs.

These tests validate the JSON action/observation layer and readiness gates.
"""

import sys
import pytest
import numpy as np

sys.path.insert(0, "/Users/jackswitzer/Desktop/SlayTheSpireRL")

from packages.engine.game import GameRunner, GamePhase
from packages.engine.handlers.reward_handler import RewardHandler, PotionReward
from packages.engine.content.potions import get_potion_by_id
from packages.engine.rl_masks import ActionSpace


def _make_runner_with_rewards(seed="REWARD1", room_type="monster"):
    runner = GameRunner(seed=seed, ascension=0, verbose=False)
    runner.phase = GamePhase.COMBAT_REWARDS
    rewards = RewardHandler.generate_combat_rewards(
        run_state=runner.run_state,
        room_type=room_type,
        card_rng=runner.card_rng,
        treasure_rng=runner.treasure_rng,
        potion_rng=runner.potion_rng,
        relic_rng=runner.relic_rng,
    )
    runner.current_rewards = rewards
    return runner, rewards


def _make_runner_with_boss_rewards(seed="BOSSREWARD1"):
    runner = GameRunner(seed=seed, ascension=0, verbose=False)
    runner.phase = GamePhase.BOSS_REWARDS
    rewards = RewardHandler.generate_boss_rewards(
        run_state=runner.run_state,
        card_rng=runner.card_rng,
        treasure_rng=runner.treasure_rng,
        potion_rng=runner.potion_rng,
        relic_rng=runner.relic_rng,
    )
    runner.current_rewards = rewards
    return runner, rewards


def test_observation_top_level_schema():
    runner = GameRunner(seed="OBS1", ascension=0, verbose=False)
    obs = runner.get_observation()
    assert isinstance(obs, dict)
    for key in ("phase", "run", "map", "combat", "event", "reward", "shop", "rest", "treasure"):
        assert key in obs


def test_observation_includes_map_visibility():
    runner = GameRunner(seed="MAPOBS1", ascension=0, verbose=False)
    runner.run_state.generate_map_for_act(1)
    obs = runner.get_observation()
    map_obs = obs.get("map", {})
    assert "nodes" in map_obs
    assert "edges" in map_obs
    assert "available_paths" in map_obs
    assert "visited_nodes" in map_obs


def test_action_dict_min_fields():
    runner = GameRunner(seed="ACT1", ascension=0, verbose=False)
    actions = runner.get_available_action_dicts()
    assert len(actions) > 0
    for action in actions:
        assert "id" in action
        assert "type" in action
        assert "params" in action
        assert "phase" in action


def test_action_ids_deterministic_for_identical_state():
    runner1 = GameRunner(seed="DET1", ascension=0, verbose=False)
    runner2 = GameRunner(seed="DET1", ascension=0, verbose=False)
    actions1 = runner1.get_available_action_dicts()
    actions2 = runner2.get_available_action_dicts()
    ids1 = [a["id"] for a in actions1]
    ids2 = [a["id"] for a in actions2]
    assert ids1 == ids2


def test_path_choice_actions_align_to_available_paths():
    runner = GameRunner(seed="MAP1", ascension=0, verbose=False)
    runner.run_state.generate_map_for_act(1)
    runner.phase = GamePhase.MAP_NAVIGATION
    obs = runner.get_observation()
    actions = runner.get_available_action_dicts()
    path_actions = [a for a in actions if a.get("type") == "path_choice"]
    assert len(path_actions) > 0
    available_paths = obs["map"]["available_paths"]
    assert len(path_actions) == len(available_paths)
    for action in path_actions:
        idx = action["params"]["node_index"]
        assert 0 <= idx < len(available_paths)


def test_reward_actions_include_pick_and_skip_card():
    runner, rewards = _make_runner_with_rewards(seed="REWARD2")
    assert len(rewards.card_rewards) > 0
    actions = runner.get_available_action_dicts()
    pick_actions = [a for a in actions if a["type"] == "pick_card"]
    skip_actions = [a for a in actions if a["type"] == "skip_card"]
    assert len(pick_actions) > 0
    for i, card_reward in enumerate(rewards.card_rewards):
        assert any(a["params"]["card_reward_index"] == i for a in pick_actions)
        assert any(a["params"]["card_reward_index"] == i for a in skip_actions)
        for j in range(len(card_reward.cards)):
            assert any(
                a["params"]["card_reward_index"] == i and a["params"]["card_index"] == j
                for a in pick_actions
            )


def test_reward_actions_include_potion_claim_and_skip():
    runner, rewards = _make_runner_with_rewards(seed="REWARD3")
    if rewards.potion is None:
        rewards.potion = PotionReward(get_potion_by_id("Fire Potion"))
    actions = runner.get_available_action_dicts()
    assert any(a["type"] == "claim_potion" for a in actions)
    assert any(a["type"] == "skip_potion" for a in actions)


def test_skip_boss_relic_json_advances_without_relic():
    runner, rewards = _make_runner_with_boss_rewards(seed="BOSSREWARD2")
    assert rewards.boss_relics is not None
    runner.run_state.act = 1
    before_relics = len(runner.run_state.relics)
    actions = runner.get_available_action_dicts()
    skip_action = next((a for a in actions if a["type"] == "skip_boss_relic"), None)
    assert skip_action is not None
    runner.take_action_dict(skip_action)
    assert len(runner.run_state.relics) == before_relics
    assert runner.run_state.act == 2
    assert runner.phase == GamePhase.MAP_NAVIGATION


# =============================================================================
# RL-ACT-001 Mask and ID Stability Tests
# =============================================================================


def test_action_id_stability_multi_step():
    """Action IDs remain identical when replaying same actions on same seed."""
    runner1 = GameRunner(seed="STAB1", ascension=0, verbose=False)
    runner2 = GameRunner(seed="STAB1", ascension=0, verbose=False)

    for step in range(25):
        if runner1.game_over or runner2.game_over:
            break
        a1 = runner1.get_available_action_dicts()
        a2 = runner2.get_available_action_dicts()
        ids1 = [a["id"] for a in a1]
        ids2 = [a["id"] for a in a2]
        assert ids1 == ids2, f"Action IDs diverged at step {step}: {ids1} vs {ids2}"
        runner1.take_action_dict(a1[0])
        runner2.take_action_dict(a2[0])


def test_action_ordering_is_stable():
    """Action list ordering is stable across repeated calls on same state."""
    runner = GameRunner(seed="ORD1", ascension=0, verbose=False)
    baseline = [a["id"] for a in runner.get_available_action_dicts()]
    for _ in range(10):
        current = [a["id"] for a in runner.get_available_action_dicts()]
        assert current == baseline


def test_mask_round_trip_preserves_actions():
    """actions -> mask -> filtered_actions preserves the full legal set."""
    runner = GameRunner(seed="MASK_RT", ascension=0, verbose=False)
    space = ActionSpace()

    for _ in range(20):
        if runner.game_over:
            break
        actions = runner.get_available_action_dicts()
        mask = space.actions_to_mask(actions)
        filtered = space.mask_to_actions(mask, actions)
        assert len(filtered) == len(actions)
        assert {a["id"] for a in filtered} == {a["id"] for a in actions}
        runner.take_action_dict(actions[0])


def test_mask_size_monotonic():
    """Action space size never decreases as new actions are observed."""
    runner = GameRunner(seed="MASK_MONO", ascension=0, verbose=False)
    space = ActionSpace()
    prev_size = 0

    for _ in range(30):
        if runner.game_over:
            break
        actions = runner.get_available_action_dicts()
        space.actions_to_mask(actions)
        assert space.size >= prev_size
        prev_size = space.size
        runner.take_action_dict(actions[0])


def test_invalid_action_rejected_not_silent():
    """Invalid action dict returns explicit error, never silent corruption."""
    runner = GameRunner(seed="INV1", ascension=0, verbose=False)
    initial_floor = runner.run_state.floor
    initial_hp = runner.run_state.current_hp

    result = runner.take_action_dict({
        "type": "nonexistent_action",
        "params": {},
    })

    assert result.get("success") is False
    assert "error" in result
    assert isinstance(result["error"], str)
    # State should not be corrupted
    assert runner.run_state.floor == initial_floor
    assert runner.run_state.current_hp == initial_hp


def test_action_types_catalog():
    """ActionSpace.action_types contains the expected known types."""
    space = ActionSpace()
    types = space.action_types
    assert "play_card" in types
    assert "end_turn" in types
    assert "path_choice" in types
    assert "select_cards" in types
    assert "select_stance" in types
