"""
Tests for Agent API - JSON-serializable action and observation interfaces.

Tests cover:
1. Action dict generation for each phase
2. Action execution with valid/invalid params
3. Observation schema completeness
4. Phase transitions
5. Determinism (same seed + actions = same results)
"""

import pytest
import json
from typing import List, Dict, Any

from packages.engine import (
    GameRunner, GamePhase,
    ActionDict, ActionResult, ObservationDict,
)
from packages.engine.combat_engine import CombatEngine
from packages.engine.content.relics import get_relic
from packages.engine.handlers.event_handler import EventPhase, EventState
from packages.engine.handlers.reward_handler import CombatRewards, BossRelicChoices
from packages.engine.handlers.reward_handler import RelicReward
from packages.engine.state.combat import create_combat, create_enemy


# =============================================================================
# Fixtures
# =============================================================================

@pytest.fixture
def runner():
    """Create a fresh GameRunner for testing."""
    return GameRunner(seed="AGENTTEST", ascension=20, verbose=False)


@pytest.fixture
def runner_neow():
    """Create a GameRunner at Neow phase."""
    return GameRunner(seed="NEOWTEST", ascension=20, skip_neow=False, verbose=False)


# =============================================================================
# Action Dict Generation Tests
# =============================================================================

class TestActionDictGeneration:
    """Test get_available_action_dicts() for each phase."""

    def test_map_navigation_actions(self, runner):
        """Test path_choice action generation."""
        assert runner.phase == GamePhase.MAP_NAVIGATION

        actions = runner.get_available_action_dicts()

        assert len(actions) > 0, "Should have at least one path choice"

        for action in actions:
            assert "id" in action
            assert "type" in action
            assert action["type"] == "path_choice"
            assert "params" in action
            assert "node_index" in action["params"]
            assert "phase" in action
            assert action["phase"] == "map"

    def test_neow_actions(self, runner_neow):
        """Test neow_choice action generation."""
        assert runner_neow.phase == GamePhase.NEOW

        actions = runner_neow.get_available_action_dicts()

        assert len(actions) == 4, "Neow should offer 4 choices"

        for i, action in enumerate(actions):
            assert action["type"] == "neow_choice"
            assert action["params"]["choice_index"] == i
            assert action["phase"] == "neow"

    def test_combat_actions(self, runner):
        """Test combat action generation."""
        # Navigate to a monster room
        actions = runner.get_available_action_dicts()
        path_actions = [a for a in actions if a["type"] == "path_choice"]
        assert len(path_actions) > 0

        # Find a monster room
        for action in path_actions:
            runner.take_action_dict(action)
            if runner.phase == GamePhase.COMBAT:
                break

        if runner.phase != GamePhase.COMBAT:
            pytest.skip("No monster room found in first floor choices")

        actions = runner.get_available_action_dicts()

        assert len(actions) > 0, "Should have combat actions"

        action_types = {a["type"] for a in actions}
        assert "end_turn" in action_types, "End turn should always be available"

        # Check play_card actions have proper structure
        card_actions = [a for a in actions if a["type"] == "play_card"]
        for action in card_actions:
            assert "card_index" in action["params"]
            assert action["phase"] == "combat"

    def test_reward_actions(self, runner):
        """Test reward action generation after combat."""
        # Navigate to monster and win combat
        _navigate_to_combat_and_win(runner)

        if runner.phase != GamePhase.COMBAT_REWARDS:
            pytest.skip("Did not reach rewards phase")

        actions = runner.get_available_action_dicts()

        assert len(actions) > 0, "Should have reward actions"

        action_types = {a["type"] for a in actions}

        # Should have proceed or card pick options
        assert "proceed_from_rewards" in action_types or "pick_card" in action_types or "skip_card" in action_types

    def test_shop_actions(self, runner):
        """Test shop action generation."""
        # Navigate to a shop
        _navigate_to_room_type(runner, "SHOP")

        if runner.phase != GamePhase.SHOP:
            # Deterministic fallback: force-enter shop to validate action surface.
            runner = GameRunner(seed="AGENTTEST_SHOP", ascension=20, verbose=False)
            runner._enter_shop()

        actions = runner.get_available_action_dicts()

        assert len(actions) > 0
        action_types = {a["type"] for a in actions}
        assert "leave_shop" in action_types

    def test_rest_actions(self, runner):
        """Test rest site action generation."""
        # Navigate to a rest site
        _navigate_to_room_type(runner, "REST")

        if runner.phase != GamePhase.REST:
            # Deterministic fallback: force-enter rest to validate action surface.
            runner = GameRunner(seed="AGENTTEST_REST", ascension=20, verbose=False)
            runner._enter_rest()

        actions = runner.get_available_action_dicts()

        assert len(actions) > 0
        action_types = {a["type"] for a in actions}

        # Should have rest or smith
        assert "rest" in action_types or "smith" in action_types

    def test_event_actions(self, runner):
        """Test event action generation."""
        # Navigate to an event
        _navigate_to_room_type(runner, "EVENT")

        if runner.phase != GamePhase.EVENT:
            pytest.skip("Could not reach event")

        actions = runner.get_available_action_dicts()

        assert len(actions) > 0
        for action in actions:
            assert action["type"] == "event_choice"
            assert "choice_index" in action["params"]

    def test_action_ids_are_deterministic(self, runner):
        """Test that action IDs are stable for identical state."""
        actions1 = runner.get_available_action_dicts()

        # Create identical runner
        runner2 = GameRunner(seed="AGENTTEST", ascension=20, verbose=False)
        actions2 = runner2.get_available_action_dicts()

        assert len(actions1) == len(actions2)

        for a1, a2 in zip(actions1, actions2):
            assert a1["id"] == a2["id"], "Action IDs should be deterministic"
            assert a1["type"] == a2["type"]
            assert a1["params"] == a2["params"]

    def test_action_lists_non_empty(self, runner):
        """Test that action lists are non-empty in all phases."""
        # Run through multiple phases
        for _ in range(20):
            if runner.game_over:
                break

            actions = runner.get_available_action_dicts()
            assert len(actions) > 0, f"Actions should be non-empty in phase {runner.phase}"

            # Take first action
            runner.take_action_dict(actions[0])


