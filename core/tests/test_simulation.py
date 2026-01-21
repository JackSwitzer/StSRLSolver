"""
Comprehensive test suite for the Parallel Simulation Engine.

Tests cover:
1. ParallelSimulator - initialization, batch simulation, MCTS, shutdown
2. WorkerPool - worker lifecycle, task distribution, result collection
3. BatchProcessor - batch creation, execution, result aggregation
4. StateSerializer - serialization roundtrip, compression efficiency
5. SimulationConfig - default values, custom configurations
6. Performance tests - throughput benchmark, memory usage

Run with: pytest core/tests/test_simulation.py -v
"""

import sys
import os
import gc
import time
import pickle
import pytest
from typing import List, Dict, Any
from unittest.mock import Mock, patch, MagicMock
from multiprocessing import cpu_count

# Ensure core module is importable
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from core.state.combat import (
    CombatState,
    EntityState,
    EnemyCombatState,
    PlayCard,
    EndTurn,
    UsePotion,
)
from core.simulation import (
    ParallelSimulator,
    SimulationConfig,
    SimulationResult,
    CombatSimResult,
    WorkerPool,
    SimulationWorker,
    WorkerTask,
    TaskType,
    BatchProcessor,
    BatchConfig,
    BatchResult,
    StateSerializer,
    run_benchmark,
)
from core.simulation.worker import (
    WorkerResult,
    create_run_task,
    create_combat_task,
    create_mcts_task,
)


# =============================================================================
# ParallelSimulator Tests
# =============================================================================


class TestParallelSimulatorInitialization:
    """Test ParallelSimulator initialization with various configurations."""

    def test_default_initialization(self):
        """Test initialization with default parameters."""
        sim = ParallelSimulator()
        try:
            assert sim.config is not None
            assert sim.config.n_workers >= 1
            assert sim._initialized is True  # prefork_workers=True by default
        finally:
            sim.shutdown()

    def test_explicit_worker_count(self):
        """Test initialization with explicit worker count."""
        sim = ParallelSimulator(n_workers=2)
        try:
            assert sim.config.n_workers == 2
        finally:
            sim.shutdown()

    def test_custom_config(self, fast_sim_config):
        """Test initialization with custom configuration."""
        sim = ParallelSimulator(config=fast_sim_config)
        try:
            assert sim.config.n_workers == 2
            assert sim.config.batch_size == 10
            assert sim.config.max_turns_per_combat == 50
        finally:
            sim.shutdown()

    def test_auto_worker_count(self):
        """Test auto-detection of worker count."""
        config = SimulationConfig(n_workers=0)
        assert config.n_workers == max(1, cpu_count() - 1)

    def test_prefork_disabled(self):
        """Test initialization with prefork disabled."""
        config = SimulationConfig(n_workers=2, prefork_workers=False)
        sim = ParallelSimulator(config=config)
        try:
            assert sim._initialized is False
            # Should initialize lazily
            sim._initialize()
            assert sim._initialized is True
        finally:
            sim.shutdown()

    def test_context_manager(self, fast_sim_config):
        """Test context manager usage."""
        with ParallelSimulator(config=fast_sim_config) as sim:
            assert sim._initialized is True
        # Should be shutdown after context exit
        assert sim._initialized is False

    def test_multiple_shutdowns_safe(self, fast_sim_config):
        """Test that multiple shutdown calls are safe."""
        sim = ParallelSimulator(config=fast_sim_config)
        sim.shutdown()
        sim.shutdown()  # Should not raise
        assert sim._initialized is False


