"""
Tests for CombatMCTS, StrategicPlanner, and StSAgent.

Tests cover:
- CombatMCTS returns valid actions
- CombatMCTS deterministic with seed
- CombatMCTS prefers lethal
- StrategicPlanner rest vs smith
- StSAgent plays full combat
- Planner only returns legal actions
"""

import pytest
from packages.engine.combat_engine import (
    CombatEngine,
    CombatPhase,
    create_simple_combat,
)
from packages.engine.state.combat import PlayCard, EndTurn, UsePotion
from packages.training.mcts import CombatMCTS, MCTSNode
from packages.training.planner import StrategicPlanner, StSAgent


# =========================================================================
# Helpers
# =========================================================================

def _make_combat(enemy_hp: int = 40, enemy_damage: int = 6, player_hp: int = 80, deck=None) -> CombatEngine:
    """Create and start a simple combat for testing."""
    engine = create_simple_combat(
        enemy_id="TestEnemy",
        enemy_hp=enemy_hp,
        enemy_damage=enemy_damage,
        player_hp=player_hp,
        deck=deck,
    )
    engine.start_combat()
    return engine


class MockRunState:
    """Minimal mock of RunState for StrategicPlanner tests."""
    def __init__(self, current_hp=50, max_hp=80, floor=5, act=1, gold=100,
                 deck=None, relics=None):
        self.current_hp = current_hp
        self.max_hp = max_hp
        self.floor = floor
        self.act = act
        self.gold = gold
        self.deck = deck or ["Strike_P"] * 5 + ["Defend_P"] * 5
        self.relics = relics or []

    def has_relic(self, name):
        return name in self.relics


class MockRunner:
    """Minimal mock of GameRunner for StrategicPlanner tests."""
    def __init__(self, **kwargs):
        self.run_state = MockRunState(**kwargs)


# =========================================================================
# CombatMCTS Tests
# =========================================================================

class TestCombatMCTSReturnsValidActions:
    """CombatMCTS.search should only return actions from get_legal_actions."""

    def test_returns_dict_of_actions(self):
        engine = _make_combat(enemy_hp=40)
        mcts = CombatMCTS(num_simulations=8)
        result = mcts.search(engine)

        assert isinstance(result, dict)
        assert len(result) > 0

        # Every key should be a legal action type
        for action in result:
            assert isinstance(action, (PlayCard, EndTurn, UsePotion))

    def test_probabilities_sum_to_one(self):
        engine = _make_combat(enemy_hp=40)
        mcts = CombatMCTS(num_simulations=16)
        result = mcts.search(engine)

        total = sum(result.values())
        assert abs(total - 1.0) < 1e-6, f"Probabilities sum to {total}, expected 1.0"

    def test_all_actions_are_legal(self):
        engine = _make_combat(enemy_hp=40)
        legal = set()
        for a in engine.get_legal_actions():
            legal.add(type(a).__name__)

        mcts = CombatMCTS(num_simulations=8)
        result = mcts.search(engine)

        for action in result:
            assert type(action).__name__ in legal


class TestCombatMCTSDeterministic:
    """With a fixed seed, CombatMCTS should produce consistent results."""

    def test_deterministic_with_seed(self):
        import numpy as np

        results = []
        for _ in range(2):
            np.random.seed(42)
            engine = _make_combat(enemy_hp=20, enemy_damage=3)
            mcts = CombatMCTS(num_simulations=16)
            result = mcts.search(engine)
            # Convert to sorted list of (action_repr, prob) for comparison
            items = sorted(
                [(repr(a), p) for a, p in result.items()],
                key=lambda x: x[0],
            )
            results.append(items)

        # Same seed should give same results
        assert len(results[0]) == len(results[1])
        for (a1, p1), (a2, p2) in zip(results[0], results[1]):
            assert a1 == a2
            assert abs(p1 - p2) < 1e-6


class TestCombatMCTSPrefersLethal:
    """When lethal is available, MCTS should strongly prefer kill actions."""

    def test_prefers_attack_when_enemy_low_hp(self):
        # Enemy at 5 HP, single Strike (6 dmg) is lethal
        engine = _make_combat(enemy_hp=5, enemy_damage=10, player_hp=80)
        mcts = CombatMCTS(num_simulations=32)
        result = mcts.search(engine)

        # Find the PlayCard actions
        play_card_prob = sum(
            prob for action, prob in result.items()
            if isinstance(action, PlayCard)
        )
        end_turn_prob = sum(
            prob for action, prob in result.items()
            if isinstance(action, EndTurn)
        )

        # Should strongly prefer playing cards over ending turn
        assert play_card_prob > end_turn_prob, (
            f"PlayCard prob {play_card_prob} should exceed EndTurn prob {end_turn_prob}"
        )


class TestCombatMCTSSelectAction:
    """Test action selection from visit distributions."""

    def test_greedy_selects_highest(self):
        mcts = CombatMCTS(num_simulations=8)
        probs = {
            PlayCard(card_idx=0, target_idx=0): 0.7,
            PlayCard(card_idx=1, target_idx=0): 0.2,
            EndTurn(): 0.1,
        }
        best = mcts.select_action(probs, temperature=0.0)
        assert best == PlayCard(card_idx=0, target_idx=0)

    def test_empty_probs_raises(self):
        mcts = CombatMCTS(num_simulations=8)
        with pytest.raises(ValueError):
            mcts.select_action({}, temperature=0.0)


# =========================================================================
# StrategicPlanner Tests
# =========================================================================