# =============================================================================
# Action Execution Tests
# =============================================================================

class TestActionExecution:
    """Test take_action_dict() execution."""

    def test_valid_path_choice(self, runner):
        """Test executing a valid path choice."""
        actions = runner.get_available_action_dicts()
        path_action = actions[0]

        result = runner.take_action_dict(path_action)

        assert result.get("success", False), f"Path action should succeed: {result}"

    def test_valid_neow_choice(self, runner_neow):
        """Test executing a valid Neow choice."""
        actions = runner_neow.get_available_action_dicts()
        neow_action = actions[0]

        result = runner_neow.take_action_dict(neow_action)

        assert result.get("success", False), f"Neow action should succeed: {result}"
        assert runner_neow.phase == GamePhase.MAP_NAVIGATION

    def test_invalid_action_type(self, runner):
        """Test that invalid action types return error without state mutation."""
        initial_floor = runner.run_state.floor
        initial_gold = runner.run_state.gold

        result = runner.take_action_dict({
            "type": "invalid_action_type",
            "params": {},
        })

        assert not result.get("success", True), "Invalid action should fail"
        assert "error" in result

        # State should not be mutated
        assert runner.run_state.floor == initial_floor
        assert runner.run_state.gold == initial_gold

    def test_combat_play_card(self, runner):
        """Test playing a card in combat."""
        # Navigate to combat
        _navigate_to_combat(runner)

        if runner.phase != GamePhase.COMBAT:
            pytest.skip("Could not reach combat")

        actions = runner.get_available_action_dicts()
        card_actions = [a for a in actions if a["type"] == "play_card"]

        if card_actions:
            result = runner.take_action_dict(card_actions[0])
            assert result.get("success", False), f"Play card should succeed: {result}"

    def test_combat_end_turn(self, runner):
        """Test ending turn in combat."""
        _navigate_to_combat(runner)

        if runner.phase != GamePhase.COMBAT:
            pytest.skip("Could not reach combat")

        result = runner.take_action_dict({
            "type": "end_turn",
            "params": {},
        })

        assert result.get("success", False), f"End turn should succeed: {result}"

    def test_selection_potion_returns_candidate_actions_when_params_missing(self, runner):
        """Selection-required potions should return explicit candidate actions."""
        state = create_combat(
            player_hp=60,
            player_max_hp=80,
            enemies=[create_enemy("TestEnemy", hp=50, max_hp=50)],
            deck=["Strike_P", "Defend_P", "Vigilance", "Eruption"],
            relics=[],
            potions=["LiquidMemories", "", ""],
        )
        state.discard_pile = ["Strike_P", "Defend_P"]
        runner.current_combat = CombatEngine(state)
        runner.phase = GamePhase.COMBAT

        result = runner.take_action_dict({
            "type": "use_potion",
            "params": {"potion_slot": 0},
        })

        assert result.get("success") is False
        assert result.get("requires_selection") is True
        candidates = result.get("candidate_actions", [])
        assert len(candidates) >= 1
        assert all(a.get("type") == "select_cards" for a in candidates)

    def test_selection_potion_roundtrip_with_select_cards(self, runner):
        """Agent should resolve Liquid Memories via follow-up select_cards action."""
        state = create_combat(
            player_hp=60,
            player_max_hp=80,
            enemies=[create_enemy("TestEnemy", hp=50, max_hp=50)],
            deck=["Strike_P", "Defend_P", "Vigilance", "Eruption"],
            relics=[],
            potions=["LiquidMemories", "", ""],
        )
        state.discard_pile = ["Strike_P", "Defend_P"]
        runner.current_combat = CombatEngine(state)
        runner.phase = GamePhase.COMBAT

        first = runner.take_action_dict({
            "type": "use_potion",
            "params": {"potion_slot": 0},
        })
        candidates = first.get("candidate_actions", [])
        assert candidates, "Expected select_cards candidates"

        second = runner.take_action_dict(candidates[0])
        assert second.get("success") is True
        assert runner.current_combat.state.potions[0] == ""
        assert len(runner.current_combat.state.hand) >= 1

    def test_selection_potion_empty_discard_returns_error(self, runner):
        """Liquid Memories should fail cleanly when discard pile is empty."""
        state = create_combat(
            player_hp=60,
            player_max_hp=80,
            enemies=[create_enemy("TestEnemy", hp=50, max_hp=50)],
            deck=["Strike_P", "Defend_P"],
            relics=[],
            potions=["LiquidMemories", "", ""],
        )
        state.discard_pile = []
        runner.current_combat = CombatEngine(state)
        runner.phase = GamePhase.COMBAT

        result = runner.take_action_dict({
            "type": "use_potion",
            "params": {"potion_slot": 0},
        })

        assert result.get("success") is False
        assert "discard" in result.get("error", "").lower()
        assert runner.current_combat.state.potions[0] == "LiquidMemories"

    def test_selection_potion_triggers_on_use_potion_relics(self, runner):
        """Selection potion flows should trigger onUsePotion relic hooks."""
        state = create_combat(
            player_hp=40,
            player_max_hp=80,
            enemies=[create_enemy("TestEnemy", hp=50, max_hp=50)],
            deck=["Strike_P", "Defend_P"],
            relics=["Toy Ornithopter"],
            potions=["LiquidMemories", "", ""],
        )
        state.discard_pile = ["Strike_P"]
        runner.current_combat = CombatEngine(state)
        runner.phase = GamePhase.COMBAT

        result = runner.take_action_dict({
            "type": "use_potion",
            "params": {"potion_slot": 0, "card_indices": [0]},
        })

        assert result.get("success") is True
        assert runner.current_combat.state.player.hp == 45
        assert runner.current_combat.state.potions[0] == ""

    def test_stance_potion_roundtrip_with_select_stance(self, runner):
        """Stance Potion should emit select_stance actions and apply chosen stance."""
        state = create_combat(
            player_hp=60,
            player_max_hp=80,
            enemies=[create_enemy("TestEnemy", hp=50, max_hp=50)],
            deck=["Strike_P", "Defend_P", "Vigilance", "Eruption"],
            relics=[],
            potions=["StancePotion", "", ""],
        )
        state.stance = "Neutral"
        runner.current_combat = CombatEngine(state)
        runner.phase = GamePhase.COMBAT

        first = runner.take_action_dict({
            "type": "use_potion",
            "params": {"potion_slot": 0},
        })
        candidates = first.get("candidate_actions", [])
        assert candidates, "Expected select_stance candidates"
        assert all(a.get("type") == "select_stance" for a in candidates)

        second = runner.take_action_dict(candidates[0])
        assert second.get("success") is True
        assert runner.current_combat.state.stance in ("Calm", "Wrath")

    def test_gamblers_brew_emits_all_selection_subsets(self, runner):
        """Gamblers Brew should emit every legal hand-subset selection action."""
        state = create_combat(
            player_hp=60,
            player_max_hp=80,
            enemies=[create_enemy("TestEnemy", hp=50, max_hp=50)],
            deck=["Strike_P", "Defend_P", "Vigilance", "Eruption"],
            relics=[],
            potions=["GamblersBrew", "", ""],
        )
        state.hand = ["Strike_P", "Defend_P", "Vigilance"]
        state.draw_pile = ["Eruption", "EmptyBody", "Defend_P", "Strike_P"]
        runner.current_combat = CombatEngine(state)
        runner.phase = GamePhase.COMBAT

        first = runner.take_action_dict({
            "type": "use_potion",
            "params": {"potion_slot": 0},
        })
        assert first.get("requires_selection") is True

        candidates = first.get("candidate_actions", [])
        subsets = {
            tuple(action.get("params", {}).get("card_indices", []))
            for action in candidates
            if action.get("type") == "select_cards"
        }

        # For hand size 3, legal subset count is 2^3 = 8.
        assert len(subsets) == 8
        assert () in subsets
        assert (0, 1, 2) in subsets

    def test_elixir_emits_all_selection_subsets(self, runner):
        """Elixir should emit every legal hand-subset selection action."""
        state = create_combat(
            player_hp=60,
            player_max_hp=80,
            enemies=[create_enemy("TestEnemy", hp=50, max_hp=50)],
            deck=["Strike_P", "Defend_P", "Vigilance", "Eruption"],
            relics=[],
            potions=["ElixirPotion", "", ""],
        )
        state.hand = ["Strike_P", "Defend_P", "Vigilance"]
        runner.current_combat = CombatEngine(state)
        runner.phase = GamePhase.COMBAT

        first = runner.take_action_dict({
            "type": "use_potion",
            "params": {"potion_slot": 0},
        })
        assert first.get("requires_selection") is True

        candidates = first.get("candidate_actions", [])
        subsets = {
            tuple(action.get("params", {}).get("card_indices", []))
            for action in candidates
            if action.get("type") == "select_cards"
        }

        assert len(subsets) == 8
        assert () in subsets
        assert (0, 1, 2) in subsets

    def test_boss_astrolabe_requires_card_selection(self, runner):
        """Picking Astrolabe should require explicit select_cards action."""
        runner.phase = GamePhase.BOSS_REWARDS
        runner.current_rewards = CombatRewards(
            room_type="boss",
            boss_relics=BossRelicChoices(
                relics=[
                    get_relic("Astrolabe"),
                    get_relic("Tiny House"),
                    get_relic("Sozu"),
                ]
            ),
        )

        result = runner.take_action_dict({
            "type": "pick_boss_relic",
            "params": {"relic_index": 0},
        })

        assert result.get("success") is False
        assert result.get("requires_selection") is True
        candidates = result.get("candidate_actions", [])
        assert candidates
        assert all(a.get("type") == "select_cards" for a in candidates)
        assert all(len(a.get("params", {}).get("card_indices", [])) == 3 for a in candidates)

    def test_boss_astrolabe_selection_roundtrip(self, runner):
        """Astrolabe pick should complete after select_cards and apply relic effect."""
        runner.phase = GamePhase.BOSS_REWARDS
        runner.current_rewards = CombatRewards(
            room_type="boss",
            boss_relics=BossRelicChoices(
                relics=[
                    get_relic("Astrolabe"),
                    get_relic("Tiny House"),
                    get_relic("Sozu"),
                ]
            ),
        )
        before_deck_size = len(runner.run_state.deck)

        first = runner.take_action_dict({
            "type": "pick_boss_relic",
            "params": {"relic_index": 0},
        })
        candidates = first.get("candidate_actions", [])
        assert candidates

        second = runner.take_action_dict(candidates[0])
        assert second.get("success") is True
        assert runner.run_state.has_relic("Astrolabe")
        assert len(runner.run_state.deck) == before_deck_size

    def test_boss_empty_cage_requires_card_selection(self, runner):
        """Picking Empty Cage should require explicit select_cards action."""
        runner.phase = GamePhase.BOSS_REWARDS
        runner.current_rewards = CombatRewards(
            room_type="boss",
            boss_relics=BossRelicChoices(
                relics=[
                    get_relic("Empty Cage"),
                    get_relic("Tiny House"),
                    get_relic("Sozu"),
                ]
            ),
        )

        result = runner.take_action_dict({
            "type": "pick_boss_relic",
            "params": {"relic_index": 0},
        })

        assert result.get("success") is False
        assert result.get("requires_selection") is True
        candidates = result.get("candidate_actions", [])
        assert candidates
        assert all(a.get("type") == "select_cards" for a in candidates)
        assert all(len(a.get("params", {}).get("card_indices", [])) == 2 for a in candidates)

    def test_boss_empty_cage_selection_roundtrip(self, runner):
        """Empty Cage pick should remove two selected cards after select_cards."""
        runner.phase = GamePhase.BOSS_REWARDS
        runner.current_rewards = CombatRewards(
            room_type="boss",
            boss_relics=BossRelicChoices(
                relics=[
                    get_relic("Empty Cage"),
                    get_relic("Tiny House"),
                    get_relic("Sozu"),
                ]
            ),
        )
        before_deck_size = len(runner.run_state.deck)

        first = runner.take_action_dict({
            "type": "pick_boss_relic",
            "params": {"relic_index": 0},
        })
        candidates = first.get("candidate_actions", [])
        assert candidates

        second = runner.take_action_dict(candidates[0])
        assert second.get("success") is True
        assert runner.run_state.has_relic("Empty Cage")
        assert len(runner.run_state.deck) == before_deck_size - 2

    def test_shop_orrery_requires_card_selection(self, runner):
        """Buying Orrery should require explicit select_cards follow-up."""
        runner._enter_shop()
        runner.run_state.gold = 999
        assert runner.current_shop is not None
        assert runner.current_shop.relics, "Expected at least one relic slot in shop"

        runner.current_shop.relics[0].relic = get_relic("Orrery")
        runner.current_shop.relics[0].price = 1
        runner.current_shop.relics[0].slot_index = 0
        runner.phase = GamePhase.SHOP

        result = runner.take_action_dict({
            "type": "buy_relic",
            "params": {"item_index": 0},
        })

        assert result.get("success") is False
        assert result.get("requires_selection") is True
        candidates = result.get("candidate_actions", [])
        assert candidates
        assert all(a.get("type") == "select_cards" for a in candidates)
        assert all(len(a.get("params", {}).get("card_indices", [])) == 5 for a in candidates)

    def test_shop_orrery_selection_roundtrip_adds_five_cards(self, runner):
        """Buying Orrery should complete after select_cards and add five cards."""
        runner._enter_shop()
        runner.run_state.gold = 999
        assert runner.current_shop is not None
        assert runner.current_shop.relics, "Expected at least one relic slot in shop"

        runner.current_shop.relics[0].relic = get_relic("Orrery")
        runner.current_shop.relics[0].price = 1
        runner.current_shop.relics[0].slot_index = 0
        runner.phase = GamePhase.SHOP
        before_deck_size = len(runner.run_state.deck)

        first = runner.take_action_dict({
            "type": "buy_relic",
            "params": {"item_index": 0},
        })
        candidates = first.get("candidate_actions", [])
        assert candidates

        second = runner.take_action_dict(candidates[0])
        assert second.get("success") is True
        assert runner.run_state.has_relic("Orrery")
        assert len(runner.run_state.deck) == before_deck_size + 5

    def test_reward_orrery_requires_card_selection(self, runner):
        """Claiming an Orrery relic reward should require explicit selection."""
        runner.phase = GamePhase.COMBAT_REWARDS
        runner.current_rewards = CombatRewards(
            room_type="elite",
            relic=RelicReward(relic=get_relic("Orrery")),
        )

        result = runner.take_action_dict({
            "type": "claim_relic",
            "params": {"relic_reward_index": 0},
        })

        assert result.get("success") is False
        assert result.get("requires_selection") is True
        candidates = result.get("candidate_actions", [])
        assert candidates
        assert all(a.get("type") == "select_cards" for a in candidates)
        assert all(len(a.get("params", {}).get("card_indices", [])) == 5 for a in candidates)

    def test_shop_bottled_flame_requires_card_selection(self, runner):
        """Buying Bottled Flame should require explicit attack-card selection."""
        runner._enter_shop()
        runner.run_state.gold = 999
        assert runner.current_shop is not None
        assert runner.current_shop.relics, "Expected at least one relic slot in shop"

        runner.current_shop.relics[0].relic = get_relic("Bottled Flame")
        runner.current_shop.relics[0].price = 1
        runner.current_shop.relics[0].slot_index = 0
        runner.phase = GamePhase.SHOP

        result = runner.take_action_dict({
            "type": "buy_relic",
            "params": {"item_index": 0},
        })

        assert result.get("success") is False
        assert result.get("requires_selection") is True
        candidates = result.get("candidate_actions", [])
        assert candidates
        assert all(a.get("type") == "select_cards" for a in candidates)
        assert all(len(a.get("params", {}).get("card_indices", [])) == 1 for a in candidates)

    def test_shop_bottled_flame_selection_roundtrip_sets_card(self, runner):
        """Bottled Flame buy should complete after select_cards and set bottled card ID."""
        runner._enter_shop()
        runner.run_state.gold = 999
        assert runner.current_shop is not None
        assert runner.current_shop.relics, "Expected at least one relic slot in shop"

        runner.current_shop.relics[0].relic = get_relic("Bottled Flame")
        runner.current_shop.relics[0].price = 1
        runner.current_shop.relics[0].slot_index = 0
        runner.phase = GamePhase.SHOP

        first = runner.take_action_dict({
            "type": "buy_relic",
            "params": {"item_index": 0},
        })
        candidates = first.get("candidate_actions", [])
        assert candidates

        selected = candidates[0]
        selected_idx = selected.get("params", {}).get("card_indices", [None])[0]
        assert selected_idx is not None
        expected_card = runner.run_state.deck[selected_idx]
        expected_card_id = f"{expected_card.id}+" if expected_card.upgraded else expected_card.id

        second = runner.take_action_dict(selected)
        assert second.get("success") is True
        relic = runner.run_state.get_relic("Bottled Flame")
        assert relic is not None
        assert relic.card_id == expected_card_id

    def test_reward_bottled_lightning_requires_card_selection(self, runner):
        """Claiming Bottled Lightning reward should require explicit skill-card selection."""
        runner.phase = GamePhase.COMBAT_REWARDS
        runner.current_rewards = CombatRewards(
            room_type="elite",
            relic=RelicReward(relic=get_relic("Bottled Lightning")),
        )

        result = runner.take_action_dict({
            "type": "claim_relic",
            "params": {"relic_reward_index": 0},
        })

        assert result.get("success") is False
        assert result.get("requires_selection") is True
        candidates = result.get("candidate_actions", [])
        assert candidates
        assert all(a.get("type") == "select_cards" for a in candidates)
        assert all(len(a.get("params", {}).get("card_indices", [])) == 1 for a in candidates)

    def test_shop_dollys_mirror_requires_card_selection(self, runner):
        """Buying DollysMirror should require explicit deck-card selection."""
        runner._enter_shop()
        runner.run_state.gold = 999
        assert runner.current_shop is not None
        assert runner.current_shop.relics, "Expected at least one relic slot in shop"

        runner.current_shop.relics[0].relic = get_relic("DollysMirror")
        runner.current_shop.relics[0].price = 1
        runner.current_shop.relics[0].slot_index = 0
        runner.phase = GamePhase.SHOP

        result = runner.take_action_dict({
            "type": "buy_relic",
            "params": {"item_index": 0},
        })

        assert result.get("success") is False
        assert result.get("requires_selection") is True
        candidates = result.get("candidate_actions", [])
        assert candidates
        assert all(a.get("type") == "select_cards" for a in candidates)
        assert all(len(a.get("params", {}).get("card_indices", [])) == 1 for a in candidates)

    def test_shop_dollys_mirror_roundtrip_duplicates_selected_card(self, runner):
        """DollysMirror buy should duplicate the selected card after select_cards."""
        runner.run_state.add_card("Nirvana", upgraded=True)
        target_index = len(runner.run_state.deck) - 1
        initial_copies = runner.run_state.count_card("Nirvana", upgraded_only=True)
        initial_deck_size = len(runner.run_state.deck)

        runner._enter_shop()
        runner.run_state.gold = 999
        assert runner.current_shop is not None
        assert runner.current_shop.relics, "Expected at least one relic slot in shop"

        runner.current_shop.relics[0].relic = get_relic("DollysMirror")
        runner.current_shop.relics[0].price = 1
        runner.current_shop.relics[0].slot_index = 0
        runner.phase = GamePhase.SHOP

        first = runner.take_action_dict({
            "type": "buy_relic",
            "params": {"item_index": 0},
        })
        candidates = first.get("candidate_actions", [])
        assert candidates

        selected = next(
            (
                action for action in candidates
                if action.get("params", {}).get("card_indices") == [target_index]
            ),
            candidates[0],
        )
        second = runner.take_action_dict(selected)
        assert second.get("success") is True
        assert runner.run_state.has_relic("DollysMirror")
        assert len(runner.run_state.deck) == initial_deck_size + 1
        assert runner.run_state.count_card("Nirvana", upgraded_only=True) == initial_copies + 1

    def test_reward_dollys_mirror_requires_card_selection(self, runner):
        """Claiming DollysMirror reward should require explicit selection."""
        runner.phase = GamePhase.COMBAT_REWARDS
        runner.current_rewards = CombatRewards(
            room_type="elite",
            relic=RelicReward(relic=get_relic("DollysMirror")),
        )

        result = runner.take_action_dict({
            "type": "claim_relic",
            "params": {"relic_reward_index": 0},
        })

        assert result.get("success") is False
        assert result.get("requires_selection") is True
        candidates = result.get("candidate_actions", [])
        assert candidates
        assert all(a.get("type") == "select_cards" for a in candidates)
        assert all(len(a.get("params", {}).get("card_indices", [])) == 1 for a in candidates)

    def test_orrery_selection_candidates_have_deterministic_ids(self, runner):
        """Equivalent state snapshots should emit identical Orrery selection IDs."""
        runner2 = GameRunner(seed="AGENTTEST", ascension=20, verbose=False)

        for r in (runner, runner2):
            r._enter_shop()
            r.run_state.gold = 999
            assert r.current_shop is not None
            assert r.current_shop.relics
            r.current_shop.relics[0].relic = get_relic("Orrery")
            r.current_shop.relics[0].price = 1
            r.current_shop.relics[0].slot_index = 0
            r.phase = GamePhase.SHOP

        first = runner.take_action_dict({
            "type": "buy_relic",
            "params": {"item_index": 0},
        })
        second = runner2.take_action_dict({
            "type": "buy_relic",
            "params": {"item_index": 0},
        })

        first_ids = [a["id"] for a in first.get("candidate_actions", [])]
        second_ids = [a["id"] for a in second.get("candidate_actions", [])]
        assert first_ids == second_ids

    def test_orrery_rejects_invalid_selection_combination(self, runner):
        """Orrery should reject selections that don't pick one card per offer set."""
        runner._enter_shop()
        runner.run_state.gold = 999
        assert runner.current_shop is not None
        assert runner.current_shop.relics
        runner.current_shop.relics[0].relic = get_relic("Orrery")
        runner.current_shop.relics[0].price = 1
        runner.current_shop.relics[0].slot_index = 0
        runner.phase = GamePhase.SHOP

        first = runner.take_action_dict({
            "type": "buy_relic",
            "params": {"item_index": 0},
        })
        assert first.get("requires_selection") is True

        invalid = runner.take_action_dict({
            "type": "select_cards",
            "params": {
                "pile": "offer",
                "card_indices": [0, 1, 3, 6, 9],  # two picks from first offer bucket
                "min_cards": 5,
                "max_cards": 5,
                "parent_action_id": "",
            },
        })
        assert invalid.get("success") is False
        assert "Invalid selection combination" in invalid.get("error", "")

    def test_bottled_selection_rejects_invalid_card_index(self, runner):
        """Bottled relic selection should reject non-eligible card indices."""
        runner._enter_shop()
        runner.run_state.gold = 999
        assert runner.current_shop is not None
        assert runner.current_shop.relics
        runner.current_shop.relics[0].relic = get_relic("Bottled Flame")
        runner.current_shop.relics[0].price = 1
        runner.current_shop.relics[0].slot_index = 0
        runner.phase = GamePhase.SHOP

        first = runner.take_action_dict({
            "type": "buy_relic",
            "params": {"item_index": 0},
        })
        assert first.get("requires_selection") is True

        invalid = runner.take_action_dict({
            "type": "select_cards",
            "params": {
                "pile": "deck",
                "card_indices": [999],
                "min_cards": 1,
                "max_cards": 1,
                "parent_action_id": "",
            },
        })
        assert invalid.get("success") is False
        assert "Invalid selected card index" in invalid.get("error", "")

    def test_event_choice_requires_selection_without_state_mutation(self, runner):
        """Card-select event choices should return candidate actions without mutating live state."""
        runner.phase = GamePhase.EVENT
        runner.current_event_state = EventState(event_id="LivingWall", phase=EventPhase.INITIAL)

        before_deck = [(card.id, card.upgraded, card.misc_value) for card in runner.run_state.deck]
        before_phase = runner.phase

        first = runner.take_action_dict({
            "type": "event_choice",
            "params": {"choice_index": 0},  # LivingWall forget -> remove
        })

        assert first.get("success") is False
        assert first.get("requires_selection") is True
        candidates = first.get("candidate_actions", [])
        assert candidates
        assert all(a.get("type") == "select_cards" for a in candidates)
        assert all(len(a.get("params", {}).get("card_indices", [])) == 1 for a in candidates)

        removable = {idx for idx, _ in runner.run_state.get_removable_cards()}
        candidate_indices = {
            a.get("params", {}).get("card_indices", [None])[0]
            for a in candidates
        }
        assert candidate_indices == removable

        after_deck = [(card.id, card.upgraded, card.misc_value) for card in runner.run_state.deck]
        assert before_deck == after_deck
        assert runner.phase == before_phase == GamePhase.EVENT
        assert runner.current_event_state is not None

    def test_event_choice_selection_roundtrip_uses_selected_card_index(self, runner):
        """select_cards follow-up should execute event choice with the selected deck index."""
        runner.phase = GamePhase.EVENT
        runner.current_event_state = EventState(event_id="LivingWall", phase=EventPhase.INITIAL)

        removable = [idx for idx, _ in runner.run_state.get_removable_cards()]
        assert removable
        target_index = removable[-1]
        target_card = runner.run_state.deck[target_index]
        target_before_count = sum(1 for card in runner.run_state.deck if card.id == target_card.id)

        first = runner.take_action_dict({
            "type": "event_choice",
            "params": {"choice_index": 0},  # LivingWall forget -> remove
        })
        assert first.get("requires_selection") is True
        candidates = first.get("candidate_actions", [])
        assert candidates

        selected = next(
            action
            for action in candidates
            if action.get("params", {}).get("card_indices") == [target_index]
        )
        second = runner.take_action_dict(selected)

        assert second.get("success") is True
        assert runner.current_event_state is None
        assert runner.phase == GamePhase.MAP_NAVIGATION

        target_after_count = sum(1 for card in runner.run_state.deck if card.id == target_card.id)
        assert target_after_count == target_before_count - 1

    def test_event_multiphase_golden_idol_keeps_event_phase_and_updates_choices(self, runner):
        """Golden Idol should remain in event phase after take and expose secondary choices."""
        runner.phase = GamePhase.EVENT
        runner.current_event_state = EventState(event_id="GoldenIdol", phase=EventPhase.INITIAL)

        initial_actions = runner.get_available_action_dicts()
        assert {a.get("params", {}).get("choice_index") for a in initial_actions} == {0, 1}

        first = runner.take_action_dict({
            "type": "event_choice",
            "params": {"choice_index": 0},  # take
        })
        assert first.get("success") is True
        assert runner.run_state.has_relic("GoldenIdol")
        assert runner.phase == GamePhase.EVENT
        assert runner.current_event_state is not None
        assert runner.current_event_state.phase == EventPhase.SECONDARY

        secondary_actions = runner.get_available_action_dicts()
        assert secondary_actions
        assert all(action.get("type") == "event_choice" for action in secondary_actions)
        assert {a.get("params", {}).get("choice_index") for a in secondary_actions} == {0, 1, 2}

    def test_event_multiphase_golden_idol_followup_action_ids_are_deterministic(self, runner):
        """Equivalent multi-phase event states should emit identical follow-up action IDs."""
        runner2 = GameRunner(seed="AGENTTEST", ascension=20, verbose=False)

        for current in (runner, runner2):
            current.phase = GamePhase.EVENT
            current.current_event_state = EventState(event_id="GoldenIdol", phase=EventPhase.INITIAL)

        first_1 = runner.take_action_dict({
            "type": "event_choice",
            "params": {"choice_index": 0},
        })
        first_2 = runner2.take_action_dict({
            "type": "event_choice",
            "params": {"choice_index": 0},
        })
        assert first_1.get("success") is True
        assert first_2.get("success") is True
        assert runner.current_event_state is not None
        assert runner2.current_event_state is not None
        assert runner.current_event_state.phase == EventPhase.SECONDARY
        assert runner2.current_event_state.phase == EventPhase.SECONDARY

        followup_ids_1 = [action["id"] for action in runner.get_available_action_dicts()]
        followup_ids_2 = [action["id"] for action in runner2.get_available_action_dicts()]
        assert followup_ids_1 == followup_ids_2


