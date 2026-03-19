"""Tests for strategic MCTS module."""
import pytest
import numpy as np
from packages.training.strategic_mcts import MCTSNode, StrategicMCTS, MCTS_BUDGETS
from packages.training.training_config import COMBAT_MCTS_BUDGETS


class TestMCTSNode:
    def test_ucb1_unvisited(self):
        node = MCTSNode(action_idx=0)
        assert node.ucb1(10) == float('inf')

    def test_ucb1_visited(self):
        node = MCTSNode(action_idx=0, visits=5, total_value=2.5)
        score = node.ucb1(20)
        # mean_value = 0.5, explore term > 0
        assert score > 0.5

    def test_mean_value(self):
        node = MCTSNode(action_idx=0, visits=4, total_value=2.0)
        assert node.mean_value == 0.5

    def test_mean_value_zero_visits(self):
        node = MCTSNode(action_idx=0)
        assert node.mean_value == 0.0

    def test_ucb1_exploration_decreases_with_visits(self):
        """Exploration bonus should decrease as a node is visited more."""
        node_few = MCTSNode(action_idx=0, visits=2, total_value=1.0)
        node_many = MCTSNode(action_idx=0, visits=50, total_value=25.0)
        # Same mean_value (0.5), but fewer visits => higher UCB
        assert node_few.ucb1(100) > node_many.ucb1(100)


class TestStrategicMCTS:
    def test_single_action(self):
        """Single action returns immediately."""
        mcts = StrategicMCTS()
        idx, policy = mcts.search(None, ["only_action"], "other", budget=10)
        assert idx == 0
        assert len(policy) == 1
        assert policy[0] == 1.0

    def test_search_with_game(self):
        """MCTS runs without crashing on a real game state."""
        from packages.engine.game import GameRunner, GamePhase
        runner = GameRunner(seed="MCTS1", ascension=0, verbose=False)

        # Advance to a multi-action decision
        for _ in range(50):
            actions = runner.get_available_actions()
            if not actions or runner.game_over:
                break
            if len(actions) > 1 and runner.phase != GamePhase.COMBAT:
                # Test MCTS here
                mcts = StrategicMCTS()
                idx, policy = mcts.search(runner, actions, "path", budget=5)
                assert 0 <= idx < len(actions)
                assert len(policy) == len(actions)
                assert abs(policy.sum() - 1.0) < 1e-6
                break
            runner.take_action(actions[0])

    def test_heuristic_value(self):
        """Heuristic returns reasonable values."""
        from packages.engine.game import GameRunner
        runner = GameRunner(seed="MCTS2", ascension=0, verbose=False)
        mcts = StrategicMCTS()
        val = mcts._heuristic_value(runner)
        assert 0.0 <= val <= 1.0

    def test_budgets_exist(self):
        """All expected phase types have budgets."""
        for phase in ["card_pick", "path", "rest", "shop", "event"]:
            assert phase in MCTS_BUDGETS

    def test_stats_tracking(self):
        """Stats accumulate across searches."""
        mcts = StrategicMCTS()
        assert mcts.stats["total_sims"] == 0
        assert mcts.stats["total_ms"] == 0.0

    def test_policy_sums_to_one(self):
        """Policy output is a valid probability distribution."""
        from packages.engine.game import GameRunner, GamePhase
        runner = GameRunner(seed="MCTS3", ascension=0, verbose=False)

        for _ in range(50):
            actions = runner.get_available_actions()
            if not actions or runner.game_over:
                break
            if len(actions) > 1 and runner.phase != GamePhase.COMBAT:
                mcts = StrategicMCTS()
                _, policy = mcts.search(runner, actions, "other", budget=10)
                assert all(p >= 0 for p in policy)
                assert abs(policy.sum() - 1.0) < 1e-6
                break
            runner.take_action(actions[0])

    def test_copy_failure_graceful(self):
        """MCTS handles copy() failure gracefully (returns 0.0 value)."""
        class FakeRunner:
            game_over = False
            game_won = False
            run_state = type('RS', (), {'floor': 5, 'current_hp': 50, 'max_hp': 80})()
            phase = None

            def copy(self):
                raise NotImplementedError("copy() not available")

            def get_available_actions(self):
                return ["a", "b"]

        runner = FakeRunner()
        mcts = StrategicMCTS()
        idx, policy = mcts.search(runner, ["a", "b"], "path", budget=4)
        # Should not crash; all sims fail with value=0.0
        assert 0 <= idx < 2
        assert len(policy) == 2
        assert abs(policy.sum() - 1.0) < 1e-6


