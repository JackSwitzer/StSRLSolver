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
import numpy as np
from typing import List, Dict, Any

from packages.engine import (
    GameRunner, GamePhase,
    ActionDict, ActionResult, ObservationDict,
)
from packages.engine.combat_engine import CombatEngine
from packages.engine.content.cards import get_card
from packages.engine.content.relics import get_relic
from packages.engine.handlers.event_handler import EventPhase, EventState
from packages.engine.handlers.reward_handler import (
    RewardHandler,
    CombatRewards,
    BossRelicChoices,
    RelicReward,
    CardReward,
    GoldReward,
    ClaimGoldAction,
    ClaimPotionAction,
    SkipPotionAction,
    PickCardAction,
    SkipCardAction,
    SingingBowlAction,
    ClaimRelicAction,
    ClaimEmeraldKeyAction,
    SkipEmeraldKeyAction,
    ProceedFromRewardsAction,
)
from packages.engine.state.combat import create_combat, create_enemy
from packages.engine.rl_masks import ActionSpace


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

    def test_reward_action_surface_matches_reward_handler(self, runner):
        """Reward action emission should mirror RewardHandler available actions."""
        runner.phase = GamePhase.COMBAT_REWARDS
        runner.current_rewards = CombatRewards(
            room_type="monster",
            gold=GoldReward(amount=25, claimed=False),
        )

        actions = runner.get_available_action_dicts()
        action_types = {action["type"] for action in actions}

        expected_types = set()
        for handler_action in RewardHandler.get_available_actions(runner.run_state, runner.current_rewards):
            if isinstance(handler_action, ClaimGoldAction):
                expected_types.add("claim_gold")
            elif isinstance(handler_action, ClaimPotionAction):
                expected_types.add("claim_potion")
            elif isinstance(handler_action, SkipPotionAction):
                expected_types.add("skip_potion")
            elif isinstance(handler_action, PickCardAction):
                expected_types.add("pick_card")
            elif isinstance(handler_action, SkipCardAction):
                expected_types.add("skip_card")
            elif isinstance(handler_action, SingingBowlAction):
                expected_types.add("singing_bowl")
            elif isinstance(handler_action, ClaimRelicAction):
                expected_types.add("claim_relic")
            elif isinstance(handler_action, ClaimEmeraldKeyAction):
                expected_types.add("claim_emerald_key")
            elif isinstance(handler_action, SkipEmeraldKeyAction):
                expected_types.add("skip_emerald_key")
            elif isinstance(handler_action, ProceedFromRewardsAction):
                expected_types.add("proceed_from_rewards")

        assert action_types == expected_types

    def test_claim_gold_returns_error_when_already_claimed(self, runner):
        """Claiming gold twice should return success=False with an error."""
        runner.phase = GamePhase.COMBAT_REWARDS
        runner.current_rewards = CombatRewards(
            room_type="monster",
            gold=GoldReward(amount=10, claimed=True),
        )

        result = runner.take_action_dict({
            "type": "claim_gold",
            "params": {},
        })

        assert result.get("success") is False
        assert "gold" in result.get("error", "").lower()

    def test_proceed_from_rewards_fails_with_unresolved_card_reward(self, runner):
        """Proceed should fail while mandatory card rewards are unresolved."""
        runner.phase = GamePhase.COMBAT_REWARDS
        runner.current_rewards = CombatRewards(
            room_type="monster",
            gold=GoldReward(amount=12, claimed=False),
            card_rewards=[CardReward(cards=[get_card("Strike_P"), get_card("Defend_P")])],
        )

        result = runner.take_action_dict({
            "type": "proceed_from_rewards",
            "params": {},
        })

        assert result.get("success") is False
        assert "proceed" in result.get("error", "").lower()
        assert runner.phase == GamePhase.COMBAT_REWARDS
        assert runner.current_rewards is not None

    def test_proceed_from_rewards_fails_with_unresolved_relic_reward(self, runner):
        """Proceed should fail while a mandatory elite relic reward is unresolved."""
        runner.phase = GamePhase.COMBAT_REWARDS
        runner.current_rewards = CombatRewards(
            room_type="elite",
            relic=RelicReward(relic=get_relic("Anchor")),
        )

        result = runner.take_action_dict({
            "type": "proceed_from_rewards",
            "params": {},
        })

        assert result.get("success") is False
        assert "proceed" in result.get("error", "").lower()
        assert runner.phase == GamePhase.COMBAT_REWARDS
        assert runner.current_rewards is not None

    def test_reward_actions_include_second_relic_claim_index(self, runner):
        """Black Star-style reward state should expose claim actions for both relic indices."""
        runner.phase = GamePhase.COMBAT_REWARDS
        runner.current_rewards = CombatRewards(
            room_type="elite",
            gold=GoldReward(amount=10, claimed=True),
            relic=RelicReward(relic=get_relic("Anchor")),
            second_relic=RelicReward(relic=get_relic("Bag of Preparation")),
        )

        actions = runner.get_available_action_dicts()
        claim_relic_actions = [a for a in actions if a.get("type") == "claim_relic"]
        indices = {
            int(action.get("params", {}).get("relic_reward_index", -1))
            for action in claim_relic_actions
        }
        assert indices == {0, 1}

    def test_claim_second_relic_by_index(self, runner):
        """claim_relic with relic_reward_index=1 should claim second relic reward."""
        runner.phase = GamePhase.COMBAT_REWARDS
        runner.current_rewards = CombatRewards(
            room_type="elite",
            gold=GoldReward(amount=10, claimed=True),
            relic=RelicReward(relic=get_relic("Anchor")),
            second_relic=RelicReward(relic=get_relic("Bag of Preparation")),
        )

        result = runner.take_action_dict({
            "type": "claim_relic",
            "params": {"relic_reward_index": 1},
        })

        assert result.get("success") is True
        assert runner.current_rewards is not None
        assert runner.current_rewards.second_relic is not None
        assert runner.current_rewards.second_relic.claimed is True
        assert runner.run_state.has_relic("Bag of Preparation")

    def test_proceed_blocked_until_second_relic_claimed(self, runner):
        """Proceed should remain blocked while second mandatory relic reward is unclaimed."""
        runner.phase = GamePhase.COMBAT_REWARDS
        runner.current_rewards = CombatRewards(
            room_type="elite",
            gold=GoldReward(amount=10, claimed=True),
            relic=RelicReward(relic=get_relic("Anchor")),
            second_relic=RelicReward(relic=get_relic("Bag of Preparation")),
        )

        claim_primary = runner.take_action_dict({
            "type": "claim_relic",
            "params": {"relic_reward_index": 0},
        })
        assert claim_primary.get("success") is True

        blocked = runner.take_action_dict({
            "type": "proceed_from_rewards",
            "params": {},
        })
        assert blocked.get("success") is False
        assert "proceed" in blocked.get("error", "").lower()
        assert runner.phase == GamePhase.COMBAT_REWARDS


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

        assert "observation_schema_version" in obs
        assert "action_schema_version" in obs
        assert isinstance(obs["observation_schema_version"], str)
        assert isinstance(obs["action_schema_version"], str)
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
# RL Action Mask Contract Tests (RL-ACT-001)
# =============================================================================

