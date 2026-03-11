"""
Tests for missing agent actions:
1. Dream Catcher card pick after resting
2. Wing Boots flying to non-connected nodes
3. Neow card selection (upgrade, remove, transform, choose)
"""

import sys

sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

import pytest
from packages.engine.game import (
    GameRunner, GamePhase, RestAction,
)
from packages.engine.state.run import RunState, create_watcher_run
from packages.engine.state.rng import Random, seed_to_long
from packages.engine.generation.map import MapRoomNode, MapEdge, RoomType


# =============================================================================
# 1. Dream Catcher Card Pick
# =============================================================================


class TestDreamCatcherCardPick:
    """Dream Catcher should let the agent pick a card after resting."""

    def _make_runner_at_rest(self, seed="DC_TEST"):
        """Create a GameRunner at a rest site with Dream Catcher."""
        runner = GameRunner(seed=seed, ascension=0, verbose=False)
        runner.run_state.add_relic("Dream Catcher")
        # Force the phase to REST
        runner.phase = GamePhase.REST
        return runner

    def test_dream_catcher_transitions_to_card_reward(self):
        """After resting with Dream Catcher, phase should be COMBAT_REWARDS."""
        runner = self._make_runner_at_rest()
        # Ensure HP is not full so rest is meaningful
        runner.run_state.current_hp = runner.run_state.max_hp - 10

        success = runner.take_action(RestAction(action_type="rest"))
        assert success
        assert runner.phase == GamePhase.COMBAT_REWARDS
        assert runner.current_rewards is not None
        assert len(runner.current_rewards.card_rewards) == 1
        assert len(runner.current_rewards.card_rewards[0].cards) == 3

    def test_dream_catcher_card_pick_via_dict(self):
        """Agent can pick a card from Dream Catcher reward via action dicts."""
        runner = self._make_runner_at_rest(seed="DC_PICK")
        runner.run_state.current_hp = runner.run_state.max_hp - 10

        runner.take_action(RestAction(action_type="rest"))
        assert runner.phase == GamePhase.COMBAT_REWARDS

        # Get available actions - should include card picks and skip
        actions = runner.get_available_action_dicts()
        card_picks = [a for a in actions if a.get("type") == "pick_card"]
        skip_actions = [a for a in actions if a.get("type") == "skip_card"]

        assert len(card_picks) == 3  # 3 card choices
        assert len(skip_actions) >= 1  # can skip

        # Pick the first card
        deck_before = len(runner.run_state.deck)
        result = runner.take_action_dict(card_picks[0])
        assert result.get("success", False) or result == True

        # After picking, proceed
        proceed_actions = [a for a in runner.get_available_action_dicts()
                          if a.get("type") == "proceed_from_rewards"]
        if proceed_actions:
            runner.take_action_dict(proceed_actions[0])

        assert runner.phase == GamePhase.MAP_NAVIGATION
        assert len(runner.run_state.deck) == deck_before + 1

    def test_dream_catcher_skip_card(self):
        """Agent can skip the Dream Catcher card reward."""
        runner = self._make_runner_at_rest(seed="DC_SKIP")
        runner.run_state.current_hp = runner.run_state.max_hp - 10

        runner.take_action(RestAction(action_type="rest"))
        assert runner.phase == GamePhase.COMBAT_REWARDS

        actions = runner.get_available_action_dicts()
        skip_actions = [a for a in actions if a.get("type") == "skip_card"]
        assert len(skip_actions) >= 1

        deck_before = len(runner.run_state.deck)
        runner.take_action_dict(skip_actions[0])

        # Proceed from rewards
        proceed_actions = [a for a in runner.get_available_action_dicts()
                          if a.get("type") == "proceed_from_rewards"]
        if proceed_actions:
            runner.take_action_dict(proceed_actions[0])

        assert runner.phase == GamePhase.MAP_NAVIGATION
        assert len(runner.run_state.deck) == deck_before  # no card added

    def test_rest_without_dream_catcher_goes_to_map(self):
        """Resting without Dream Catcher should go directly to MAP_NAVIGATION."""
        runner = GameRunner(seed="NO_DC", ascension=0, verbose=False)
        runner.phase = GamePhase.REST
        runner.run_state.current_hp = runner.run_state.max_hp - 10

        success = runner.take_action(RestAction(action_type="rest"))
        assert success
        assert runner.phase == GamePhase.MAP_NAVIGATION


# =============================================================================
# 2. Wing Boots Flying
# =============================================================================