# =============================================================================
# Observation Schema Tests
# =============================================================================

class TestObservationSchema:
    """Test get_observation() returns complete, valid data."""

    def test_observation_is_json_serializable(self, runner):
        """Test that observation can be serialized to JSON."""
        obs = runner.get_observation()

        # Should not raise
        json_str = json.dumps(obs)
        assert len(json_str) > 0

    def test_observation_has_required_fields(self, runner):
        """Test observation contains all required top-level fields."""
        obs = runner.get_observation()

        assert "phase" in obs
        assert "run" in obs
        assert "map" in obs

    def test_run_section_completeness(self, runner):
        """Test run section contains all required fields."""
        obs = runner.get_observation()
        run = obs["run"]

        required_fields = [
            "seed", "ascension", "act", "floor",
            "gold", "current_hp", "max_hp",
            "deck", "relics", "potions", "keys", "map_position",
        ]

        for field in required_fields:
            assert field in run, f"Run section missing {field}"

    def test_deck_observation_format(self, runner):
        """Test deck cards have proper format."""
        obs = runner.get_observation()
        deck = obs["run"]["deck"]

        assert len(deck) > 0, "Deck should not be empty"

        for card in deck:
            assert "id" in card
            assert "upgraded" in card
            assert "misc_value" in card

    def test_relics_observation_format(self, runner):
        """Test relics have proper format."""
        obs = runner.get_observation()
        relics = obs["run"]["relics"]

        assert len(relics) > 0, "Should have starting relic"

        for relic in relics:
            assert "id" in relic
            assert "counter" in relic

    def test_map_observation_completeness(self, runner):
        """Test map section contains all required fields."""
        obs = runner.get_observation()
        map_data = obs["map"]

        required_fields = ["act", "nodes", "edges", "available_paths", "visited_nodes"]

        for field in required_fields:
            assert field in map_data, f"Map section missing {field}"

    def test_available_paths_matches_actions(self, runner):
        """Test available_paths count matches path_choice action count."""
        obs = runner.get_observation()
        actions = runner.get_available_action_dicts()

        path_actions = [a for a in actions if a["type"] == "path_choice"]
        available_paths = obs["map"]["available_paths"]

        assert len(path_actions) == len(available_paths)

    def test_combat_observation_when_in_combat(self, runner):
        """Test combat section is populated during combat."""
        _navigate_to_combat(runner)

        if runner.phase != GamePhase.COMBAT:
            pytest.skip("Could not reach combat")

        obs = runner.get_observation()

        assert obs["combat"] is not None
        combat = obs["combat"]

        required_fields = [
            "player", "energy", "max_energy", "stance",
            "hand", "draw_pile", "discard_pile", "exhaust_pile",
            "enemies", "turn",
        ]

        for field in required_fields:
            assert field in combat, f"Combat section missing {field}"

    def test_enemy_observation_format(self, runner):
        """Test enemy data format in combat."""
        _navigate_to_combat(runner)

        if runner.phase != GamePhase.COMBAT:
            pytest.skip("Could not reach combat")

        obs = runner.get_observation()
        enemies = obs["combat"]["enemies"]

        assert len(enemies) > 0

        for enemy in enemies:
            assert "id" in enemy
            assert "hp" in enemy
            assert "max_hp" in enemy
            assert "move_damage" in enemy
            assert "move_hits" in enemy

    def test_observation_determinism(self, runner):
        """Test observation is deterministic for identical state."""
        obs1 = runner.get_observation()

        runner2 = GameRunner(seed="AGENTTEST", ascension=20, verbose=False)
        obs2 = runner2.get_observation()

        # Compare JSON strings for full equality
        json1 = json.dumps(obs1, sort_keys=True)
        json2 = json.dumps(obs2, sort_keys=True)

        assert json1 == json2, "Observations should be identical for same seed"


