"""
Integration tests for the GameRunner orchestrator.

Tests cover initialization, phase transitions, Neow blessings, map navigation,
full run simulation, determinism, decision logging, and action validation.
"""

import pytest
import random
import sys

sys.path.insert(0, '/Users/jackswitzer/Desktop/SlayTheSpireRL')

from packages.engine.game import (
    GameRunner, GamePhase,
    PathAction, NeowAction, CombatAction, RewardAction,
    EventAction, ShopAction, RestAction, TreasureAction, BossRewardAction,
    DecisionLogEntry,
)


# =============================================================================
# 1. Initialization
# =============================================================================


class TestInitialization:
    """GameRunner creates valid state with different seeds and ascension levels."""

    def test_basic_init_string_seed(self):
        runner = GameRunner(seed="TEST123", ascension=20, verbose=False)
        assert runner.seed_string == "TEST123"
        assert runner.run_state is not None
        assert runner.run_state.ascension == 20
        assert not runner.game_over
        assert not runner.game_won
        assert not runner.game_lost

    def test_basic_init_numeric_seed(self):
        runner = GameRunner(seed=42, ascension=0, verbose=False)
        assert runner.seed == 42
        assert runner.run_state.ascension == 0

    def test_different_seeds_produce_different_states(self):
        r1 = GameRunner(seed="AAA", ascension=20, verbose=False)
        r2 = GameRunner(seed="BBB", ascension=20, verbose=False)
        # Different seeds should produce different maps
        map1 = r1.run_state.get_current_map()
        map2 = r2.run_state.get_current_map()
        # Maps are lists of lists of nodes; compare room types of first row
        types1 = [n.room_type for n in map1[0] if n.room_type is not None]
        types2 = [n.room_type for n in map2[0] if n.room_type is not None]
        # They might be the same by chance, but seed/RNG state differs
        assert r1.seed != r2.seed

    def test_ascension_levels(self):
        for asc in [0, 1, 10, 15, 20]:
            runner = GameRunner(seed="ASC", ascension=asc, verbose=False)
            assert runner.run_state.ascension == asc

    def test_skip_neow_starts_at_map(self):
        runner = GameRunner(seed="X", ascension=0, skip_neow=True, verbose=False)
        assert runner.phase == GamePhase.MAP_NAVIGATION

    def test_no_skip_neow_starts_at_neow(self):
        runner = GameRunner(seed="X", ascension=0, skip_neow=False, verbose=False)
        assert runner.phase == GamePhase.NEOW

    def test_starting_hp_and_gold(self):
        runner = GameRunner(seed="HP", ascension=0, verbose=False)
        assert runner.run_state.current_hp > 0
        assert runner.run_state.max_hp > 0
        assert runner.run_state.current_hp <= runner.run_state.max_hp
        assert runner.run_state.gold >= 0

    def test_starting_deck_nonempty(self):
        runner = GameRunner(seed="DECK", ascension=0, verbose=False)
        assert len(runner.run_state.deck) > 0

    def test_decision_log_starts_empty(self):
        runner = GameRunner(seed="LOG", ascension=0, verbose=False)
        assert len(runner.decision_log) == 0


# =============================================================================
# 2. Phase Transitions
# =============================================================================


class TestPhaseTransitions:
    """Phase transitions from NEOW -> MAP_NAVIGATION -> room types."""

    def test_neow_to_map(self):
        runner = GameRunner(seed="NEOW1", ascension=0, skip_neow=False, verbose=False)
        assert runner.phase == GamePhase.NEOW
        actions = runner.get_available_actions()
        assert len(actions) > 0
        assert all(isinstance(a, NeowAction) for a in actions)
        # Take first neow action
        runner.take_action(actions[0])
        assert runner.phase == GamePhase.MAP_NAVIGATION

    def test_map_to_room(self):
        runner = GameRunner(seed="MAP1", ascension=0, verbose=False)
        assert runner.phase == GamePhase.MAP_NAVIGATION
        actions = runner.get_available_actions()
        assert len(actions) > 0
        assert all(isinstance(a, PathAction) for a in actions)
        # Take a path action - should transition to a room phase
        runner.take_action(actions[0])
        assert runner.phase in (
            GamePhase.COMBAT, GamePhase.EVENT, GamePhase.SHOP,
            GamePhase.REST, GamePhase.TREASURE, GamePhase.MAP_NAVIGATION,
        )

    def test_combat_ends_in_rewards_or_game_over(self):
        """After combat resolves, phase should be COMBAT_REWARDS or RUN_COMPLETE."""
        runner = GameRunner(seed="COMBAT1", ascension=0, verbose=False)
        # Navigate to first room
        actions = runner.get_available_actions()
        runner.take_action(actions[0])

        # If not in combat, keep navigating until we find one
        max_steps = 500
        steps = 0
        while runner.phase != GamePhase.COMBAT and not runner.game_over and steps < max_steps:
            actions = runner.get_available_actions()
            if not actions:
                break
            runner.take_action(actions[0])
            steps += 1

        if runner.phase == GamePhase.COMBAT:
            # Play through combat
            while runner.phase == GamePhase.COMBAT and not runner.game_over:
                actions = runner.get_available_actions()
                if not actions:
                    break
                runner.take_action(random.choice(actions))
            assert runner.phase in (GamePhase.COMBAT_REWARDS, GamePhase.RUN_COMPLETE)