class TestActionMaskContract:
    """Tests for RL action mask contract: determinism, stability, round-trips."""

    def test_action_ordering_deterministic(self, runner):
        """get_available_action_dicts returns same order for identical state."""
        actions1 = runner.get_available_action_dicts()
        actions2 = runner.get_available_action_dicts()

        assert [a["id"] for a in actions1] == [a["id"] for a in actions2]

    def test_action_ordering_deterministic_across_runners(self):
        """Two runners from same seed produce identical action ordering."""
        r1 = GameRunner(seed="MASK_ORDER", ascension=20, verbose=False)
        r2 = GameRunner(seed="MASK_ORDER", ascension=20, verbose=False)

        for _ in range(15):
            if r1.game_over or r2.game_over:
                break
            a1 = r1.get_available_action_dicts()
            a2 = r2.get_available_action_dicts()
            assert [a["id"] for a in a1] == [a["id"] for a in a2], (
                f"Action ID ordering diverged at phase {r1.phase}"
            )
            assert [a["type"] for a in a1] == [a["type"] for a in a2]
            assert [a["params"] for a in a1] == [a["params"] for a in a2]
            r1.take_action_dict(a1[0])
            r2.take_action_dict(a2[0])

    def test_action_ids_stable_across_repeated_calls(self, runner):
        """Action IDs are preserved across multiple get calls on same state."""
        ids_first = [a["id"] for a in runner.get_available_action_dicts()]
        for _ in range(5):
            ids_now = [a["id"] for a in runner.get_available_action_dicts()]
            assert ids_now == ids_first

    def test_action_ids_unique_within_list(self, runner):
        """No duplicate IDs within a single action list."""
        for _ in range(20):
            if runner.game_over:
                break
            actions = runner.get_available_action_dicts()
            ids = [a["id"] for a in actions]
            assert len(ids) == len(set(ids)), f"Duplicate IDs found: {ids}"
            runner.take_action_dict(actions[0])

    def test_invalid_action_type_rejected_with_error(self, runner):
        """Submitting an invalid action type returns error, not silent corruption."""
        result = runner.take_action_dict({
            "type": "totally_invalid",
            "params": {},
        })
        assert not result.get("success", True)
        assert "error" in result
        assert isinstance(result["error"], str)
        assert len(result["error"]) > 0

    def test_invalid_action_params_rejected(self, runner):
        """Submitting wrong params for a valid type returns error."""
        result = runner.take_action_dict({
            "type": "path_choice",
            "params": {"node_index": 9999},
        })
        assert not result.get("success", True)
        assert "error" in result

    def test_missing_required_params_rejected(self, runner):
        """Missing required params causes an error."""
        result = runner.take_action_dict({
            "type": "path_choice",
            "params": {},
        })
        assert not result.get("success", True)

    def test_generate_action_id_deterministic(self):
        """generate_action_id produces consistent IDs for same inputs."""
        from packages.engine.agent_api import generate_action_id
        id1 = generate_action_id("play_card", 2, 0)
        id2 = generate_action_id("play_card", 2, 0)
        assert id1 == id2
        # Different params produce different IDs
        id3 = generate_action_id("play_card", 3, 0)
        assert id1 != id3

    def test_two_step_selection_emits_explicit_actions(self, runner):
        """Two-step selection puts engine in pending state with explicit actions."""
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

        # Trigger the two-step flow
        result = runner.take_action_dict({
            "type": "use_potion",
            "params": {"potion_slot": 0},
        })
        assert result.get("requires_selection") is True

        # Engine should now be in pending_selection state
        assert runner.pending_selection is not None

        # get_available_action_dicts should return selection actions
        selection_actions = runner.get_available_action_dicts()
        assert len(selection_actions) > 0
        assert all(a["type"] == "select_cards" for a in selection_actions)

        # Selection actions should have proper IDs
        for a in selection_actions:
            assert "id" in a
            assert len(a["id"]) > 0

    def test_two_step_selection_mask_round_trip(self, runner):
        """Mask built from selection actions correctly round-trips."""
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

        runner.take_action_dict({
            "type": "use_potion",
            "params": {"potion_slot": 0},
        })

        selection_actions = runner.get_available_action_dicts()
        space = ActionSpace()
        mask = space.actions_to_mask(selection_actions)

        # Round-trip
        filtered = space.mask_to_actions(mask, selection_actions)
        assert len(filtered) == len(selection_actions)
        assert [a["id"] for a in filtered] == [a["id"] for a in selection_actions]


