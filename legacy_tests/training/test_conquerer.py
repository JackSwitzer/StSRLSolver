"""Tests for SeedConquerer: 10x parallel beam search on the same seed.

Tests cover:
- Single path completes without error
- Sequential conquer produces 10 paths
- Parallel conquer produces 10 paths
- Best path selection logic (win > floor > HP)
- Divergence tree structure
- Different strategies produce different decision logs
- Deterministic results with same seed
"""

import pytest
from packages.training.conquerer import (
    ConquererResult,
    PathResult,
    SeedConquerer,
    _find_divergence_points,
    _run_path,
)


# =========================================================================
# Helpers
# =========================================================================

SEED = "CONQ01"
FAST_STEPS = 200  # Enough to get through a few floors without being slow


# =========================================================================
# Tests
# =========================================================================


class TestSinglePath:
    """Test that a single path completes and returns valid data."""

    def test_single_path_completes(self):
        result = _run_path(
            seed=SEED,
            path_id=0,
            ascension=20,
            character="Watcher",
            max_steps=FAST_STEPS,
        )
        assert isinstance(result, PathResult)
        assert result.path_id == 0
        assert result.seed == SEED
        assert isinstance(result.won, bool)
        assert result.floors_reached >= 0
        assert result.hp_remaining >= 0
        assert isinstance(result.decision_log, list)
        assert len(result.decision_log) > 0

    def test_single_path_greedy_baseline(self):
        """Path 0 always picks first action (greedy)."""
        result = _run_path(SEED, 0, 20, "Watcher", FAST_STEPS)
        # Every logged action should have been chosen from available alternatives
        for entry in result.decision_log:
            assert entry["alternatives"] >= 1

    def test_single_path_temperature(self):
        """Paths 1-3 use temperature-based sampling."""
        for pid in [1, 2, 3]:
            result = _run_path(SEED, pid, 20, "Watcher", FAST_STEPS)
            assert isinstance(result, PathResult)
            assert result.path_id == pid
            assert len(result.decision_log) > 0

    def test_single_path_heuristic(self):
        """Paths 4-6 use heuristic picking."""
        for pid in [4, 5, 6]:
            result = _run_path(SEED, pid, 20, "Watcher", FAST_STEPS)
            assert isinstance(result, PathResult)
            assert result.path_id == pid

    def test_single_path_weighted(self):
        """Paths 7-9 use weighted random picking."""
        for pid in [7, 8, 9]:
            result = _run_path(SEED, pid, 20, "Watcher", FAST_STEPS)
            assert isinstance(result, PathResult)
            assert result.path_id == pid


class TestSequentialConquer:
    """Test sequential (single-process) conquering."""

    def test_sequential_conquer_10_paths(self):
        conq = SeedConquerer(
            num_paths=10,
            ascension=20,
            parallel=False,
            max_steps=FAST_STEPS,
        )
        result = conq.conquer(SEED)

        assert isinstance(result, ConquererResult)
        assert result.seed == SEED
        assert len(result.paths) == 10
        assert 0 <= result.best_path_id < 10
        assert 0 <= result.win_count <= 10
        assert result.max_floor >= 0
        assert result.elapsed_seconds > 0

    def test_sequential_conquer_fewer_paths(self):
        conq = SeedConquerer(
            num_paths=3,
            ascension=20,
            parallel=False,
            max_steps=FAST_STEPS,
        )
        result = conq.conquer(SEED)
        assert len(result.paths) == 3

    def test_paths_sorted_by_id(self):
        conq = SeedConquerer(
            num_paths=5,
            ascension=20,
            parallel=False,
            max_steps=FAST_STEPS,
        )
        result = conq.conquer(SEED)
        ids = [p.path_id for p in result.paths]
        assert ids == list(range(5))


class TestParallelConquer:
    """Test parallel (multi-process) conquering."""

    def test_parallel_conquer_10_paths(self):
        conq = SeedConquerer(
            num_paths=10,
            ascension=20,
            parallel=True,
            max_steps=FAST_STEPS,
            max_workers=2,
        )
        result = conq.conquer(SEED)

        assert isinstance(result, ConquererResult)
        assert len(result.paths) == 10
        assert 0 <= result.best_path_id < 10

    def test_parallel_matches_sequential_path_count(self):
        conq_seq = SeedConquerer(num_paths=5, parallel=False, max_steps=FAST_STEPS)
        conq_par = SeedConquerer(num_paths=5, parallel=True, max_steps=FAST_STEPS, max_workers=2)

        res_seq = conq_seq.conquer(SEED)
        res_par = conq_par.conquer(SEED)

        assert len(res_seq.paths) == len(res_par.paths)


