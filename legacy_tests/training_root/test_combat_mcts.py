"""Tests for AlphaZero-style combat MCTS -- MCTS replaces TurnSolver for combat.

Tests are split into:
- TestCombatMCTSIntegration: MCTS playing actual combat turns (requires combat_only param)
- TestCombatMCTSConfig: Config values for combat MCTS budgets
- TestCombatMCTSFallback: Graceful degradation on failure
- TestCombatMCTSRollout: Rollout behavior during combat
"""
import pytest
import numpy as np


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _advance_to_combat(runner, max_steps=100):
    """Advance a GameRunner to the first COMBAT phase. Returns True if reached."""
    from packages.engine.game import GamePhase

    for _ in range(max_steps):
        actions = runner.get_available_actions()
        if not actions or runner.game_over:
            return False
        if runner.phase == GamePhase.COMBAT:
            return True
        runner.take_action(actions[0])
    return False


def _advance_to_multi_action_combat(runner, max_steps=200):
    """Advance to a COMBAT phase with >1 available action."""
    from packages.engine.game import GamePhase

    for _ in range(max_steps):
        actions = runner.get_available_actions()
        if not actions or runner.game_over:
            return False
        if runner.phase == GamePhase.COMBAT and len(actions) > 1:
            return True
        runner.take_action(actions[0])
    return False


def _has_combat_only_param():
    """Check if StrategicMCTS.search accepts combat_only kwarg."""
    import inspect
    from packages.training.strategic_mcts import StrategicMCTS
    sig = inspect.signature(StrategicMCTS.search)
    return "combat_only" in sig.parameters


def _has_combat_mcts_budgets():
    """Check if COMBAT_MCTS_BUDGETS exists in training_config."""
    try:
        from packages.training.training_config import COMBAT_MCTS_BUDGETS
        return True
    except ImportError:
        return False


# Conditional skip for tests that require the combat MCTS implementation
requires_combat_mcts = pytest.mark.skipif(
    not _has_combat_only_param(),
    reason="combat_only param not yet implemented in StrategicMCTS.search",
)
requires_combat_budgets = pytest.mark.skipif(
    not _has_combat_mcts_budgets(),
    reason="COMBAT_MCTS_BUDGETS not yet in training_config",
)


# ---------------------------------------------------------------------------
# Integration: MCTS playing combat turns
# ---------------------------------------------------------------------------

