"""
Agent-readiness tests for action/observation/map specs.

These tests validate the JSON action/observation layer and readiness gates.
"""

import sys
import pytest

sys.path.insert(0, "/Users/jackswitzer/Desktop/SlayTheSpireRL")

from packages.engine.game import GameRunner, GamePhase
from packages.engine.handlers.reward_handler import RewardHandler, PotionReward
from packages.engine.content.potions import get_potion_by_id


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