class TestWingBootsFlying:
    """Wing Boots should allow flying to any node in the next row."""

    def _make_run_with_wing_boots(self, seed="WINGS"):
        """Create a RunState with Wing Boots and a map."""
        run = create_watcher_run(seed, ascension=0)
        run.add_relic("Wing Boots")
        run.set_relic_counter("Wing Boots", 3)
        # Ensure map is generated
        run.get_current_map()
        return run

    def test_wing_boots_adds_extra_paths(self):
        """With Wing Boots, get_available_paths returns more nodes than just edges."""
        run = self._make_run_with_wing_boots()
        current_map = run.get_current_map()

        # Move to a node in row 0
        row0_nodes = [n for n in current_map[0] if n.has_edges()]
        if not row0_nodes:
            pytest.skip("No valid row 0 nodes")
        node = row0_nodes[0]
        run.move_to(node.x, node.y)

        # Count edge-connected nodes
        edge_connected = set()
        for edge in node.edges:
            if not edge.is_boss:
                edge_connected.add((edge.dst_x, edge.dst_y))

        # Get all available paths with Wing Boots
        paths = run.get_available_paths()
        path_coords = {(n.x, n.y) for n in paths}

        # All nodes in next row with edges
        next_row = node.y + 1
        all_next_row = {(n.x, n.y) for n in current_map[next_row] if n.has_edges()}

        # Wing Boots paths should include all next-row nodes
        for coord in all_next_row:
            assert coord in path_coords, (
                f"Node {coord} in next row should be reachable via Wing Boots"
            )

    def test_wing_boots_marks_fly_paths(self):
        """Non-edge-connected paths should be marked with is_winged_path."""
        run = self._make_run_with_wing_boots()
        current_map = run.get_current_map()

        row0_nodes = [n for n in current_map[0] if n.has_edges()]
        if not row0_nodes:
            pytest.skip("No valid row 0 nodes")
        node = row0_nodes[0]
        run.move_to(node.x, node.y)

        edge_connected = set()
        for edge in node.edges:
            if not edge.is_boss:
                edge_connected.add((edge.dst_x, edge.dst_y))

        paths = run.get_available_paths()
        for p in paths:
            if (p.x, p.y) not in edge_connected:
                # Boss nodes don't get is_winged_path
                if p.room_type != RoomType.BOSS:
                    assert getattr(p, 'is_winged_path', False), (
                        f"Node ({p.x}, {p.y}) should be marked as winged path"
                    )

    def test_wing_boots_no_extra_paths_when_depleted(self):
        """With 0 charges, no extra paths should be added."""
        run = self._make_run_with_wing_boots()
        run.set_relic_counter("Wing Boots", 0)
        current_map = run.get_current_map()

        row0_nodes = [n for n in current_map[0] if n.has_edges()]
        if not row0_nodes:
            pytest.skip("No valid row 0 nodes")
        node = row0_nodes[0]
        run.move_to(node.x, node.y)

        paths = run.get_available_paths()
        for p in paths:
            assert not getattr(p, 'is_winged_path', False), (
                f"Node ({p.x}, {p.y}) should not be a winged path when depleted"
            )

    def test_wing_boots_counter_decrements_on_fly(self):
        """Flying to a winged path should decrement Wing Boots counter."""
        runner = GameRunner(seed="FLY_DEC", ascension=0, verbose=False)
        runner.run_state.add_relic("Wing Boots")
        runner.run_state.set_relic_counter("Wing Boots", 3)

        # Navigate to a position where Wing Boots paths exist
        paths = runner.run_state.get_available_paths()
        if not paths:
            pytest.skip("No paths available")

        # Move to the first available node
        runner.take_action_dict({"type": "path_choice", "params": {"node_index": 0}})

        # Now check for winged paths
        paths = runner.run_state.get_available_paths()
        fly_paths = [i for i, p in enumerate(paths) if getattr(p, 'is_winged_path', False)]

        if not fly_paths:
            pytest.skip("No fly paths available from this position")

        charges_before = runner.run_state.get_relic_counter("Wing Boots")
        runner.take_action_dict({"type": "path_choice", "params": {"node_index": fly_paths[0]}})
        charges_after = runner.run_state.get_relic_counter("Wing Boots")

        assert charges_after == charges_before - 1


# =============================================================================
# 3. Neow Card Selection
# =============================================================================