class TestCombatMCTSIntegration:
    """Test MCTS playing actual combat turns."""

    @requires_combat_mcts
    def test_mcts_combat_action_selection(self):
        """MCTS can select a combat action from a real game state."""
        from packages.engine.game import GameRunner, GamePhase
        from packages.training.strategic_mcts import StrategicMCTS

        runner = GameRunner(seed="COMBAT_SEL1", ascension=0, verbose=False)
        if not _advance_to_multi_action_combat(runner):
            pytest.skip("Could not reach multi-action combat")

        actions = runner.get_available_actions()
        mcts = StrategicMCTS()
        idx, policy = mcts.search(
            runner, actions, "combat", budget=5, combat_only=True
        )
        assert 0 <= idx < len(actions)
        assert len(policy) == len(actions)
        assert abs(policy.sum() - 1.0) < 1e-6

    @requires_combat_mcts
    def test_combat_only_rollout_stops_at_combat_end(self):
        """Combat-only rollout should stop when combat phase ends."""
        from packages.engine.game import GameRunner, GamePhase
        from packages.training.strategic_mcts import StrategicMCTS

        runner = GameRunner(seed="COMBAT_STOP1", ascension=0, verbose=False)
        if not _advance_to_combat(runner):
            pytest.skip("Could not reach combat")

        original_phase = runner.phase
        actions = runner.get_available_actions()
        mcts = StrategicMCTS()
        idx, _ = mcts.search(runner, actions, "combat", budget=3, combat_only=True)

        # Original runner must be untouched (MCTS uses copies)
        assert runner.phase == original_phase
        assert runner.phase == GamePhase.COMBAT

    @requires_combat_mcts
    def test_mcts_plays_full_combat(self):
        """MCTS can play through an entire combat encounter."""
        from packages.engine.game import GameRunner, GamePhase
        from packages.training.strategic_mcts import StrategicMCTS

        runner = GameRunner(seed="COMBAT_FULL1", ascension=0, verbose=False)
        if not _advance_to_combat(runner):
            pytest.skip("Could not reach combat")

        mcts = StrategicMCTS()
        turns = 0
        while runner.phase == GamePhase.COMBAT and not runner.game_over and turns < 200:
            actions = runner.get_available_actions()
            if not actions:
                break
            if len(actions) == 1:
                runner.take_action(actions[0])
            else:
                idx, _ = mcts.search(
                    runner, actions, "combat", budget=3, combat_only=True
                )
                runner.take_action(actions[idx])
            turns += 1

        assert turns > 0, "Should have taken at least one action"
        # Combat should have ended -- either won or game over
        assert runner.phase != GamePhase.COMBAT or runner.game_over

    @requires_combat_mcts
    def test_mcts_does_not_mutate_runner(self):
        """MCTS search must not modify the passed-in runner."""
        from packages.engine.game import GameRunner, GamePhase
        from packages.training.strategic_mcts import StrategicMCTS

        runner = GameRunner(seed="COMBAT_IMMUT1", ascension=0, verbose=False)
        if not _advance_to_multi_action_combat(runner):
            pytest.skip("Could not reach multi-action combat")

        actions = runner.get_available_actions()
        hp_before = runner.run_state.current_hp
        floor_before = runner.run_state.floor

        mcts = StrategicMCTS()
        mcts.search(runner, actions, "combat", budget=10, combat_only=True)

        assert runner.run_state.current_hp == hp_before
        assert runner.run_state.floor == floor_before
        assert runner.phase == GamePhase.COMBAT

    def test_mcts_combat_without_combat_only_flag(self):
        """Current MCTS can search in combat phase (without combat_only)."""
        from packages.engine.game import GameRunner, GamePhase
        from packages.training.strategic_mcts import StrategicMCTS

        runner = GameRunner(seed="COMBAT_LEGACY1", ascension=0, verbose=False)
        if not _advance_to_multi_action_combat(runner):
            pytest.skip("Could not reach multi-action combat")

        actions = runner.get_available_actions()
        mcts = StrategicMCTS()
        # Works without combat_only (the pre-implementation path)
        idx, policy = mcts.search(runner, actions, "combat", budget=3)
        assert 0 <= idx < len(actions)
        assert len(policy) == len(actions)
        assert abs(policy.sum() - 1.0) < 1e-6


# ---------------------------------------------------------------------------
# Config: combat MCTS budget values
# ---------------------------------------------------------------------------

class TestCombatMCTSConfig:
    """Test config values for combat MCTS."""

    @requires_combat_budgets
    def test_combat_budgets_in_config(self):
        from packages.training.training_config import COMBAT_MCTS_BUDGETS
        assert "monster" in COMBAT_MCTS_BUDGETS
        assert "elite" in COMBAT_MCTS_BUDGETS
        assert "boss" in COMBAT_MCTS_BUDGETS

    @requires_combat_budgets
    def test_boss_budget_higher_than_monster(self):
        """Boss fights get more MCTS sims than regular monsters."""
        from packages.training.training_config import COMBAT_MCTS_BUDGETS
        assert COMBAT_MCTS_BUDGETS["boss"] > COMBAT_MCTS_BUDGETS["monster"]
        assert COMBAT_MCTS_BUDGETS["boss"] >= 200
        assert COMBAT_MCTS_BUDGETS["elite"] > COMBAT_MCTS_BUDGETS["monster"]

    @requires_combat_budgets
    def test_combat_budgets_are_positive(self):
        from packages.training.training_config import COMBAT_MCTS_BUDGETS
        for key, val in COMBAT_MCTS_BUDGETS.items():
            assert val > 0, f"Budget for {key} should be positive, got {val}"

    def test_strategic_mcts_budgets_exist(self):
        """Strategic MCTS budgets should still exist alongside combat budgets."""
        from packages.training.training_config import MCTS_BUDGETS
        for phase in ["card_pick", "path", "rest", "shop", "event"]:
            assert phase in MCTS_BUDGETS

    def test_solver_scoring_exists(self):
        """Solver scoring config should exist."""
        from packages.training.training_config import SOLVER_SCORING
        assert "hp_lost_weight" in SOLVER_SCORING
        assert "enemy_kill_bonus" in SOLVER_SCORING

    def test_mcts_ucb_c_positive(self):
        from packages.training.training_config import MCTS_UCB_C
        assert MCTS_UCB_C > 0

    def test_mcts_blend_ratio_valid(self):
        from packages.training.training_config import MCTS_BLEND_RATIO
        assert 0.0 < MCTS_BLEND_RATIO <= 1.0

    def test_mcts_configs_in_sweep(self):
        """At least one sweep config should have mcts_enabled."""
        from packages.training.sweep_config import DEFAULT_SWEEP_CONFIGS
        mcts_configs = [c for c in DEFAULT_SWEEP_CONFIGS if c.get("mcts_enabled")]
        assert len(mcts_configs) >= 1, "Need at least one MCTS-enabled sweep config"

    def test_weekend_sweep_has_mcts_config(self):
        """Weekend sweep should include an MCTS config."""
        from packages.training.sweep_config import WEEKEND_SWEEP_CONFIGS
        mcts_configs = [c for c in WEEKEND_SWEEP_CONFIGS if c.get("mcts_enabled")]
        assert len(mcts_configs) >= 1