class TestActionSpaceMask:
    """Tests for the ActionSpace / rl_masks module."""

    def test_basic_mask_round_trip(self, runner):
        """actions -> mask -> filtered actions preserves all actions."""
        actions = runner.get_available_action_dicts()
        space = ActionSpace()
        mask = space.actions_to_mask(actions)

        assert mask.dtype == np.bool_
        assert mask.sum() == len(actions)

        filtered = space.mask_to_actions(mask, actions)
        assert len(filtered) == len(actions)
        assert {a["id"] for a in filtered} == {a["id"] for a in actions}

    def test_action_to_index_and_back(self, runner):
        """action_to_index and index_to_action are consistent inverses."""
        actions = runner.get_available_action_dicts()
        space = ActionSpace()
        space.register_actions(actions)

        for action in actions:
            idx = space.action_to_index(action)
            recovered = space.index_to_action(idx, actions)
            assert recovered is not None
            assert recovered["id"] == action["id"]

    def test_mask_grows_with_new_actions(self):
        """Space size grows as new action IDs are registered."""
        space = ActionSpace()
        assert space.size == 0

        actions_a = [{"id": "end_turn", "type": "end_turn", "params": {}, "phase": "combat"}]
        mask_a = space.actions_to_mask(actions_a)
        assert space.size == 1
        assert mask_a.shape == (1,)

        actions_b = [{"id": "rest", "type": "rest", "params": {}, "phase": "rest"}]
        mask_b = space.actions_to_mask(actions_b)
        assert space.size == 2
        assert mask_b.shape == (2,)
        # First action should not be in second mask
        assert mask_b[0] == False
        assert mask_b[1] == True

    def test_mask_is_deterministic_across_runners(self):
        """Same seed produces identical masks."""
        r1 = GameRunner(seed="MASK_DET", ascension=20, verbose=False)
        r2 = GameRunner(seed="MASK_DET", ascension=20, verbose=False)

        space = ActionSpace()
        mask1 = space.actions_to_mask(r1.get_available_action_dicts())
        mask2 = space.actions_to_mask(r2.get_available_action_dicts())

        np.testing.assert_array_equal(mask1, mask2)

    def test_index_to_action_returns_none_for_unknown(self):
        """index_to_action returns None for out-of-range index."""
        space = ActionSpace()
        space.register("end_turn")
        result = space.index_to_action(999, [])
        assert result is None

    def test_action_to_index_raises_without_id(self):
        """action_to_index raises KeyError when action has no id."""
        space = ActionSpace()
        with pytest.raises(KeyError):
            space.action_to_index({"type": "end_turn", "params": {}})

    def test_contains_check(self):
        """__contains__ reflects registration state."""
        space = ActionSpace()
        assert "end_turn" not in space
        space.register("end_turn")
        assert "end_turn" in space

    def test_mask_across_multiple_phases(self):
        """Mask accumulates actions across different game phases."""
        runner = GameRunner(seed="MASK_PHASES", ascension=20, verbose=False)
        space = ActionSpace()

        for _ in range(50):
            if runner.game_over:
                break
            actions = runner.get_available_action_dicts()
            mask = space.actions_to_mask(actions)
            assert mask.sum() == len(actions)
            assert mask.shape[0] == space.size
            runner.take_action_dict(actions[0])

        # Should have accumulated multiple action types
        assert space.size > 3, f"Expected diverse actions, got {space.size}"

    def test_partial_mask_filters_correctly(self):
        """A manually constructed partial mask returns only matching actions."""
        space = ActionSpace()
        actions = [
            {"id": "a1", "type": "t1", "params": {}, "phase": "p"},
            {"id": "a2", "type": "t2", "params": {}, "phase": "p"},
            {"id": "a3", "type": "t3", "params": {}, "phase": "p"},
        ]
        space.register_actions(actions)

        mask = np.zeros(space.size, dtype=np.bool_)
        mask[space.get_index("a2")] = True

        filtered = space.mask_to_actions(mask, actions)
        assert len(filtered) == 1
        assert filtered[0]["id"] == "a2"

    def test_index_to_action_id(self):
        """index_to_action_id returns the right string."""
        space = ActionSpace()
        idx = space.register("end_turn")
        assert space.index_to_action_id(idx) == "end_turn"
        assert space.index_to_action_id(999) is None


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
# RL-OBS-001 Observation Profile Lock Tests
# =============================================================================