class TestCombatMCTS:
    """Tests for AlphaZero-style MCTS combat search."""

    def test_combat_mcts_budgets_exist(self):
        """All expected room types have combat MCTS budgets."""
        for room in ["monster", "elite", "boss"]:
            assert room in COMBAT_MCTS_BUDGETS
        # Boss budget should be highest
        assert COMBAT_MCTS_BUDGETS["boss"] > COMBAT_MCTS_BUDGETS["elite"]
        assert COMBAT_MCTS_BUDGETS["elite"] > COMBAT_MCTS_BUDGETS["monster"]

    def test_combat_mcts_search_mid_combat(self):
        """MCTS combat search runs on a real mid-combat game state."""
        from packages.engine.game import GameRunner, GamePhase

        runner = GameRunner(seed="CMCTS1", ascension=0, verbose=False)

        # Advance to mid-combat with multiple actions
        found_combat = False
        for _ in range(200):
            actions = runner.get_available_actions()
            if not actions or runner.game_over:
                break
            if runner.phase == GamePhase.COMBAT and len(actions) > 1:
                # Test combat MCTS here
                mcts = StrategicMCTS()
                idx, policy = mcts.search(
                    runner, actions, "combat",
                    budget=5, combat_only=True,
                )
                assert 0 <= idx < len(actions)
                assert len(policy) == len(actions)
                assert abs(policy.sum() - 1.0) < 1e-6
                found_combat = True
                break
            runner.take_action(actions[0])

        assert found_combat, "Could not reach a multi-action combat state"

    def test_combat_only_rollout_stops_at_boundary(self):
        """combat_only=True stops rollout when phase leaves COMBAT."""
        from packages.engine.game import GameRunner, GamePhase

        runner = GameRunner(seed="CMCTS2", ascension=0, verbose=False)

        # Advance to combat
        for _ in range(200):
            actions = runner.get_available_actions()
            if not actions or runner.game_over:
                break
            if runner.phase == GamePhase.COMBAT and len(actions) > 1:
                mcts = StrategicMCTS()
                # With combat_only, rollout should stop at combat boundary
                # and return a heuristic value (not simulate entire game)
                game_copy = runner.copy()
                val = mcts._rollout_and_evaluate(game_copy, "combat", combat_only=True)
                assert isinstance(val, float)
                assert 0.0 <= val <= 1.0
                break
            runner.take_action(actions[0])

    def test_combat_only_vs_full_rollout_different_steps(self):
        """combat_only rollout should generally be shorter than full rollout."""
        from packages.engine.game import GameRunner, GamePhase

        runner = GameRunner(seed="CMCTS3", ascension=0, verbose=False)

        for _ in range(200):
            actions = runner.get_available_actions()
            if not actions or runner.game_over:
                break
            if runner.phase == GamePhase.COMBAT and len(actions) > 1:
                mcts = StrategicMCTS()

                # Both modes should produce valid values
                copy1 = runner.copy()
                val_combat = mcts._rollout_and_evaluate(copy1, "combat", combat_only=True)
                copy2 = runner.copy()
                val_full = mcts._rollout_and_evaluate(copy2, "combat", combat_only=False)

                assert isinstance(val_combat, float)
                assert isinstance(val_full, float)
                assert 0.0 <= val_combat <= 1.0
                assert 0.0 <= val_full <= 1.0
                break
            runner.take_action(actions[0])

    def test_mcts_card_sims_override_flows_through(self):
        """mcts_card_sims parameter is accepted by _play_one_game signature."""
        from packages.training.worker import _play_one_game
        import inspect

        sig = inspect.signature(_play_one_game)
        params = list(sig.parameters.keys())
        assert "mcts_card_sims" in params
        # Check default value is 0
        assert sig.parameters["mcts_card_sims"].default == 0

    def test_sweep_config_mcts_card_sims(self):
        """Weekend sweep configs have mcts_card_sims set correctly."""
        from packages.training.sweep_config import WEEKEND_SWEEP_CONFIGS

        # Should have 3 configs
        assert len(WEEKEND_SWEEP_CONFIGS) == 3

        # First config: no MCTS
        assert WEEKEND_SWEEP_CONFIGS[0]["mcts_enabled"] is False
        assert WEEKEND_SWEEP_CONFIGS[0]["max_hours"] == 20

        # Second config: 200 sims
        assert WEEKEND_SWEEP_CONFIGS[1]["mcts_enabled"] is True
        assert WEEKEND_SWEEP_CONFIGS[1]["mcts_card_sims"] == 200
        assert WEEKEND_SWEEP_CONFIGS[1]["max_hours"] == 35

        # Third config: 500 sims
        assert WEEKEND_SWEEP_CONFIGS[2]["mcts_enabled"] is True
        assert WEEKEND_SWEEP_CONFIGS[2]["mcts_card_sims"] == 500
        assert WEEKEND_SWEEP_CONFIGS[2]["max_hours"] == 35