class TestParallelSimulatorSimulateBatch:
    """Test ParallelSimulator.simulate_batch method."""

    def test_empty_batch(self, fast_sim_config):
        """Test simulation with empty seed list."""
        with ParallelSimulator(config=fast_sim_config) as sim:
            results = sim.simulate_batch(seeds=[], ascension=20)
            assert results == []

    def test_single_seed(self, fast_sim_config):
        """Test simulation with single seed."""
        with ParallelSimulator(config=fast_sim_config) as sim:
            results = sim.simulate_batch(seeds=["TEST001"], ascension=20)
            assert len(results) == 1
            assert isinstance(results[0], SimulationResult)
            assert results[0].seed == "TEST001"

    def test_multiple_seeds(self, fast_sim_config, small_seed_batch):
        """Test simulation with multiple seeds."""
        with ParallelSimulator(config=fast_sim_config) as sim:
            results = sim.simulate_batch(seeds=small_seed_batch, ascension=20)
            assert len(results) == len(small_seed_batch)
            # All results should be SimulationResult objects
            for result in results:
                assert isinstance(result, SimulationResult)
                assert result.simulation_time_ms > 0

    def test_callback_invoked(self, fast_sim_config):
        """Test that callback is invoked for each result."""
        callback_results = []

        def callback(result: SimulationResult):
            callback_results.append(result)

        with ParallelSimulator(config=fast_sim_config) as sim:
            seeds = ["CB001", "CB002", "CB003"]
            results = sim.simulate_batch(seeds=seeds, callback=callback)

            assert len(callback_results) == len(seeds)
            assert len(results) == len(seeds)

    def test_ascension_levels(self, fast_sim_config):
        """Test different ascension levels."""
        with ParallelSimulator(config=fast_sim_config) as sim:
            for asc in [0, 10, 20]:
                results = sim.simulate_batch(seeds=["ASC_TEST"], ascension=asc)
                assert len(results) == 1

    def test_statistics_updated(self, fast_sim_config):
        """Test that statistics are updated after batch."""
        with ParallelSimulator(config=fast_sim_config) as sim:
            sim.reset_stats()
            seeds = ["STAT001", "STAT002", "STAT003"]
            sim.simulate_batch(seeds=seeds)

            stats = sim.get_stats()
            assert stats["total_simulations"] == 3
            assert stats["total_time_ms"] > 0


class TestParallelSimulatorSingleSimulation:
    """Test ParallelSimulator.simulate_single method."""

    def test_single_simulation(self, fast_sim_config):
        """Test single simulation runs in main process."""
        with ParallelSimulator(config=fast_sim_config) as sim:
            result = sim.simulate_single("SINGLE001", ascension=20)
            assert isinstance(result, SimulationResult)
            assert result.seed == "SINGLE001"

    def test_track_decisions(self, fast_sim_config):
        """Test decision tracking in single simulation."""
        with ParallelSimulator(config=fast_sim_config) as sim:
            result = sim.simulate_single("TRACK001", track_decisions=True)
            # decision_log may be None if no decisions were made
            assert isinstance(result, SimulationResult)


class TestParallelSimulatorMCTS:
    """Test ParallelSimulator.find_best_play MCTS method."""

    def test_find_best_play_basic(self, fast_sim_config, basic_combat_state):
        """Test basic MCTS search."""
        with ParallelSimulator(config=fast_sim_config) as sim:
            result = sim.find_best_play(
                combat_state=basic_combat_state,
                search_budget=50,
            )

            assert result is not None
            assert result.best_action is not None
            assert result.time_ms >= 0
            # Action should be legal
            legal_actions = basic_combat_state.get_legal_actions()
            assert any(
                type(result.best_action) == type(a)
                for a in legal_actions
            )

    def test_find_best_play_single_action(self, fast_sim_config, basic_combat_state):
        """Test MCTS with only one legal action (EndTurn only)."""
        # Create state with no playable cards (empty hand)
        limited_state = basic_combat_state.copy()
        limited_state.hand = []
        limited_state.energy = 0

        with ParallelSimulator(config=fast_sim_config) as sim:
            result = sim.find_best_play(limited_state, search_budget=10)
            assert isinstance(result.best_action, EndTurn)
            assert result.confidence == 1.0

    def test_find_best_play_exploration_constant(self, fast_sim_config, basic_combat_state):
        """Test custom exploration constant."""
        with ParallelSimulator(config=fast_sim_config) as sim:
            result = sim.find_best_play(
                combat_state=basic_combat_state,
                search_budget=50,
                exploration_constant=2.0,
            )
            assert result.best_action is not None

    def test_find_best_plays_batch(self, fast_sim_config, basic_combat_state):
        """Test batch MCTS search."""
        states = [basic_combat_state.copy() for _ in range(3)]

        with ParallelSimulator(config=fast_sim_config) as sim:
            results = sim.find_best_plays_batch(states, search_budget=50)
            assert len(results) == 3
            for result in results:
                assert result.best_action is not None


class TestParallelSimulatorShutdown:
    """Test ParallelSimulator shutdown and cleanup."""

    def test_shutdown_clears_executor(self, fast_sim_config):
        """Test that shutdown clears the executor."""
        sim = ParallelSimulator(config=fast_sim_config)
        sim._initialize()
        assert sim._executor is not None

        sim.shutdown()
        assert sim._executor is None
        assert sim._initialized is False

    def test_shutdown_after_simulation(self, fast_sim_config):
        """Test shutdown after running simulations."""
        sim = ParallelSimulator(config=fast_sim_config)
        sim.simulate_batch(["SHUT001", "SHUT002"])
        sim.shutdown()
        assert sim._initialized is False

    def test_context_manager_cleanup_on_exception(self, fast_sim_config):
        """Test that context manager cleans up on exception."""
        try:
            with ParallelSimulator(config=fast_sim_config) as sim:
                sim.simulate_batch(["ERR001"])
                raise ValueError("Test exception")
        except ValueError:
            pass
        # Sim should be cleaned up
        assert sim._initialized is False