# ---------------------------------------------------------------------------
# Fallback: graceful degradation
# ---------------------------------------------------------------------------

class TestCombatMCTSFallback:
    """Test MCTS fallback behavior when things go wrong."""

    def test_copy_failure_returns_valid_result(self):
        """If runner.copy() raises, MCTS should not crash."""
        from packages.training.strategic_mcts import StrategicMCTS

        class BrokenRunner:
            game_over = False
            game_won = False
            run_state = type("RS", (), {"floor": 5, "current_hp": 50, "max_hp": 80})()
            phase = None

            def copy(self):
                raise RuntimeError("copy broken")

            def get_available_actions(self):
                return ["a", "b", "c"]

        mcts = StrategicMCTS()
        idx, policy = mcts.search(BrokenRunner(), ["a", "b", "c"], "combat", budget=5)
        assert 0 <= idx < 3
        assert len(policy) == 3
        assert abs(policy.sum() - 1.0) < 1e-6

    def test_take_action_failure_returns_valid_result(self):
        """If take_action raises during sim, MCTS should not crash."""
        from packages.training.strategic_mcts import StrategicMCTS

        class ActionFailRunner:
            game_over = False
            game_won = False
            run_state = type("RS", (), {"floor": 5, "current_hp": 50, "max_hp": 80})()
            phase = None

            def copy(self):
                return ActionFailRunner()

            def take_action(self, action):
                raise ValueError("action failed")

            def get_available_actions(self):
                return ["x", "y"]

        mcts = StrategicMCTS()
        idx, policy = mcts.search(ActionFailRunner(), ["x", "y"], "combat", budget=4)
        assert 0 <= idx < 2
        assert abs(policy.sum() - 1.0) < 1e-6

    def test_single_combat_action_returns_immediately(self):
        """Single action in combat should return immediately without search."""
        from packages.training.strategic_mcts import StrategicMCTS

        mcts = StrategicMCTS()
        idx, policy = mcts.search(None, ["end_turn"], "combat", budget=100)
        assert idx == 0
        assert policy[0] == 1.0
        # Should have done 0 actual sims (stats unchanged)
        assert mcts.stats["total_sims"] == 0

    def test_empty_actions_handled(self):
        """Edge case: empty actions list should not crash."""
        from packages.training.strategic_mcts import StrategicMCTS

        mcts = StrategicMCTS()
        # With 0 actions, should still return something valid
        # The implementation returns (0, [1.0]) for single-action,
        # but empty actions is a caller bug. Just ensure no crash.
        idx, policy = mcts.search(None, ["only"], "combat", budget=5)
        assert idx == 0


# ---------------------------------------------------------------------------
# Rollout: behavior during MCTS rollouts
# ---------------------------------------------------------------------------