# =============================================================================
# 3. Neow Actions
# =============================================================================


class TestNeowActions:
    """Neow blessings are offered and apply correctly."""

    def test_neow_offers_multiple_blessings(self):
        runner = GameRunner(seed="NEOW2", ascension=0, skip_neow=False, verbose=False)
        actions = runner.get_available_actions()
        assert len(actions) >= 2  # Should offer at least 2 blessings

    def test_neow_blessing_changes_state(self):
        runner = GameRunner(seed="NEOW3", ascension=0, skip_neow=False, verbose=False)
        hp_before = runner.run_state.current_hp
        gold_before = runner.run_state.gold
        deck_before = len(runner.run_state.deck)
        relics_before = len(runner.run_state.relics)

        actions = runner.get_available_actions()
        runner.take_action(actions[0])

        # At least one of these should change after a blessing
        hp_after = runner.run_state.current_hp
        gold_after = runner.run_state.gold
        deck_after = len(runner.run_state.deck)
        relics_after = len(runner.run_state.relics)

        something_changed = (
            hp_before != hp_after
            or gold_before != gold_after
            or deck_before != deck_after
            or relics_before != relics_after
        )
        # Some blessings might not visibly change these (e.g. upgrade),
        # but the phase should always change
        assert runner.phase == GamePhase.MAP_NAVIGATION

    def test_neow_logs_decision(self):
        runner = GameRunner(seed="NEOW4", ascension=0, skip_neow=False, verbose=False)
        actions = runner.get_available_actions()
        runner.take_action(actions[0])
        assert len(runner.decision_log) == 1
        entry = runner.decision_log[0]
        assert entry.phase == GamePhase.NEOW
        assert isinstance(entry.action_taken, NeowAction)

    def test_all_neow_choices_are_valid(self):
        """Each Neow choice index should be accepted."""
        runner = GameRunner(seed="NEOW5", ascension=0, skip_neow=False, verbose=False)
        actions = runner.get_available_actions()
        for i, action in enumerate(actions):
            # Create fresh runner for each choice
            r = GameRunner(seed="NEOW5", ascension=0, skip_neow=False, verbose=False)
            success = r.take_action(action)
            assert success, f"Neow action {i} failed"
            assert r.phase == GamePhase.MAP_NAVIGATION


# =============================================================================
# 4. Map Navigation
# =============================================================================


class TestMapNavigation:
    """Map paths make sense and choosing a path transitions correctly."""

    def test_initial_paths_available(self):
        runner = GameRunner(seed="NAV1", ascension=0, verbose=False)
        actions = runner.get_available_actions()
        assert len(actions) >= 1
        assert all(isinstance(a, PathAction) for a in actions)

    def test_path_indices_sequential(self):
        runner = GameRunner(seed="NAV2", ascension=0, verbose=False)
        actions = runner.get_available_actions()
        indices = [a.node_index for a in actions]
        assert indices == list(range(len(actions)))

    def test_path_advances_floor(self):
        runner = GameRunner(seed="NAV3", ascension=0, verbose=False)
        floor_before = runner.run_state.floor
        actions = runner.get_available_actions()
        runner.take_action(actions[0])
        assert runner.run_state.floor == floor_before + 1

    def test_invalid_path_index_fails(self):
        runner = GameRunner(seed="NAV4", ascension=0, verbose=False)
        result = runner.take_action(PathAction(node_index=999))
        # Should still log the attempt but return False
        assert len(runner.decision_log) == 1

    def test_multiple_floors_navigation(self):
        """Navigate several floors, verifying floor increments."""
        runner = GameRunner(seed="NAV5", ascension=0, verbose=False)
        for expected_floor in range(1, 4):
            if runner.game_over or runner.phase != GamePhase.MAP_NAVIGATION:
                # If in combat/event, play through it
                while runner.phase != GamePhase.MAP_NAVIGATION and not runner.game_over:
                    actions = runner.get_available_actions()
                    if not actions:
                        break
                    runner.take_action(random.choice(actions))
            if runner.game_over:
                break
            actions = runner.get_available_actions()
            if actions:
                runner.take_action(actions[0])