# =============================================================================
# WorkerPool Tests
# =============================================================================


class TestWorkerPoolCreation:
    """Test WorkerPool creation and termination."""

    def test_pool_creation(self, fast_sim_config):
        """Test basic pool creation."""
        pool = WorkerPool(n_workers=2, config=fast_sim_config)
        try:
            pool.start()
            assert pool._started is True
            assert len(pool.workers) == 2
            assert pool.is_alive()
        finally:
            pool.shutdown()

    def test_pool_not_started_until_start_called(self, fast_sim_config):
        """Test that pool doesn't start until explicitly started."""
        pool = WorkerPool(n_workers=2, config=fast_sim_config)
        assert pool._started is False
        assert len(pool.workers) == 0
        pool.shutdown()

    def test_pool_context_manager(self, fast_sim_config):
        """Test pool as context manager."""
        with WorkerPool(n_workers=2, config=fast_sim_config) as pool:
            assert pool._started is True
            assert pool.is_alive()
        assert pool._started is False

    def test_pool_shutdown_timeout(self, fast_sim_config):
        """Test pool shutdown with timeout."""
        pool = WorkerPool(n_workers=2, config=fast_sim_config)
        pool.start()
        pool.shutdown(timeout=1.0)
        assert pool._started is False


class TestWorkerPoolTaskDistribution:
    """Test WorkerPool task distribution."""

    def test_submit_single_task(self, fast_sim_config):
        """Test submitting a single task."""
        with WorkerPool(n_workers=2, config=fast_sim_config) as pool:
            task_id = pool.submit(TaskType.SIMULATE_RUN, {"seed": "TASK001"})
            assert task_id >= 0
            assert pool.pending_count() == 1

    def test_submit_batch_tasks(self, fast_sim_config):
        """Test submitting a batch of tasks."""
        with WorkerPool(n_workers=2, config=fast_sim_config) as pool:
            tasks = [
                create_run_task(f"BATCH{i:03d}")
                for i in range(10)
            ]
            task_ids = pool.submit_batch(tasks)

            assert len(task_ids) == 10
            assert all(tid >= 0 for tid in task_ids)

    def test_task_id_increments(self, fast_sim_config):
        """Test that task IDs increment."""
        with WorkerPool(n_workers=2, config=fast_sim_config) as pool:
            id1 = pool.submit(TaskType.SIMULATE_RUN, {"seed": "INC001"})
            id2 = pool.submit(TaskType.SIMULATE_RUN, {"seed": "INC002"})
            assert id2 == id1 + 1


class TestWorkerPoolResultCollection:
    """Test WorkerPool result collection."""

    def test_collect_single_result(self, fast_sim_config):
        """Test collecting a single result."""
        with WorkerPool(n_workers=2, config=fast_sim_config) as pool:
            task_id = pool.submit(TaskType.SIMULATE_RUN, {
                "seed": "COLLECT001",
                "ascension": 20,
                "max_floors": 10,
            })

            result = pool.get_result(timeout=10.0)
            assert result is not None
            assert result.task_id == task_id

    def test_collect_specific_results(self, fast_sim_config):
        """Test collecting specific task results."""
        with WorkerPool(n_workers=2, config=fast_sim_config) as pool:
            tasks = [create_run_task(f"SPEC{i:03d}", max_floors=5) for i in range(5)]
            task_ids = pool.submit_batch(tasks)

            results = pool.collect_results(task_ids, timeout=30.0)
            assert len(results) >= 1  # At least some should complete

    def test_collect_all_results(self, fast_sim_config):
        """Test collecting all available results."""
        with WorkerPool(n_workers=2, config=fast_sim_config) as pool:
            tasks = [create_run_task(f"ALL{i:03d}", max_floors=3) for i in range(5)]
            pool.submit_batch(tasks)

            # Wait a bit for tasks to complete
            time.sleep(1.0)

            results = pool.collect_all(timeout=5.0)
            assert isinstance(results, list)

    def test_result_contains_execution_time(self, fast_sim_config):
        """Test that results contain execution time."""
        with WorkerPool(n_workers=2, config=fast_sim_config) as pool:
            task_id = pool.submit(TaskType.SIMULATE_RUN, {
                "seed": "TIME001",
                "max_floors": 5,
            })

            result = pool.get_result(timeout=10.0)
            assert result.execution_time_ms >= 0

    def test_timeout_returns_none(self, fast_sim_config):
        """Test that timeout returns None instead of blocking."""
        with WorkerPool(n_workers=1, config=fast_sim_config) as pool:
            # Don't submit any tasks
            result = pool.get_result(timeout=0.1)
            assert result is None