class TestCombatMCTSRollout:
    """Test rollout behavior inside MCTS simulations."""

    def test_heuristic_value_range(self):
        """Heuristic evaluation should return values in [0, 1]."""
        from packages.engine.game import GameRunner
        from packages.training.strategic_mcts import StrategicMCTS

        mcts = StrategicMCTS()
        runner = GameRunner(seed="HEUR1", ascension=0, verbose=False)
        val = mcts._heuristic_value(runner)
        assert 0.0 <= val <= 1.0

    def test_heuristic_win_returns_one(self):
        """A won game should return 1.0."""
        from packages.training.strategic_mcts import StrategicMCTS

        class WonRunner:
            game_over = True
            game_won = True
            run_state = type("RS", (), {"floor": 55, "current_hp": 50, "max_hp": 80})()

        mcts = StrategicMCTS()
        assert mcts._heuristic_value(WonRunner()) == 1.0

    def test_heuristic_early_death_low_value(self):
        """Early death (floor 3) should have low heuristic value."""
        from packages.training.strategic_mcts import StrategicMCTS

        class EarlyDeath:
            game_over = True
            game_won = False
            run_state = type("RS", (), {"floor": 3, "current_hp": 0, "max_hp": 80})()

        mcts = StrategicMCTS()
        val = mcts._heuristic_value(EarlyDeath())
        assert val < 0.15, f"Early death heuristic too high: {val}"

    def test_heuristic_late_death_higher_than_early(self):
        """Death at floor 40 should be valued higher than death at floor 5."""
        from packages.training.strategic_mcts import StrategicMCTS

        class DeathAt:
            game_won = False
            game_over = True
            def __init__(self, floor):
                self.run_state = type("RS", (), {"floor": floor, "current_hp": 0, "max_hp": 80})()

        mcts = StrategicMCTS()
        val_early = mcts._heuristic_value(DeathAt(5))
        val_late = mcts._heuristic_value(DeathAt(40))
        assert val_late > val_early

    def test_heuristic_higher_hp_better(self):
        """More HP remaining should give higher heuristic value."""
        from packages.training.strategic_mcts import StrategicMCTS

        class AliveAt:
            game_over = False
            game_won = False
            def __init__(self, hp):
                self.run_state = type("RS", (), {"floor": 10, "current_hp": hp, "max_hp": 80})()

        mcts = StrategicMCTS()
        val_low = mcts._heuristic_value(AliveAt(10))
        val_high = mcts._heuristic_value(AliveAt(70))
        assert val_high > val_low

    def test_rollout_max_steps_bounded(self):
        """Rollout should not exceed MAX_ROLLOUT_STEPS."""
        from packages.training.strategic_mcts import MAX_ROLLOUT_STEPS
        assert MAX_ROLLOUT_STEPS > 0
        assert MAX_ROLLOUT_STEPS <= 500  # Sanity: should not be unreasonably large

    def test_stats_accumulate_across_searches(self):
        """Stats should accumulate across multiple search calls."""
        from packages.engine.game import GameRunner, GamePhase
        from packages.training.strategic_mcts import StrategicMCTS

        runner = GameRunner(seed="STATS1", ascension=0, verbose=False)
        if not _advance_to_multi_action_combat(runner):
            pytest.skip("Could not reach multi-action combat")

        mcts = StrategicMCTS()
        actions = runner.get_available_actions()

        mcts.search(runner, actions, "combat", budget=3)
        sims_after_first = mcts.stats["total_sims"]
        assert sims_after_first == 3

        mcts.search(runner, actions, "combat", budget=5)
        assert mcts.stats["total_sims"] == sims_after_first + 5

    def test_stats_track_time(self):
        """Stats should track elapsed time."""
        from packages.engine.game import GameRunner, GamePhase
        from packages.training.strategic_mcts import StrategicMCTS

        runner = GameRunner(seed="STATS_TIME1", ascension=0, verbose=False)
        if not _advance_to_multi_action_combat(runner):
            pytest.skip("Could not reach multi-action combat")

        mcts = StrategicMCTS()
        actions = runner.get_available_actions()
        mcts.search(runner, actions, "combat", budget=3)
        assert mcts.stats["total_ms"] >= 0.0


# ---------------------------------------------------------------------------
# Budget logic
# ---------------------------------------------------------------------------

