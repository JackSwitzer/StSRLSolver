"""
Tests for StrategicPlanner.

Tests cover:
- StrategicPlanner rest vs smith
- StrategicPlanner path choice
- StrategicPlanner card pick
- StrategicPlanner state evaluation
"""

import pytest
from packages.training.planner import StrategicPlanner


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


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