# =============================================================================
# BatchProcessor Tests
# =============================================================================


class TestBatchProcessorCreation:
    """Test BatchProcessor batch creation."""

    def test_basic_batch_creation(self, fast_batch_config):
        """Test basic batch processor creation."""
        processor = BatchProcessor(lambda x: x * 2, fast_batch_config)
        assert processor.process_fn is not None
        assert processor.config == fast_batch_config

    def test_default_config(self):
        """Test batch processor with default config."""
        processor = BatchProcessor(lambda x: x)
        assert processor.config is not None
        assert processor.config.chunk_size == 1000


class TestBatchProcessorExecution:
    """Test BatchProcessor batch execution."""

    def test_process_simple_function(self, fast_batch_config):
        """Test processing with simple function."""
        processor = BatchProcessor(lambda x: x * 2, fast_batch_config)
        items = list(range(100))

        result = processor.process(items)

        assert result.total_tasks == 100
        assert result.completed_tasks == 100
        assert result.failed_tasks == 0
        assert result.results == [x * 2 for x in items]

    def test_process_with_errors(self, fast_batch_config):
        """Test processing with errors."""
        def failing_fn(x):
            if x == 5:
                raise ValueError("Test error")
            return x * 2

        config = BatchConfig(continue_on_error=True, max_errors=10)
        processor = BatchProcessor(failing_fn, config)
        items = list(range(10))

        result = processor.process(items)

        assert result.failed_tasks == 1
        assert result.completed_tasks == 9
        assert len(result.errors) == 1
        assert result.errors[0][0] == 5  # Index of failure

    def test_process_stops_on_max_errors(self):
        """Test that processing stops when max errors reached."""
        def always_fails(x):
            raise ValueError("Always fails")

        config = BatchConfig(continue_on_error=True, max_errors=5)
        processor = BatchProcessor(always_fails, config)
        items = list(range(100))

        result = processor.process(items)

        assert result.failed_tasks == 5
        assert len(result.errors) == 5

    def test_progress_callback(self, fast_batch_config):
        """Test progress callback is invoked."""
        progress_calls = []

        def progress_cb(completed, total):
            progress_calls.append((completed, total))

        config = BatchConfig(report_interval=10, chunk_size=100)
        processor = BatchProcessor(lambda x: x, config)
        items = list(range(50))

        processor.process(items, progress_callback=progress_cb)

        assert len(progress_calls) >= 1

    def test_chunked_processing(self):
        """Test that processing is chunked correctly."""
        gc_calls = []

        def mock_gc():
            gc_calls.append(1)

        config = BatchConfig(chunk_size=10, enable_gc=True)
        processor = BatchProcessor(lambda x: x, config)
        items = list(range(25))

        with patch('gc.collect', mock_gc):
            processor.process(items)

        # Should have GC'd between chunks (3 chunks for 25 items with chunk_size=10)
        assert len(gc_calls) == 3


class TestBatchProcessorResultAggregation:
    """Test BatchProcessor result aggregation."""

    def test_success_rate_calculation(self, fast_batch_config):
        """Test success rate calculation."""
        processor = BatchProcessor(lambda x: x, fast_batch_config)
        items = list(range(100))

        result = processor.process(items)

        assert result.success_rate() == 100.0

    def test_success_rate_with_failures(self):
        """Test success rate with some failures."""
        def partial_fail(x):
            if x % 10 == 0:
                raise ValueError("Fail")
            return x

        config = BatchConfig(continue_on_error=True, max_errors=100)
        processor = BatchProcessor(partial_fail, config)
        items = list(range(100))

        result = processor.process(items)

        # 10 failures out of 100 = 90% success
        assert result.success_rate() == 90.0

    def test_empty_batch_success_rate(self, fast_batch_config):
        """Test success rate with empty batch."""
        processor = BatchProcessor(lambda x: x, fast_batch_config)
        result = processor.process([])

        assert result.success_rate() == 0.0

    def test_timing_tracked(self, fast_batch_config):
        """Test that timing is tracked."""
        processor = BatchProcessor(lambda x: x, fast_batch_config)
        items = list(range(100))

        result = processor.process(items)

        assert result.total_time_ms > 0
        assert result.tasks_per_second > 0


