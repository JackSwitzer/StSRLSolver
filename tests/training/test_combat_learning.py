"""Tests for combat learning changes: EndTurn trust, boss budget, solver scores."""

from packages.training.training_config import SOLVER_BUDGETS


def test_boss_solver_budget_is_120s():
    """Boss solver should have 120s base budget for deeper search."""
    base_ms, _, _ = SOLVER_BUDGETS["boss"]
    assert base_ms == 120_000.0


def test_solver_adapter_has_last_solver_scores():
    """TurnSolverAdapter should expose last_solver_scores after pick_action."""
    from packages.training.turn_solver import TurnSolverAdapter

    adapter = TurnSolverAdapter(time_budget_ms=5.0, node_budget=100)
    assert hasattr(adapter, "last_solver_scores")
    assert adapter.last_solver_scores == []


def test_solver_adapter_reset_clears_scores():
    """Reset should clear solver scores."""
    from packages.training.turn_solver import TurnSolverAdapter

    adapter = TurnSolverAdapter(time_budget_ms=5.0, node_budget=100)
    adapter.last_solver_scores = [("test", 1.0)]
    adapter.reset()
    assert adapter.last_solver_scores == []