# =============================================================================
# Phase Transition Tests
# =============================================================================

class TestPhaseTransitions:
    """Test valid phase transitions."""

    def test_neow_to_map(self, runner_neow):
        """Test NEOW -> MAP_NAVIGATION transition."""
        assert runner_neow.phase == GamePhase.NEOW

        actions = runner_neow.get_available_action_dicts()
        runner_neow.take_action_dict(actions[0])

        assert runner_neow.phase == GamePhase.MAP_NAVIGATION

    def test_map_to_combat(self, runner):
        """Test MAP_NAVIGATION -> COMBAT transition."""
        _navigate_to_combat(runner)

        # Should be in combat or some other valid phase
        assert runner.phase in [
            GamePhase.COMBAT, GamePhase.EVENT, GamePhase.SHOP,
            GamePhase.REST, GamePhase.TREASURE,
        ]

    def test_combat_to_rewards(self, runner):
        """Test COMBAT -> COMBAT_REWARDS transition."""
        _navigate_to_combat_and_win(runner)

        # After winning combat, should be in rewards or map
        assert runner.phase in [GamePhase.COMBAT_REWARDS, GamePhase.MAP_NAVIGATION, GamePhase.RUN_COMPLETE]

    def test_rewards_to_map(self, runner):
        """Test COMBAT_REWARDS -> MAP_NAVIGATION transition."""
        _navigate_to_combat_and_win(runner)

        if runner.phase != GamePhase.COMBAT_REWARDS:
            pytest.skip("Did not reach rewards")

        # Proceed through rewards
        max_iterations = 20
        for _ in range(max_iterations):
            if runner.phase != GamePhase.COMBAT_REWARDS:
                break
            actions = runner.get_available_action_dicts()
            runner.take_action_dict(actions[0])

        assert runner.phase in [GamePhase.MAP_NAVIGATION, GamePhase.RUN_COMPLETE, GamePhase.BOSS_REWARDS]