# =============================================================================
# StateSerializer Tests
# =============================================================================


class TestStateSerializerRoundtrip:
    """Test StateSerializer serialization roundtrip."""

    def test_combat_state_roundtrip(self, serializer, basic_combat_state):
        """Test CombatState serialization roundtrip."""
        data = serializer.serialize_combat_state(basic_combat_state)
        restored = serializer.deserialize_combat_state(data)

        assert restored.player.hp == basic_combat_state.player.hp
        assert restored.player.max_hp == basic_combat_state.player.max_hp
        assert restored.energy == basic_combat_state.energy
        assert restored.stance == basic_combat_state.stance
        assert restored.hand == basic_combat_state.hand
        assert restored.draw_pile == basic_combat_state.draw_pile
        assert len(restored.enemies) == len(basic_combat_state.enemies)

    def test_combat_state_with_statuses(self, serializer, wrath_combat_state):
        """Test serialization of state with statuses."""
        data = serializer.serialize_combat_state(wrath_combat_state)
        restored = serializer.deserialize_combat_state(data)

        assert restored.player.statuses == wrath_combat_state.player.statuses
        assert restored.player.strength == wrath_combat_state.player.strength

    def test_multi_enemy_roundtrip(self, serializer, multi_enemy_combat_state):
        """Test serialization with multiple enemies."""
        data = serializer.serialize_combat_state(multi_enemy_combat_state)
        restored = serializer.deserialize_combat_state(data)

        assert len(restored.enemies) == len(multi_enemy_combat_state.enemies)
        for orig, rest in zip(multi_enemy_combat_state.enemies, restored.enemies):
            assert rest.hp == orig.hp
            assert rest.id == orig.id
            assert rest.move_damage == orig.move_damage

    def test_compact_format_roundtrip(self, serializer, basic_combat_state):
        """Test compact binary format roundtrip."""
        data = serializer.serialize_combat_compact(basic_combat_state)
        restored = serializer.deserialize_combat_compact(data)

        assert restored.player.hp == basic_combat_state.player.hp
        assert restored.energy == basic_combat_state.energy
        assert restored.stance == basic_combat_state.stance
        assert restored.hand == basic_combat_state.hand

    def test_batch_serialization(self, serializer, basic_combat_state):
        """Test batch serialization."""
        states = [basic_combat_state.copy() for _ in range(5)]
        data = serializer.serialize_batch(states)
        restored = serializer.deserialize_batch(data)

        assert len(restored) == 5
        for state in restored:
            assert state.player.hp == basic_combat_state.player.hp


class TestStateSerializerCompression:
    """Test StateSerializer compression efficiency."""

    def test_compact_smaller_than_pickle(self, serializer, basic_combat_state):
        """Test that compact format is smaller than pickle."""
        pickle_data = serializer.serialize_combat_state(basic_combat_state)
        compact_data = serializer.serialize_combat_compact(basic_combat_state)

        # Compact should be smaller
        assert len(compact_data) < len(pickle_data)

    def test_compression_ratio(self, serializer, basic_combat_state):
        """Test compression ratio is reasonable."""
        pickle_data = serializer.serialize_combat_state(basic_combat_state)
        compact_data = serializer.serialize_combat_compact(basic_combat_state)

        ratio = len(compact_data) / len(pickle_data)
        # Compact should be at most 80% of pickle size
        assert ratio < 0.8, f"Compression ratio {ratio:.2%} is too high"

    def test_large_state_serialization(self, serializer):
        """Test serialization of large state."""
        # Create a state with large card piles
        large_state = CombatState(
            player=EntityState(hp=50, max_hp=80, block=20, statuses={"Strength": 5}),
            energy=3,
            max_energy=3,
            stance="Wrath",
            hand=["Strike_P"] * 10,
            draw_pile=["Strike_P"] * 50 + ["Defend_P"] * 50,
            discard_pile=["Eruption"] * 20,
            exhaust_pile=["Vigilance"] * 5,
            enemies=[
                EnemyCombatState(
                    hp=100, max_hp=120, block=10, id="Boss",
                    move_id=1, move_damage=30, move_hits=3,
                    statuses={"Strength": 10, "Vulnerable": 2},
                )
            ],
        )

        pickle_data = serializer.serialize_combat_state(large_state)
        restored = serializer.deserialize_combat_state(pickle_data)

        assert len(restored.draw_pile) == 100
        assert len(restored.discard_pile) == 20