class TestStrategicPlannerRestVsSmith:
    """Planner should prefer rest when HP is low and upgrade when healthy."""

    def test_prefers_rest_when_low_hp(self):
        planner = StrategicPlanner()
        runner = MockRunner(current_hp=20, max_hp=80)  # 25% HP

        from packages.engine.game import RestAction
        options = [
            RestAction(action_type="rest"),
            RestAction(action_type="upgrade", card_index=0),
        ]
        idx = planner.plan_rest_site(runner, options)
        assert options[idx].action_type == "rest"

    def test_prefers_upgrade_when_healthy(self):
        planner = StrategicPlanner()
        runner = MockRunner(current_hp=75, max_hp=80)  # 94% HP

        from packages.engine.game import RestAction
        options = [
            RestAction(action_type="rest"),
            RestAction(action_type="upgrade", card_index=0),
        ]
        idx = planner.plan_rest_site(runner, options)
        assert options[idx].action_type == "upgrade"


class TestStrategicPlannerPathChoice:
    """Planner should choose paths sensibly based on HP."""

    def test_prefers_rest_site_when_low_hp(self):
        planner = StrategicPlanner()
        runner = MockRunner(current_hp=15, max_hp=80)

        paths = [
            {"room_type": "monster"},
            {"room_type": "rest"},
        ]
        idx = planner.plan_path_choice(runner, paths)
        assert paths[idx]["room_type"] == "rest"

    def test_avoids_elite_when_low_hp(self):
        planner = StrategicPlanner()
        runner = MockRunner(current_hp=15, max_hp=80)

        paths = [
            {"room_type": "elite"},
            {"room_type": "event"},
        ]
        idx = planner.plan_path_choice(runner, paths)
        assert paths[idx]["room_type"] == "event"


class TestStrategicPlannerCardPick:
    """Planner should prefer high-tier Watcher cards."""

    def test_prefers_tier1_card(self):
        planner = StrategicPlanner()
        runner = MockRunner()

        cards = ["Strike_P", "Rushdown", "Defend_P"]
        idx = planner.plan_card_pick(runner, cards)
        assert cards[idx] == "Rushdown"

    def test_skip_when_deck_bloated(self):
        planner = StrategicPlanner()
        runner = MockRunner(deck=["Strike_P"] * 40)

        cards = ["Strike_P", "Defend_P"]  # low-value cards
        idx = planner.plan_card_pick(runner, cards)
        # Should skip (index == len(cards))
        assert idx == len(cards)


class TestStrategicPlannerEvaluate:
    """State evaluation should return values in [0, 1]."""

    def test_evaluate_returns_bounded_value(self):
        planner = StrategicPlanner()
        runner = MockRunner(current_hp=50, max_hp=80, floor=10)
        value = planner.evaluate_state(runner)
        assert 0.0 <= value <= 1.0

    def test_higher_hp_gives_higher_value(self):
        planner = StrategicPlanner()
        low_hp = MockRunner(current_hp=10, max_hp=80)
        high_hp = MockRunner(current_hp=70, max_hp=80)
        assert planner.evaluate_state(high_hp) > planner.evaluate_state(low_hp)


# =========================================================================
# StSAgent Tests
# =========================================================================

class TestAgentPlaysFullCombat:
    """StSAgent should be able to play through an entire combat."""

    def test_plays_full_combat_to_completion(self):
        engine = _make_combat(enemy_hp=20, enemy_damage=3, player_hp=80)
        mcts = CombatMCTS(num_simulations=8)

        max_actions = 200
        action_count = 0

        while not engine.is_combat_over() and action_count < max_actions:
            result = mcts.search(engine)
            if not result:
                break
            action = mcts.select_action(result, temperature=0.0)
            engine.execute_action(action)
            action_count += 1

        assert engine.is_combat_over(), (
            f"Combat did not finish after {action_count} actions"
        )

    def test_wins_easy_combat(self):
        """Agent should win against a weak enemy."""
        engine = _make_combat(enemy_hp=15, enemy_damage=2, player_hp=80)
        mcts = CombatMCTS(num_simulations=16)

        max_actions = 200
        action_count = 0

        while not engine.is_combat_over() and action_count < max_actions:
            result = mcts.search(engine)
            if not result:
                break
            action = mcts.select_action(result, temperature=0.0)
            engine.execute_action(action)
            action_count += 1

        assert engine.is_combat_over()
        assert engine.is_victory(), "Agent should win against a weak enemy"


# =========================================================================
# Legal-actions-only test
# =========================================================================

class TestPlannerLegalActionsOnly:
    """All returned actions must be from the legal set."""

    def test_mcts_only_returns_engine_legal_actions(self):
        engine = _make_combat(enemy_hp=40)
        legal_actions = engine.get_legal_actions()
        legal_set = {repr(a) for a in legal_actions}

        mcts = CombatMCTS(num_simulations=8)
        result = mcts.search(engine)

        for action in result:
            assert repr(action) in legal_set, (
                f"Action {repr(action)} not in legal set"
            )

    def test_select_action_always_in_search_result(self):
        engine = _make_combat(enemy_hp=30)
        mcts = CombatMCTS(num_simulations=8)
        result = mcts.search(engine)
        action = mcts.select_action(result, temperature=0.0)
        assert action in result


class TestMCTSNodeProperties:
    """Basic MCTSNode property tests."""

    def test_value_zero_when_no_visits(self):
        node = MCTSNode(state=None)
        assert node.value == 0.0

    def test_value_is_mean(self):
        node = MCTSNode(state=None)
        node.visits = 4
        node.value_sum = 2.0
        assert node.value == 0.5

    def test_is_expanded(self):
        parent = MCTSNode(state=None)
        assert not parent.is_expanded
        parent.children["a"] = MCTSNode(state=None)
        assert parent.is_expanded


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