class TestBestPathSelection:
    """Test that best path selection picks correctly."""

    def test_win_beats_loss(self):
        conq = SeedConquerer(num_paths=2, parallel=False, max_steps=1)

        # Manually construct results to test selection logic
        winner = PathResult(
            path_id=0,
            seed="X",
            won=True,
            floors_reached=5,
            hp_remaining=10,
            total_reward=1.0,
            decision_log=[],
            divergence_points=[],
        )
        loser = PathResult(
            path_id=1,
            seed="X",
            won=False,
            floors_reached=50,
            hp_remaining=50,
            total_reward=0.8,
            decision_log=[],
            divergence_points=[],
        )
        best = conq._select_best([winner, loser])
        assert best.path_id == 0  # Winner beats further-reaching loser

    def test_higher_floor_beats_lower(self):
        conq = SeedConquerer(num_paths=2, parallel=False)

        far = PathResult(
            path_id=0,
            seed="X",
            won=False,
            floors_reached=30,
            hp_remaining=5,
            total_reward=0.5,
            decision_log=[],
            divergence_points=[],
        )
        near = PathResult(
            path_id=1,
            seed="X",
            won=False,
            floors_reached=10,
            hp_remaining=50,
            total_reward=0.2,
            decision_log=[],
            divergence_points=[],
        )
        best = conq._select_best([far, near])
        assert best.path_id == 0  # Floor > HP when both lost

    def test_higher_hp_breaks_tie(self):
        conq = SeedConquerer(num_paths=2, parallel=False)

        healthy = PathResult(
            path_id=0,
            seed="X",
            won=False,
            floors_reached=20,
            hp_remaining=50,
            total_reward=0.3,
            decision_log=[],
            divergence_points=[],
        )
        injured = PathResult(
            path_id=1,
            seed="X",
            won=False,
            floors_reached=20,
            hp_remaining=10,
            total_reward=0.3,
            decision_log=[],
            divergence_points=[],
        )
        best = conq._select_best([healthy, injured])
        assert best.path_id == 0

    def test_empty_results_raises(self):
        conq = SeedConquerer(num_paths=1, parallel=False)
        with pytest.raises(ValueError):
            conq._select_best([])


class TestDivergenceTree:
    """Test divergence tree structure."""

    def test_divergence_tree_structure(self):
        conq = SeedConquerer(
            num_paths=5,
            ascension=20,
            parallel=False,
            max_steps=FAST_STEPS,
        )
        result = conq.conquer(SEED)

        tree = result.divergence_tree
        assert "total_paths" in tree
        assert tree["total_paths"] == 5
        assert "divergent_floors" in tree
        assert isinstance(tree["divergent_floors"], dict)

    def test_divergence_points_populated(self):
        conq = SeedConquerer(
            num_paths=3,
            ascension=20,
            parallel=False,
            max_steps=FAST_STEPS,
        )
        result = conq.conquer(SEED)

        # Path 0 has no divergence points (it IS the baseline)
        assert result.paths[0].divergence_points == []
        # Other paths may have divergence points
        for p in result.paths[1:]:
            assert isinstance(p.divergence_points, list)

    def test_find_divergence_points_identical(self):
        """Identical logs produce no divergence."""
        log = [
            {"floor": 1, "action_id": "a1", "phase": "COMBAT", "alternatives": 3},
            {"floor": 1, "action_id": "a2", "phase": "COMBAT", "alternatives": 2},
        ]
        assert _find_divergence_points(log, log) == []

    def test_find_divergence_points_different(self):
        """Different action at floor 1 is flagged."""
        baseline = [
            {"floor": 1, "action_id": "a1", "phase": "COMBAT", "alternatives": 3},
        ]
        compare = [
            {"floor": 1, "action_id": "b1", "phase": "COMBAT", "alternatives": 3},
        ]
        divs = _find_divergence_points(baseline, compare)
        assert 1 in divs


class TestStrategiesDiverge:
    """Test that different strategies produce different decision logs."""

    def test_different_strategies_produce_different_results(self):
        """At least some strategies should diverge from the greedy baseline."""
        results = []
        for pid in range(4):  # 0=greedy, 1-3=temperature
            r = _run_path(SEED, pid, 20, "Watcher", FAST_STEPS)
            results.append(r)

        baseline_actions = [e["action_id"] for e in results[0].decision_log]

        any_different = False
        for r in results[1:]:
            other_actions = [e["action_id"] for e in r.decision_log]
            if other_actions != baseline_actions:
                any_different = True
                break

        assert any_different, "All strategies produced identical action sequences"


class TestDeterminism:
    """Test that same seed + same path_id gives deterministic results."""

    def test_deterministic_with_same_seed(self):
        r1 = _run_path(SEED, 0, 20, "Watcher", FAST_STEPS)
        r2 = _run_path(SEED, 0, 20, "Watcher", FAST_STEPS)

        assert r1.won == r2.won
        assert r1.floors_reached == r2.floors_reached
        assert r1.hp_remaining == r2.hp_remaining
        assert len(r1.decision_log) == len(r2.decision_log)

        # Every action should match
        for e1, e2 in zip(r1.decision_log, r2.decision_log):
            assert e1["action_id"] == e2["action_id"]
            assert e1["floor"] == e2["floor"]

    def test_deterministic_temperature_path(self):
        """Temperature paths use floor-seeded RNG so should be deterministic."""
        r1 = _run_path(SEED, 2, 20, "Watcher", FAST_STEPS)
        r2 = _run_path(SEED, 2, 20, "Watcher", FAST_STEPS)

        assert r1.floors_reached == r2.floors_reached
        assert len(r1.decision_log) == len(r2.decision_log)
        for e1, e2 in zip(r1.decision_log, r2.decision_log):
            assert e1["action_id"] == e2["action_id"]