class TestObservationProfileLock:
    """Tests for RL-OBS-001: observation profile, schema versioning, field presence."""

    # -- Required run fields ------------------------------------------------

    def test_observation_has_required_run_fields(self, runner):
        """Human profile must include all run-level fields for RL training."""
        obs = runner.get_observation()
        run = obs["run"]

        required = {
            "current_hp", "max_hp", "gold", "floor", "act",
            "deck", "relics", "potions", "ascension",
        }
        for field in required:
            assert field in run, f"Run section missing required field: {field}"

        # Type sanity
        assert isinstance(run["current_hp"], int)
        assert isinstance(run["max_hp"], int)
        assert isinstance(run["gold"], int)
        assert isinstance(run["floor"], int)
        assert isinstance(run["act"], int)
        assert isinstance(run["ascension"], int)
        assert isinstance(run["deck"], list)
        assert isinstance(run["relics"], list)
        assert isinstance(run["potions"], list)

    # -- Required combat fields ---------------------------------------------

    def test_observation_has_required_combat_fields(self, runner):
        """Human profile combat section must include all RL-required fields."""
        _navigate_to_combat(runner)
        if runner.phase != GamePhase.COMBAT:
            pytest.skip("Could not reach combat")

        obs = runner.get_observation()
        combat = obs["combat"]
        assert combat is not None

        required = {
            "player", "enemies", "hand", "energy", "turn", "stance",
            "draw_pile", "discard_pile", "exhaust_pile",
        }
        for field in required:
            assert field in combat, f"Combat section missing required field: {field}"

        # Player sub-fields
        player = combat["player"]
        for pf in ("hp", "max_hp", "block", "statuses"):
            assert pf in player, f"Combat player missing: {pf}"

        # Enemies must have essential fields
        assert len(combat["enemies"]) > 0
        for enemy in combat["enemies"]:
            for ef in ("id", "hp", "max_hp", "block", "move_damage", "move_hits"):
                assert ef in enemy, f"Enemy missing: {ef}"

    # -- Schema version fields ----------------------------------------------

    def test_observation_schema_version_present(self, runner):
        """observation_schema_version must be a semver string in every observation."""
        obs = runner.get_observation()

        assert "observation_schema_version" in obs
        version = obs["observation_schema_version"]
        assert isinstance(version, str)
        parts = version.split(".")
        assert len(parts) == 3, f"Expected semver x.y.z, got {version}"
        assert all(p.isdigit() for p in parts), f"Expected numeric semver parts: {version}"

    def test_observation_action_schema_version_present(self, runner):
        """action_schema_version must be a semver string in every observation."""
        obs = runner.get_observation()

        assert "action_schema_version" in obs
        version = obs["action_schema_version"]
        assert isinstance(version, str)
        parts = version.split(".")
        assert len(parts) == 3, f"Expected semver x.y.z, got {version}"
        assert all(p.isdigit() for p in parts), f"Expected numeric semver parts: {version}"

    # -- Determinism --------------------------------------------------------

    def test_observation_deterministic(self, runner):
        """Same state must produce bit-identical observations (human profile)."""
        obs1 = runner.get_observation()

        runner2 = GameRunner(seed="AGENTTEST", ascension=20, verbose=False)
        obs2 = runner2.get_observation()

        json1 = json.dumps(obs1, sort_keys=True)
        json2 = json.dumps(obs2, sort_keys=True)
        assert json1 == json2, "Identical state must yield identical observations"

    def test_observation_deterministic_across_calls(self, runner):
        """Repeated get_observation on same state returns identical dict."""
        obs1 = runner.get_observation()
        obs2 = runner.get_observation()

        json1 = json.dumps(obs1, sort_keys=True)
        json2 = json.dumps(obs2, sort_keys=True)
        assert json1 == json2

    # -- Profile parameter --------------------------------------------------

    def test_profile_field_present(self, runner):
        """Observation dict must include the profile name."""
        obs_human = runner.get_observation(profile="human")
        assert obs_human.get("profile") == "human"

        obs_debug = runner.get_observation(profile="debug")
        assert obs_debug.get("profile") == "debug"

    def test_human_profile_has_no_debug_key(self, runner):
        """Human profile must not include a debug section."""
        obs = runner.get_observation(profile="human")
        assert "debug" not in obs or obs["debug"] is None

    def test_debug_profile_has_debug_key(self, runner):
        """Debug profile must include a debug section with diagnostic fields."""
        obs = runner.get_observation(profile="debug")
        assert "debug" in obs
        debug = obs["debug"]
        assert isinstance(debug, dict)
        assert "game_over" in debug
        assert "rng_streams" in debug

    def test_debug_profile_superset_of_human(self, runner):
        """Debug profile must contain all fields that human profile has."""
        obs_human = runner.get_observation(profile="human")
        obs_debug = runner.get_observation(profile="debug")

        # Every top-level key in human (except 'profile' and 'debug') must exist in debug
        for key in obs_human:
            if key == "profile":
                continue
            assert key in obs_debug, f"Debug profile missing human key: {key}"
            # For non-None values, they should be equal
            if obs_human[key] is not None and key != "debug":
                human_json = json.dumps(obs_human[key], sort_keys=True)
                debug_json = json.dumps(obs_debug[key], sort_keys=True)
                assert human_json == debug_json, f"Key {key} differs between profiles"

    def test_debug_combat_internals(self, runner):
        """Debug profile in combat includes full pile contents."""
        _navigate_to_combat(runner)
        if runner.phase != GamePhase.COMBAT:
            pytest.skip("Could not reach combat")

        obs = runner.get_observation(profile="debug")
        debug = obs.get("debug", {})
        internals = debug.get("combat_internals", {})

        assert "draw_pile_contents" in internals
        assert "discard_pile_contents" in internals
        assert "exhaust_pile_contents" in internals
        assert isinstance(internals["draw_pile_contents"], list)

    def test_invalid_profile_raises(self, runner):
        """Unknown profile name must raise ValueError."""
        with pytest.raises(ValueError, match="Unknown observation profile"):
            runner.get_observation(profile="nonexistent")

    # -- Default profile is human (backward compat) -------------------------

    def test_default_profile_is_human(self, runner):
        """get_observation() with no args uses human profile."""
        obs_default = runner.get_observation()
        obs_human = runner.get_observation(profile="human")

        assert obs_default.get("profile") == "human"
        json1 = json.dumps(obs_default, sort_keys=True)
        json2 = json.dumps(obs_human, sort_keys=True)
        assert json1 == json2