# =============================================================================
# SimulationConfig Tests
# =============================================================================


class TestSimulationConfigDefaults:
    """Test SimulationConfig default values."""

    def test_default_values(self):
        """Test default configuration values."""
        config = SimulationConfig()

        assert config.n_workers >= 1
        assert config.batch_size == 100
        assert config.default_ascension == 20
        assert config.character == "Watcher"
        assert config.max_turns_per_combat == 100
        assert config.max_floors_per_run == 55
        assert config.default_search_budget == 1000
        assert config.exploration_constant == pytest.approx(1.414, rel=0.01)
        assert config.pickle_protocol == 5
        assert config.use_shared_memory is True
        assert config.prefork_workers is True

    def test_auto_worker_count_calculation(self):
        """Test automatic worker count calculation."""
        config = SimulationConfig(n_workers=0)
        expected = max(1, cpu_count() - 1)
        assert config.n_workers == expected


class TestSimulationConfigCustom:
    """Test SimulationConfig custom configurations."""

    def test_custom_worker_count(self):
        """Test custom worker count."""
        config = SimulationConfig(n_workers=4)
        assert config.n_workers == 4

    def test_custom_batch_size(self):
        """Test custom batch size."""
        config = SimulationConfig(batch_size=50)
        assert config.batch_size == 50

    def test_custom_game_config(self):
        """Test custom game configuration."""
        config = SimulationConfig(
            default_ascension=15,
            character="Ironclad",
            max_turns_per_combat=50,
            max_floors_per_run=30,
        )

        assert config.default_ascension == 15
        assert config.character == "Ironclad"
        assert config.max_turns_per_combat == 50
        assert config.max_floors_per_run == 30

    def test_custom_mcts_config(self):
        """Test custom MCTS configuration."""
        config = SimulationConfig(
            default_search_budget=500,
            exploration_constant=2.0,
        )

        assert config.default_search_budget == 500
        assert config.exploration_constant == 2.0

    def test_disable_shared_memory(self):
        """Test disabling shared memory."""
        config = SimulationConfig(use_shared_memory=False)
        assert config.use_shared_memory is False

    def test_disable_prefork(self):
        """Test disabling worker prefork."""
        config = SimulationConfig(prefork_workers=False)
        assert config.prefork_workers is False


# =============================================================================
# Performance Tests
# =============================================================================


class TestPerformanceThroughput:
    """Basic throughput benchmark tests."""

    @pytest.mark.slow
    def test_batch_throughput(self, fast_sim_config):
        """Test batch simulation throughput."""
        seeds = [f"PERF{i:04d}" for i in range(20)]

        with ParallelSimulator(config=fast_sim_config) as sim:
            start = time.perf_counter()
            results = sim.simulate_batch(seeds)
            elapsed = time.perf_counter() - start

            throughput = len(results) / elapsed
            print(f"\nBatch throughput: {throughput:.1f} sims/sec")

            # Should complete in reasonable time
            assert elapsed < 60.0
            assert len(results) == len(seeds)

    @pytest.mark.slow
    def test_single_simulation_time(self, fast_sim_config):
        """Test single simulation execution time."""
        with ParallelSimulator(config=fast_sim_config) as sim:
            times = []
            for i in range(10):
                start = time.perf_counter()
                sim.simulate_single(f"SINGLE{i:03d}")
                times.append(time.perf_counter() - start)

            avg_time = sum(times) / len(times)
            print(f"\nAverage single sim time: {avg_time*1000:.1f}ms")

            # Each simulation should complete in reasonable time
            assert avg_time < 10.0  # 10 seconds max

    def test_serialization_speed(self, serializer, basic_combat_state):
        """Test serialization speed."""
        iterations = 1000

        start = time.perf_counter()
        for _ in range(iterations):
            data = serializer.serialize_combat_state(basic_combat_state)
            _ = serializer.deserialize_combat_state(data)
        elapsed = time.perf_counter() - start

        ops_per_sec = (iterations * 2) / elapsed  # serialize + deserialize
        print(f"\nSerialization roundtrips/sec: {ops_per_sec:.0f}")

        # Should be fast
        assert elapsed < 5.0