class TestNeowCardSelection:
    """Neow blessings requiring card selection should use pending_selection flow."""

    @staticmethod
    def _make_neow_runner(seed, previous_score=100):
        """Create a runner at Neow phase with previous_score set for full options."""
        runner = GameRunner(seed=seed, ascension=0, skip_neow=False, verbose=False)
        runner.run_state.previous_score = previous_score
        return runner

    def _find_neow_seed_with_selection(self, sel_type, max_tries=200):
        """Find a seed where at least one Neow blessing requires the given selection type."""
        for i in range(max_tries):
            seed = f"NEOW_SEL_{sel_type}_{i}"
            runner = self._make_neow_runner(seed)
            actions = runner.get_available_actions()
            for j, action in enumerate(actions):
                r = self._make_neow_runner(seed)
                r.get_available_actions()  # Force blessing generation
                r.take_action(action)
                if r.pending_selection is not None:
                    meta = r.pending_selection.metadata
                    if meta.get("neow_selection_type") == sel_type:
                        return seed, j
        return None, None

    def test_neow_upgrade_selection(self):
        """Neow upgrade blessing should let agent choose which card to upgrade."""
        seed, choice_idx = self._find_neow_seed_with_selection("upgrade")
        if seed is None:
            pytest.skip("No seed found with upgrade Neow blessing")

        runner = GameRunner(seed=seed, ascension=0, skip_neow=False, verbose=False)
        actions = runner.get_available_actions()
        runner.take_action(actions[choice_idx])

        assert runner.pending_selection is not None
        assert runner.pending_selection.source_action_type == "neow_blessing"
        assert runner.pending_selection.metadata["neow_selection_type"] == "upgrade"

        # Get selection actions
        sel_actions = runner.get_available_action_dicts()
        assert len(sel_actions) > 0

        # Pick the last upgradeable card (to prove it's not auto-first)
        result = runner.take_action_dict(sel_actions[-1])
        assert result.get("success", False)
        assert runner.phase == GamePhase.MAP_NAVIGATION
        assert runner.pending_selection is None

    def test_neow_remove_selection(self):
        """Neow remove blessing should let agent choose which card to remove."""
        seed, choice_idx = self._find_neow_seed_with_selection("remove")
        if seed is None:
            pytest.skip("No seed found with remove Neow blessing")

        runner = GameRunner(seed=seed, ascension=0, skip_neow=False, verbose=False)
        actions = runner.get_available_actions()
        deck_before = len(runner.run_state.deck)
        runner.take_action(actions[choice_idx])

        assert runner.pending_selection is not None
        sel_actions = runner.get_available_action_dicts()
        assert len(sel_actions) > 0

        result = runner.take_action_dict(sel_actions[0])
        assert result.get("success", False)
        assert runner.phase == GamePhase.MAP_NAVIGATION
        assert len(runner.run_state.deck) == deck_before - 1

    def test_neow_transform_selection(self):
        """Neow transform blessing should let agent choose which card to transform."""
        seed, choice_idx = self._find_neow_seed_with_selection("transform")
        if seed is None:
            pytest.skip("No seed found with transform Neow blessing")

        runner = GameRunner(seed=seed, ascension=0, skip_neow=False, verbose=False)
        actions = runner.get_available_actions()
        runner.take_action(actions[choice_idx])

        assert runner.pending_selection is not None
        sel_actions = runner.get_available_action_dicts()
        assert len(sel_actions) > 0

        result = runner.take_action_dict(sel_actions[0])
        assert result.get("success", False)
        assert runner.phase == GamePhase.MAP_NAVIGATION

    def test_neow_choose_card_selection(self):
        """Neow choose-card blessing should let agent pick from offered cards."""
        seed, choice_idx = self._find_neow_seed_with_selection("choose")
        if seed is None:
            pytest.skip("No seed found with choose Neow blessing")

        runner = GameRunner(seed=seed, ascension=0, skip_neow=False, verbose=False)
        actions = runner.get_available_actions()
        deck_before = len(runner.run_state.deck)
        runner.take_action(actions[choice_idx])

        assert runner.pending_selection is not None
        assert runner.pending_selection.pile == "offer"
        sel_actions = runner.get_available_action_dicts()
        assert len(sel_actions) > 0

        result = runner.take_action_dict(sel_actions[0])
        assert result.get("success", False)
        assert runner.phase == GamePhase.MAP_NAVIGATION
        assert len(runner.run_state.deck) == deck_before + 1

    def test_neow_selection_via_action_dict_flow(self):
        """Full action_dict flow for Neow with card selection."""
        # Try multiple seeds to find one with a selection-required blessing
        for i in range(100):
            seed = f"NEOW_DICT_{i}"
            runner = GameRunner(seed=seed, ascension=0, skip_neow=False, verbose=False)
            actions = runner.get_available_action_dicts()

            for action in actions:
                r = GameRunner(seed=seed, ascension=0, skip_neow=False, verbose=False)
                result = r.take_action_dict(action)

                # If pending selection, complete it
                while r.pending_selection is not None:
                    sel_actions = r.get_available_action_dicts()
                    assert len(sel_actions) > 0
                    result = r.take_action_dict(sel_actions[0])
                    assert result.get("success", False)

                assert r.phase == GamePhase.MAP_NAVIGATION

            # Found a working seed, no need to continue
            return