# =============================================================================
# Determinism Tests
# =============================================================================

class TestDeterminism:
    """Test that same seed + actions = same results."""

    def test_full_run_determinism(self):
        """Test that replaying the same actions produces identical results."""
        # First run - collect action sequence
        runner1 = GameRunner(seed="DETERMINISM", ascension=20, verbose=False)
        action_sequence = []

        for _ in range(50):  # Run 50 steps
            if runner1.game_over:
                break
            actions = runner1.get_available_action_dicts()
            action = actions[0]  # Always take first action
            action_sequence.append(action)
            runner1.take_action_dict(action)

        final_obs1 = runner1.get_observation()

        # Second run - replay same actions
        runner2 = GameRunner(seed="DETERMINISM", ascension=20, verbose=False)

        for action in action_sequence:
            if runner2.game_over:
                break
            runner2.take_action_dict(action)

        final_obs2 = runner2.get_observation()

        # Should be identical
        assert final_obs1["run"]["floor"] == final_obs2["run"]["floor"]
        assert final_obs1["run"]["current_hp"] == final_obs2["run"]["current_hp"]
        assert final_obs1["run"]["gold"] == final_obs2["run"]["gold"]
        assert len(final_obs1["run"]["deck"]) == len(final_obs2["run"]["deck"])

    def test_action_id_stability(self):
        """Test that action IDs are stable across runs."""
        runner1 = GameRunner(seed="STABILITY", ascension=20, verbose=False)
        runner2 = GameRunner(seed="STABILITY", ascension=20, verbose=False)

        for _ in range(10):
            if runner1.game_over or runner2.game_over:
                break

            actions1 = runner1.get_available_action_dicts()
            actions2 = runner2.get_available_action_dicts()

            # Action IDs should match
            ids1 = [a["id"] for a in actions1]
            ids2 = [a["id"] for a in actions2]

            assert ids1 == ids2, "Action IDs should be identical"

            # Take same action in both
            runner1.take_action_dict(actions1[0])
            runner2.take_action_dict(actions2[0])