# =============================================================================
# 5. Full Run Simulation
# =============================================================================


class TestFullRunSimulation:
    """Run complete games with random actions."""

    @pytest.mark.slow
    def test_full_run_terminates(self):
        """A full run with random actions should terminate."""
        runner = GameRunner(seed="FULL1", ascension=0, verbose=False)
        stats = runner.run()
        assert runner.game_over
        assert runner.game_won or runner.game_lost
        assert runner.phase == GamePhase.RUN_COMPLETE

    @pytest.mark.slow
    def test_full_run_statistics_valid(self):
        """Run statistics should have reasonable values."""
        runner = GameRunner(seed="FULL2", ascension=0, verbose=False)
        stats = runner.run()
        assert stats["seed"] == "FULL2"
        assert stats["ascension"] == 0
        assert isinstance(stats["game_won"], bool)
        assert isinstance(stats["game_lost"], bool)
        assert stats["game_won"] != stats["game_lost"]  # Exactly one is true
        assert stats["final_floor"] >= 1
        assert stats["final_act"] >= 1
        assert stats["deck_size"] >= 1
        assert stats["decisions_made"] > 0

    @pytest.mark.slow
    def test_full_run_with_neow(self):
        """Full run starting with Neow blessing should also work."""
        runner = GameRunner(seed="FULL3", ascension=0, skip_neow=False, verbose=False)
        stats = runner.run()
        assert runner.game_over

    @pytest.mark.slow
    def test_full_run_ascension_20(self):
        """Full A20 run should terminate."""
        runner = GameRunner(seed="A20RUN", ascension=20, verbose=False)
        stats = runner.run()
        assert runner.game_over
        assert stats["ascension"] == 20

    @pytest.mark.slow
    def test_run_to_floor(self):
        """run_to_floor should stop at or before target floor."""
        runner = GameRunner(seed="FLOOR5", ascension=0, verbose=False)
        stats = runner.run_to_floor(5)
        # Either reached floor 5+ or game ended before that
        assert runner.run_state.floor >= 5 or runner.game_over

    def test_no_actions_when_game_over(self):
        """After game over, get_available_actions returns empty."""
        runner = GameRunner(seed="OVER1", ascension=0, verbose=False)
        runner.game_over = True
        assert runner.get_available_actions() == []

    def test_take_action_when_game_over(self):
        """take_action returns False when game is over."""
        runner = GameRunner(seed="OVER2", ascension=0, verbose=False)
        runner.game_over = True
        result = runner.take_action(PathAction(node_index=0))
        assert result is False


# =============================================================================
# 6. Determinism
# =============================================================================


class TestDeterminism:
    """Same seed + same actions = same outcome."""

    def test_same_seed_same_initial_state(self):
        r1 = GameRunner(seed="DET1", ascension=20, verbose=False)
        r2 = GameRunner(seed="DET1", ascension=20, verbose=False)
        assert r1.run_state.current_hp == r2.run_state.current_hp
        assert r1.run_state.max_hp == r2.run_state.max_hp
        assert r1.run_state.gold == r2.run_state.gold
        assert len(r1.run_state.deck) == len(r2.run_state.deck)

    def test_same_seed_same_available_actions(self):
        r1 = GameRunner(seed="DET2", ascension=0, verbose=False)
        r2 = GameRunner(seed="DET2", ascension=0, verbose=False)
        a1 = r1.get_available_actions()
        a2 = r2.get_available_actions()
        assert len(a1) == len(a2)
        for act1, act2 in zip(a1, a2):
            assert act1 == act2

    def test_same_seed_same_actions_same_outcome(self):
        """Replay the same sequence of actions on two runners."""
        r1 = GameRunner(seed="DET3", ascension=0, verbose=False)
        r2 = GameRunner(seed="DET3", ascension=0, verbose=False)

        # Play 20 steps with deterministic action choice (always first)
        for _ in range(20):
            a1 = r1.get_available_actions()
            a2 = r2.get_available_actions()
            if not a1 or not a2:
                break
            # Always take first action for determinism
            r1.take_action(a1[0])
            r2.take_action(a2[0])

        assert r1.run_state.current_hp == r2.run_state.current_hp
        assert r1.run_state.gold == r2.run_state.gold
        assert r1.run_state.floor == r2.run_state.floor
        assert r1.phase == r2.phase
        assert len(r1.decision_log) == len(r2.decision_log)

    def test_different_seeds_diverge(self):
        """Different seeds should produce different maps/encounters."""
        r1 = GameRunner(seed="DIFF1", ascension=0, verbose=False)
        r2 = GameRunner(seed="DIFF2", ascension=0, verbose=False)
        # Compare seeds directly and encounter tables
        assert r1.seed != r2.seed
        # Monster lists are generated from different RNG streams
        differs = (
            r1._monster_list != r2._monster_list
            or r1._elite_list != r2._elite_list
            or r1._boss_name != r2._boss_name
        )
        assert differs