class TestObservationEncoding:
    """Tests for RL observation encoding utilities."""

    def test_observation_to_array_returns_numpy(self, runner):
        """observation_to_array should return a float32 numpy array."""
        from packages.engine.rl_observations import observation_to_array

        obs = runner.get_observation()
        arr = observation_to_array(obs)

        assert isinstance(arr, np.ndarray)
        assert arr.dtype == np.float32
        assert arr.ndim == 1
        assert arr.shape[0] > 0

    def test_observation_to_array_deterministic(self, runner):
        """Same observation must produce identical arrays."""
        from packages.engine.rl_observations import observation_to_array

        obs = runner.get_observation()
        arr1 = observation_to_array(obs)
        arr2 = observation_to_array(obs)

        np.testing.assert_array_equal(arr1, arr2)

    def test_observation_to_array_differs_across_states(self):
        """Different game states should produce different arrays."""
        from packages.engine.rl_observations import observation_to_array

        r1 = GameRunner(seed="ENC_A", ascension=0, verbose=False)
        r2 = GameRunner(seed="ENC_B", ascension=20, verbose=False)

        arr1 = observation_to_array(r1.get_observation())
        arr2 = observation_to_array(r2.get_observation())

        # At minimum the ascension value differs
        assert not np.array_equal(arr1, arr2)

    def test_array_round_trip_preserves_scalars(self, runner):
        """Encoding then decoding should preserve key scalar values."""
        from packages.engine.rl_observations import (
            observation_to_array, array_to_observation,
        )

        obs = runner.get_observation()
        arr = observation_to_array(obs)
        recovered = array_to_observation(arr)

        assert recovered["run"]["max_hp"] == obs["run"]["max_hp"]
        assert recovered["run"]["gold"] == obs["run"]["gold"]
        assert recovered["run"]["floor"] == obs["run"]["floor"]
        assert recovered["run"]["act"] == obs["run"]["act"]
        assert recovered["run"]["ascension"] == obs["run"]["ascension"]

    def test_encoder_size_is_positive(self):
        """ObservationEncoder.size should be a positive integer."""
        from packages.engine.rl_observations import ObservationEncoder

        enc = ObservationEncoder()
        assert enc.size > 0
        assert isinstance(enc.size, int)

    def test_combat_encoding(self, runner):
        """Combat features should be non-zero when in combat."""
        from packages.engine.rl_observations import ObservationEncoder

        _navigate_to_combat(runner)
        if runner.phase != GamePhase.COMBAT:
            pytest.skip("Could not reach combat")

        enc = ObservationEncoder()
        obs = runner.get_observation()
        arr = enc.observation_to_array(obs)

        # Energy and turn should be non-zero or at least the energy position set
        combat_start = enc._off_combat_scalars
        # At least turn should be >= 1
        assert arr[combat_start + 2] >= 1.0, "Turn should be >= 1 in combat"


# =============================================================================
# Run tests
# =============================================================================

if __name__ == "__main__":
    pytest.main([__file__, "-v"])