# =============================================================================
# Integration Tests
# =============================================================================

class TestIntegration:
    """End-to-end integration tests."""

    def test_full_floor_cycle(self, runner):
        """Test completing a full floor cycle (map -> room -> rewards -> map)."""
        initial_floor = runner.run_state.floor

        # Navigate to room
        actions = runner.get_available_action_dicts()
        runner.take_action_dict(actions[0])

        # Handle whatever room type
        max_iterations = 100
        for _ in range(max_iterations):
            if runner.game_over or runner.phase == GamePhase.MAP_NAVIGATION:
                break
            actions = runner.get_available_action_dicts()
            if not actions:
                break
            runner.take_action_dict(actions[0])

        # Should have advanced floor and returned to map
        if not runner.game_over:
            assert runner.run_state.floor == initial_floor + 1

    def test_multiple_floors(self, runner):
        """Test completing multiple floors."""
        floors_completed = 0
        max_iterations = 500

        for _ in range(max_iterations):
            if runner.game_over:
                break

            actions = runner.get_available_action_dicts()
            if not actions:
                break

            if runner.phase == GamePhase.MAP_NAVIGATION:
                floors_completed = runner.run_state.floor

            runner.take_action_dict(actions[0])

        # Should have completed at least a few floors
        assert floors_completed >= 1, "Should complete at least 1 floor"

    def test_observation_action_consistency(self, runner):
        """Test that observations contain info needed to select actions."""
        for _ in range(30):
            if runner.game_over:
                break

            obs = runner.get_observation()
            actions = runner.get_available_action_dicts()

            # Check phase consistency
            phase_name = obs["phase"]
            for action in actions:
                # Action phase should correspond to observation phase
                assert action["phase"] in [phase_name, "combat", "reward", "boss_reward", "map", "event", "shop", "rest", "treasure", "neow"]

            runner.take_action_dict(actions[0])