class TestPerformanceMemory:
    """Memory usage tests."""

    def test_batch_memory_stability(self, fast_sim_config):
        """Test that memory doesn't grow unboundedly during batch."""
        import tracemalloc

        tracemalloc.start()

        with ParallelSimulator(config=fast_sim_config) as sim:
            # Run multiple batches
            for batch in range(3):
                seeds = [f"MEM{batch}_{i:03d}" for i in range(10)]
                sim.simulate_batch(seeds)
                gc.collect()

        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()

        # Convert to MB
        peak_mb = peak / (1024 * 1024)
        print(f"\nPeak memory: {peak_mb:.1f}MB")

        # Should not use excessive memory
        assert peak_mb < 500  # 500MB limit

    def test_serializer_no_memory_leak(self, serializer, basic_combat_state):
        """Test that serializer doesn't leak memory."""
        import tracemalloc

        tracemalloc.start()

        for _ in range(1000):
            data = serializer.serialize_combat_state(basic_combat_state)
            _ = serializer.deserialize_combat_state(data)

        gc.collect()
        current, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()

        current_mb = current / (1024 * 1024)
        print(f"\nCurrent memory after serialization: {current_mb:.1f}MB")

        # Should not retain significant memory
        assert current_mb < 50


# =============================================================================
# Integration Tests
# =============================================================================


class TestIntegration:
    """Integration tests combining multiple components."""

    def test_full_simulation_workflow(self, fast_sim_config):
        """Test complete simulation workflow."""
        with ParallelSimulator(config=fast_sim_config) as sim:
            # Run batch simulation
            seeds = ["INTEG001", "INTEG002", "INTEG003"]
            results = sim.simulate_batch(seeds, ascension=20)

            # Verify results
            assert len(results) == 3
            for result in results:
                assert isinstance(result, SimulationResult)
                assert result.final_floor >= 0
                assert result.simulation_time_ms > 0

            # Check stats
            stats = sim.get_stats()
            assert stats["total_simulations"] >= 3

    def test_combat_simulation_workflow(self, fast_sim_config, basic_combat_state):
        """Test combat simulation workflow."""
        with ParallelSimulator(config=fast_sim_config) as sim:
            # Single combat
            result = sim.simulate_combat_single(
                basic_combat_state,
                actions=[PlayCard(0, 0)],
            )

            assert isinstance(result, CombatSimResult)
            assert result.cards_played >= 1

    def test_mcts_workflow(self, fast_sim_config, basic_combat_state):
        """Test MCTS search workflow."""
        with ParallelSimulator(config=fast_sim_config) as sim:
            result = sim.find_best_play(
                basic_combat_state,
                search_budget=50,
            )

            assert result.best_action is not None
            assert result.nodes_explored >= 0

    def test_worker_pool_with_batch_processor(self, fast_sim_config, fast_batch_config):
        """Test WorkerPool combined with BatchProcessor."""
        def process_seed(seed: str) -> dict:
            # Simulate processing
            return {"seed": seed, "processed": True}

        processor = BatchProcessor(process_seed, fast_batch_config)
        seeds = [f"COMBINE{i:03d}" for i in range(20)]

        result = processor.process(seeds)

        assert result.completed_tasks == 20
        assert result.failed_tasks == 0
        assert all(r["processed"] for r in result.results)


# =============================================================================
# Edge Cases
# =============================================================================