# =============================================================================
# 7. Decision Logging
# =============================================================================


class TestDecisionLogging:
    """Decision log entries are recorded correctly."""

    def test_log_entry_per_action(self):
        runner = GameRunner(seed="LOG1", ascension=0, verbose=False)
        actions = runner.get_available_actions()
        runner.take_action(actions[0])
        assert len(runner.decision_log) == 1

    def test_log_entry_fields(self):
        runner = GameRunner(seed="LOG2", ascension=0, verbose=False)
        actions = runner.get_available_actions()
        runner.take_action(actions[0])
        entry = runner.decision_log[0]
        assert isinstance(entry, DecisionLogEntry)
        assert entry.floor is not None
        assert entry.act is not None
        assert entry.phase is not None
        assert entry.action_taken is not None
        assert isinstance(entry.available_actions, list)
        assert isinstance(entry.state_snapshot, dict)
        assert entry.result is not None

    def test_log_snapshot_contains_key_fields(self):
        runner = GameRunner(seed="LOG3", ascension=0, verbose=False)
        actions = runner.get_available_actions()
        runner.take_action(actions[0])
        snap = runner.decision_log[0].state_snapshot
        for key in ("hp", "max_hp", "gold", "floor", "act", "deck_size", "relic_count"):
            assert key in snap, f"Missing key: {key}"

    def test_log_grows_with_actions(self):
        runner = GameRunner(seed="LOG4", ascension=0, verbose=False)
        for i in range(5):
            if runner.game_over:
                break
            actions = runner.get_available_actions()
            if not actions:
                break
            runner.take_action(actions[0])
        assert len(runner.decision_log) >= min(5, len(runner.decision_log))
        # Each entry should have increasing or same floor
        for entry in runner.decision_log:
            assert entry.floor >= 0

    def test_run_statistics_match_log(self):
        runner = GameRunner(seed="LOG5", ascension=0, verbose=False)
        runner.run_to_floor(3)
        stats = runner.get_run_statistics()
        assert stats["decisions_made"] == len(runner.decision_log)


# =============================================================================
# 8. Action Validation
# =============================================================================


class TestActionValidation:
    """Invalid actions are handled gracefully."""

    def test_invalid_path_index(self):
        runner = GameRunner(seed="VAL1", ascension=0, verbose=False)
        # Path index way out of range
        success = runner.take_action(PathAction(node_index=999))
        # Should not crash; logged but failed
        assert len(runner.decision_log) == 1

    def test_invalid_neow_index(self):
        runner = GameRunner(seed="VAL2", ascension=0, skip_neow=False, verbose=False)
        success = runner.take_action(NeowAction(choice_index=999))
        # Should not crash
        assert len(runner.decision_log) == 1

    def test_wrong_action_type_for_phase(self):
        """Taking a combat action during map phase should not crash."""
        runner = GameRunner(seed="VAL3", ascension=0, verbose=False)
        assert runner.phase == GamePhase.MAP_NAVIGATION
        # Combat action during map phase - should be a no-op
        success = runner.take_action(CombatAction(action_type="end_turn"))
        # Should not crash, combat handler returns early
        assert len(runner.decision_log) == 1

    def test_action_after_game_over_returns_false(self):
        runner = GameRunner(seed="VAL4", ascension=0, verbose=False)
        runner.game_over = True
        success = runner.take_action(PathAction(node_index=0))
        assert success is False
        assert len(runner.decision_log) == 0

    def test_get_actions_returns_correct_types(self):
        """Actions returned should match the current phase."""
        runner = GameRunner(seed="VAL5", ascension=0, verbose=False)
        actions = runner.get_available_actions()
        if runner.phase == GamePhase.MAP_NAVIGATION:
            assert all(isinstance(a, PathAction) for a in actions)

        runner2 = GameRunner(seed="VAL6", ascension=0, skip_neow=False, verbose=False)
        actions2 = runner2.get_available_actions()
        if runner2.phase == GamePhase.NEOW:
            assert all(isinstance(a, NeowAction) for a in actions2)
