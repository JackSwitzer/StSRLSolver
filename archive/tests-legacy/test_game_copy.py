"""Tests for GameRunner.copy() — deep copy for MCTS simulation."""
import pytest
from packages.engine.game import GameRunner, GamePhase


class TestGameRunnerCopy:
    def test_copy_basic(self):
        """Copy creates independent game state."""
        runner = GameRunner(seed="COPY1", ascension=0, verbose=False)
        clone = runner.copy()

        assert clone.seed == runner.seed
        assert clone.phase == runner.phase
        assert clone.run_state.current_hp == runner.run_state.current_hp
        assert clone.game_over == runner.game_over
        assert clone.verbose is False  # Always silent

    def test_copy_independence(self):
        """Mutations on copy don't affect original."""
        runner = GameRunner(seed="COPY2", ascension=0, verbose=False)
        original_hp = runner.run_state.current_hp

        clone = runner.copy()
        clone.run_state.current_hp = 1

        assert runner.run_state.current_hp == original_hp
        assert clone.run_state.current_hp == 1

    def test_copy_rng_independence(self):
        """RNG streams diverge after copy."""
        runner = GameRunner(seed="COPY3", ascension=0, verbose=False)
        clone = runner.copy()

        # Advance clone's RNG
        v1 = clone.misc_rng.random(100)
        v2 = runner.misc_rng.random(100)

        # Same initial state means same first value
        assert v1 == v2

        # But they're independent objects
        assert clone.misc_rng is not runner.misc_rng

    def test_copy_play_forward(self):
        """Can play the copy forward independently."""
        runner = GameRunner(seed="COPY4", ascension=0, verbose=False)

        # Play a few steps
        for _ in range(5):
            actions = runner.get_available_actions()
            if not actions or runner.game_over:
                break
            runner.take_action(actions[0])

        original_floor = runner.run_state.floor
        clone = runner.copy()

        # Play clone forward
        for _ in range(20):
            actions = clone.get_available_actions()
            if not actions or clone.game_over:
                break
            clone.take_action(actions[0])

        # Original unchanged
        assert runner.run_state.floor == original_floor