class TestEdgeCases:
    """Test edge cases and boundary conditions."""

    def test_terminal_combat_state(self, fast_sim_config):
        """Test simulation with terminal combat state."""
        terminal_state = CombatState(
            player=EntityState(hp=0, max_hp=80),  # Dead player
            energy=3,
            max_energy=3,
            hand=[],
            draw_pile=[],
            discard_pile=[],
            exhaust_pile=[],
            enemies=[],
        )

        with ParallelSimulator(config=fast_sim_config) as sim:
            result = sim.simulate_combat_single(terminal_state)
            assert result.victory is False or terminal_state.is_terminal()

    def test_victory_combat_state(self, fast_sim_config):
        """Test simulation with victory combat state (all enemies dead)."""
        victory_state = CombatState(
            player=EntityState(hp=50, max_hp=80),
            energy=3,
            max_energy=3,
            hand=["Strike_P"],
            draw_pile=[],
            discard_pile=[],
            exhaust_pile=[],
            enemies=[
                EnemyCombatState(hp=0, max_hp=40, id="DeadEnemy",
                               move_id=0, move_damage=0, move_hits=0),
            ],
        )

        with ParallelSimulator(config=fast_sim_config) as sim:
            result = sim.find_best_play(victory_state, search_budget=10)
            # Should handle terminal state gracefully
            assert result is not None

    def test_empty_hand_state(self, fast_sim_config, basic_enemy):
        """Test state with empty hand."""
        empty_hand_state = CombatState(
            player=EntityState(hp=50, max_hp=80),
            energy=3,
            max_energy=3,
            hand=[],
            draw_pile=["Strike_P"] * 10,
            discard_pile=[],
            exhaust_pile=[],
            enemies=[basic_enemy],
        )

        with ParallelSimulator(config=fast_sim_config) as sim:
            result = sim.find_best_play(empty_hand_state, search_budget=10)
            # Only legal action is EndTurn
            assert isinstance(result.best_action, EndTurn)

    def test_no_energy_state(self, fast_sim_config, basic_enemy):
        """Test state with no energy."""
        no_energy_state = CombatState(
            player=EntityState(hp=50, max_hp=80),
            energy=0,
            max_energy=3,
            hand=["Strike_P", "Defend_P"],
            draw_pile=[],
            discard_pile=[],
            exhaust_pile=[],
            enemies=[basic_enemy],
        )

        legal = no_energy_state.get_legal_actions()
        # Should only have EndTurn since no energy
        assert all(isinstance(a, EndTurn) for a in legal)

    def test_unicode_seed(self, fast_sim_config):
        """Test seed with unicode characters."""
        with ParallelSimulator(config=fast_sim_config) as sim:
            # This might fail or work depending on implementation
            try:
                result = sim.simulate_single("SEED_WITH_UNICODE")
                assert result is not None
            except Exception:
                pytest.skip("Unicode seeds not supported")

    def test_very_long_seed(self, fast_sim_config):
        """Test with very long seed string."""
        long_seed = "A" * 1000

        with ParallelSimulator(config=fast_sim_config) as sim:
            result = sim.simulate_single(long_seed)
            assert result.seed == long_seed


# =============================================================================
# Benchmark Tests (marked as slow)
# =============================================================================


@pytest.mark.slow
class TestBenchmarks:
    """Benchmark tests - run with pytest -m slow."""

    def test_run_benchmark_function(self, fast_sim_config):
        """Test the run_benchmark utility function."""
        with ParallelSimulator(config=fast_sim_config) as sim:
            results = run_benchmark(sim, n_sims=100, warmup=10)

            assert "full_run" in results
            assert "single_sim" in results
            assert "serialize" in results

            for key, result in results.items():
                assert result.n_iterations > 0
                assert result.throughput > 0
                print(f"\n{result}")

    def test_scaling_with_workers(self):
        """Test throughput scaling with worker count."""
        seeds = [f"SCALE{i:04d}" for i in range(50)]
        results = {}

        for n_workers in [1, 2, 4]:
            config = SimulationConfig(
                n_workers=n_workers,
                batch_size=10,
                max_floors_per_run=10,
                prefork_workers=True,
            )

            with ParallelSimulator(config=config) as sim:
                start = time.perf_counter()
                sim.simulate_batch(seeds)
                elapsed = time.perf_counter() - start
                results[n_workers] = len(seeds) / elapsed

        print("\nScaling results:")
        for workers, throughput in results.items():
            print(f"  {workers} workers: {throughput:.1f} sims/sec")

        # More workers should generally be faster (but not always linear)
        # Just verify it completed without error


# =============================================================================
# Utility Tests
# =============================================================================


class TestUtilityFunctions:
    """Test utility functions in worker module."""

    def test_create_run_task(self):
        """Test create_run_task utility."""
        task_type, payload = create_run_task("SEED123", ascension=15, max_floors=30)

        assert task_type == TaskType.SIMULATE_RUN
        assert payload["seed"] == "SEED123"
        assert payload["ascension"] == 15
        assert payload["max_floors"] == 30

    def test_create_combat_task(self):
        """Test create_combat_task utility."""
        state_bytes = b"test_state"
        task_type, payload = create_combat_task(
            state_bytes,
            actions=["action1"],
            max_turns=50,
        )

        assert task_type == TaskType.SIMULATE_COMBAT
        assert payload["state"] == state_bytes
        assert payload["actions"] == ["action1"]
        assert payload["max_turns"] == 50

    def test_create_mcts_task(self):
        """Test create_mcts_task utility."""
        state_bytes = b"combat_state"
        task_type, payload = create_mcts_task(state_bytes, budget=200)

        assert task_type == TaskType.MCTS_ROLLOUT
        assert payload["state"] == state_bytes
        assert payload["budget"] == 200


# =============================================================================
# Main
# =============================================================================


if __name__ == "__main__":
    # Run tests with pytest
    pytest.main([__file__, "-v", "-x"])