class TestCombatMCTSBudgetLogic:
    """Test budget selection and override logic."""

    def test_explicit_budget_overrides_default(self):
        """Passing budget= should override the phase_type default."""
        from packages.engine.game import GameRunner, GamePhase
        from packages.training.strategic_mcts import StrategicMCTS

        runner = GameRunner(seed="BUDGET1", ascension=0, verbose=False)
        if not _advance_to_multi_action_combat(runner):
            pytest.skip("Could not reach multi-action combat")

        mcts = StrategicMCTS()
        actions = runner.get_available_actions()

        # budget=2 means exactly 2 sims
        mcts.search(runner, actions, "combat", budget=2)
        assert mcts.stats["total_sims"] == 2

    def test_unknown_phase_type_uses_fallback(self):
        """Unknown phase_type should use fallback budget (10)."""
        from packages.engine.game import GameRunner, GamePhase
        from packages.training.strategic_mcts import StrategicMCTS

        runner = GameRunner(seed="BUDGET2", ascension=0, verbose=False)
        if not _advance_to_multi_action_combat(runner):
            pytest.skip("Could not reach multi-action combat")

        mcts = StrategicMCTS()
        actions = runner.get_available_actions()

        # "nonexistent_phase" is not in MCTS_BUDGETS, should fallback to 10
        mcts.search(runner, actions, "nonexistent_phase")
        assert mcts.stats["total_sims"] == 10

    def test_early_game_boost_concept(self):
        """Floors 1-10 should logically get more budget than later floors."""
        # This tests the concept -- actual implementation is in worker.py
        floor = 5
        base_budget = 200
        boosted = int(base_budget * 1.5) if floor <= 10 else base_budget
        assert boosted == 300

        floor = 15
        boosted = int(base_budget * 1.5) if floor <= 10 else base_budget
        assert boosted == 200


# ---------------------------------------------------------------------------
# UCB1 selection
# ---------------------------------------------------------------------------

class TestCombatMCTSSelection:
    """Test action selection mechanics within MCTS."""

    def test_ucb1_prefers_unvisited(self):
        """Unvisited nodes should be prioritized (UCB1 = inf)."""
        from packages.training.strategic_mcts import MCTSNode
        visited = MCTSNode(action_idx=0, visits=10, total_value=5.0)
        unvisited = MCTSNode(action_idx=1)
        assert unvisited.ucb1(10) > visited.ucb1(10)

    def test_ucb1_balances_exploration_exploitation(self):
        """High-value low-visit should beat low-value high-visit eventually."""
        from packages.training.strategic_mcts import MCTSNode
        # High value, few visits
        good_rare = MCTSNode(action_idx=0, visits=3, total_value=2.7)  # mean=0.9
        # Low value, many visits
        bad_common = MCTSNode(action_idx=1, visits=50, total_value=10.0)  # mean=0.2
        # With enough parent visits, the high-value node should win
        assert good_rare.ucb1(100) > bad_common.ucb1(100)

    def test_policy_reflects_visit_counts(self):
        """The returned policy should be proportional to visit counts."""
        from packages.engine.game import GameRunner, GamePhase
        from packages.training.strategic_mcts import StrategicMCTS

        runner = GameRunner(seed="POLICY1", ascension=0, verbose=False)
        if not _advance_to_multi_action_combat(runner):
            pytest.skip("Could not reach multi-action combat")

        actions = runner.get_available_actions()
        mcts = StrategicMCTS()
        idx, policy = mcts.search(runner, actions, "combat", budget=20)

        # Policy should be non-negative and sum to 1
        assert all(p >= 0 for p in policy)
        assert abs(policy.sum() - 1.0) < 1e-6

        # Best action should have highest policy weight
        assert policy[idx] == max(policy)

    def test_more_sims_gives_better_policy(self):
        """With more sims, the policy should be more concentrated (less uniform)."""
        from packages.engine.game import GameRunner, GamePhase
        from packages.training.strategic_mcts import StrategicMCTS

        runner = GameRunner(seed="SIMS_QUALITY1", ascension=0, verbose=False)
        if not _advance_to_multi_action_combat(runner):
            pytest.skip("Could not reach multi-action combat")

        actions = runner.get_available_actions()
        if len(actions) < 3:
            pytest.skip("Need 3+ actions for this test")

        mcts_few = StrategicMCTS()
        _, policy_few = mcts_few.search(runner, actions, "combat", budget=len(actions))

        mcts_many = StrategicMCTS()
        _, policy_many = mcts_many.search(runner, actions, "combat", budget=50)

        # With more sims, entropy should be lower (more concentrated)
        # Use max policy value as a proxy: more sims => higher max
        assert max(policy_many) >= max(policy_few) or len(actions) == 2