# =============================================================================
# Helper Functions
# =============================================================================

def _navigate_to_combat(runner: GameRunner, max_steps: int = 50):
    """Navigate to a combat room."""
    for _ in range(max_steps):
        if runner.game_over or runner.phase == GamePhase.COMBAT:
            break

        actions = runner.get_available_action_dicts()
        if not actions:
            break

        # If on map, try to find a monster room
        if runner.phase == GamePhase.MAP_NAVIGATION:
            for action in actions:
                if action["type"] == "path_choice":
                    runner.take_action_dict(action)
                    break
        else:
            runner.take_action_dict(actions[0])


def _navigate_to_combat_and_win(runner: GameRunner, max_steps: int = 200):
    """Navigate to combat and win it."""
    _navigate_to_combat(runner)

    if runner.phase != GamePhase.COMBAT:
        return

    for _ in range(max_steps):
        if runner.game_over or runner.phase != GamePhase.COMBAT:
            break

        actions = runner.get_available_action_dicts()
        if not actions:
            break

        runner.take_action_dict(actions[0])


def _navigate_to_room_type(runner: GameRunner, room_type: str, max_floors: int = 10):
    """Try to navigate to a specific room type."""
    for _ in range(max_floors):
        if runner.game_over:
            break

        # Handle current phase
        if runner.phase == GamePhase.MAP_NAVIGATION:
            obs = runner.get_observation()
            available_paths = obs["map"]["available_paths"]

            # Look for desired room type
            target_idx = None
            for i, path in enumerate(available_paths):
                if path["room_type"] == room_type:
                    target_idx = i
                    break

            if target_idx is not None:
                actions = runner.get_available_action_dicts()
                for action in actions:
                    if action["params"].get("node_index") == target_idx:
                        runner.take_action_dict(action)
                        return

            # If not found, take first path
            actions = runner.get_available_action_dicts()
            if actions:
                runner.take_action_dict(actions[0])

        else:
            # Handle other phases (combat, events, etc.)
            max_iterations = 100
            for _ in range(max_iterations):
                if runner.game_over or runner.phase == GamePhase.MAP_NAVIGATION:
                    break
                actions = runner.get_available_action_dicts()
                if not actions:
                    break
                runner.take_action_dict(actions[0])


# =============================================================================
# Run tests
# =============================================================================

if __name__ == "__main__":
    pytest.main([__file__, "-v"])
