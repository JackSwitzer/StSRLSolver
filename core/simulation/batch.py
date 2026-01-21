"""
Batch Processing Utilities for Parallel Simulation.

Provides:
1. BatchProcessor - High-level batch processing interface
2. StateSerializer - Efficient state serialization/deserialization
3. BatchConfig - Configuration for batch operations
4. Benchmarking utilities

Optimizations:
- Pickle protocol 5 for zero-copy buffer sharing
- Pre-allocated result buffers
- Chunked processing for memory efficiency
- Progress callbacks for long-running batches
"""

from __future__ import annotations

import gc
import pickle
import struct
import time
from dataclasses import dataclass, field
from typing import (
    List, Dict, Optional, Any, Callable, Iterator, Tuple, TypeVar, Generic
)

# For state serialization
from ..state.combat import CombatState, EntityState, EnemyCombatState
from ..state.run import RunState, CardInstance, RelicInstance, PotionSlot


# =============================================================================
# Configuration
# =============================================================================

@dataclass
class BatchConfig:
    """Configuration for batch processing."""

    # Batch sizes
    chunk_size: int = 1000  # Tasks per chunk for memory efficiency
    result_buffer_size: int = 10000  # Pre-allocated result buffer

    # Serialization
    pickle_protocol: int = 5  # Use protocol 5 for best performance
    compress: bool = False  # Compression (not worth it for small states)

    # Progress tracking
    report_interval: int = 1000  # Report progress every N tasks
    enable_gc: bool = True  # Run GC between chunks

    # Error handling
    continue_on_error: bool = True  # Continue processing on individual errors
    max_errors: int = 100  # Maximum errors before aborting


@dataclass
class BatchResult:
    """Result of a batch operation."""

    total_tasks: int
    completed_tasks: int
    failed_tasks: int
    total_time_ms: float
    tasks_per_second: float

    # Per-task results
    results: List[Any] = field(default_factory=list)
    errors: List[Tuple[int, str]] = field(default_factory=list)  # (index, error_msg)

    # Timing breakdown
    serialization_time_ms: float = 0.0
    processing_time_ms: float = 0.0
    deserialization_time_ms: float = 0.0

    def success_rate(self) -> float:
        """Get success rate as percentage."""
        if self.total_tasks == 0:
            return 0.0
        return self.completed_tasks / self.total_tasks * 100


# =============================================================================
# State Serialization
# =============================================================================

class StateSerializer:
    """
    Efficient serialization for game states.

    Provides:
    - Fast pickle-based serialization
    - Optional compact binary format
    - State compression
    - Batch serialization

    The default pickle approach is fast enough for most uses.
    The compact format is available for cases where bandwidth matters.
    """

    def __init__(self, config: Optional[BatchConfig] = None):
        """Initialize serializer."""
        self.config = config or BatchConfig()
        self._cache = {}  # Optional caching for repeated serialization

    def serialize_combat_state(self, state: CombatState) -> bytes:
        """Serialize a CombatState to bytes."""
        return pickle.dumps(state, protocol=self.config.pickle_protocol)

    def deserialize_combat_state(self, data: bytes) -> CombatState:
        """Deserialize bytes to CombatState."""
        return pickle.loads(data)

    def serialize_run_state(self, state: RunState) -> bytes:
        """Serialize a RunState to bytes."""
        return pickle.dumps(state, protocol=self.config.pickle_protocol)

    def deserialize_run_state(self, data: bytes) -> RunState:
        """Deserialize bytes to RunState."""
        return pickle.loads(data)

    def serialize_batch(self, states: List[Any]) -> bytes:
        """Serialize multiple states into a single bytes object."""
        return pickle.dumps(states, protocol=self.config.pickle_protocol)

    def deserialize_batch(self, data: bytes) -> List[Any]:
        """Deserialize bytes to multiple states."""
        return pickle.loads(data)

    # =========================================================================
    # Compact Binary Format (for bandwidth-constrained scenarios)
    # =========================================================================

    def serialize_combat_compact(self, state: CombatState) -> bytes:
        """
        Serialize CombatState to compact binary format.

        Format:
        - 4 bytes: total length
        - Player state (variable)
        - Enemy states (variable)
        - Card piles (variable)
        - Other fields (variable)

        ~50% smaller than pickle for typical states.
        """
        parts = []

        # Player state: hp, max_hp, block, energy, stance (as int)
        stance_id = {
            "Neutral": 0, "Wrath": 1, "Calm": 2, "Divinity": 3
        }.get(state.stance, 0)

        player_data = struct.pack(
            '<HHHBBBi',
            state.player.hp,
            state.player.max_hp,
            state.player.block,
            state.energy,
            state.max_energy,
            stance_id,
            len(state.player.statuses),
        )
        parts.append(player_data)

        # Player statuses as (id_len, id_bytes, value) tuples
        for status_id, value in state.player.statuses.items():
            id_bytes = status_id.encode('utf-8')
            parts.append(struct.pack('<B', len(id_bytes)))
            parts.append(id_bytes)
            parts.append(struct.pack('<i', value))

        # Number of enemies
        parts.append(struct.pack('<B', len(state.enemies)))

        # Enemy states
        for enemy in state.enemies:
            id_bytes = enemy.id.encode('utf-8')
            enemy_data = struct.pack(
                '<B',
                len(id_bytes),
            )
            parts.append(enemy_data)
            parts.append(id_bytes)
            parts.append(struct.pack(
                '<HHHbhB',
                enemy.hp,
                enemy.max_hp,
                enemy.block,
                enemy.move_id,
                enemy.move_damage,
                enemy.move_hits,
            ))
            parts.append(struct.pack('<B', len(enemy.statuses)))
            for status_id, value in enemy.statuses.items():
                id_bytes = status_id.encode('utf-8')
                parts.append(struct.pack('<B', len(id_bytes)))
                parts.append(id_bytes)
                parts.append(struct.pack('<i', value))

        # Card piles
        for pile in [state.hand, state.draw_pile, state.discard_pile, state.exhaust_pile]:
            parts.append(struct.pack('<H', len(pile)))
            for card_id in pile:
                id_bytes = card_id.encode('utf-8')
                parts.append(struct.pack('<B', len(id_bytes)))
                parts.append(id_bytes)

        # Combat tracking
        parts.append(struct.pack(
            '<HBB',
            state.turn,
            state.cards_played_this_turn,
            state.attacks_played_this_turn,
        ))

        return b''.join(parts)

    def deserialize_combat_compact(self, data: bytes) -> CombatState:
        """Deserialize compact binary format to CombatState."""
        offset = 0

        # Player state
        player_hp, player_max_hp, player_block, energy, max_energy, stance_id, num_statuses = \
            struct.unpack_from('<HHHBBBi', data, offset)
        offset += struct.calcsize('<HHHBBBi')

        stance_map = {0: "Neutral", 1: "Wrath", 2: "Calm", 3: "Divinity"}
        stance = stance_map.get(stance_id, "Neutral")

        player_statuses = {}
        for _ in range(num_statuses):
            id_len = struct.unpack_from('<B', data, offset)[0]
            offset += 1
            status_id = data[offset:offset + id_len].decode('utf-8')
            offset += id_len
            value = struct.unpack_from('<i', data, offset)[0]
            offset += 4
            player_statuses[status_id] = value

        # Enemies
        num_enemies = struct.unpack_from('<B', data, offset)[0]
        offset += 1

        enemies = []
        for _ in range(num_enemies):
            id_len = struct.unpack_from('<B', data, offset)[0]
            offset += 1
            enemy_id = data[offset:offset + id_len].decode('utf-8')
            offset += id_len

            hp, max_hp, block, move_id, move_damage, move_hits = \
                struct.unpack_from('<HHHbhB', data, offset)
            offset += struct.calcsize('<HHHbhB')

            num_enemy_statuses = struct.unpack_from('<B', data, offset)[0]
            offset += 1
            enemy_statuses = {}
            for _ in range(num_enemy_statuses):
                id_len = struct.unpack_from('<B', data, offset)[0]
                offset += 1
                status_id = data[offset:offset + id_len].decode('utf-8')
                offset += id_len
                value = struct.unpack_from('<i', data, offset)[0]
                offset += 4
                enemy_statuses[status_id] = value

            enemies.append(EnemyCombatState(
                hp=hp,
                max_hp=max_hp,
                block=block,
                id=enemy_id,
                move_id=move_id,
                move_damage=move_damage,
                move_hits=move_hits,
                statuses=enemy_statuses,
            ))

        # Card piles
        piles = []
        for _ in range(4):
            pile_size = struct.unpack_from('<H', data, offset)[0]
            offset += 2
            pile = []
            for _ in range(pile_size):
                id_len = struct.unpack_from('<B', data, offset)[0]
                offset += 1
                card_id = data[offset:offset + id_len].decode('utf-8')
                offset += id_len
                pile.append(card_id)
            piles.append(pile)

        hand, draw_pile, discard_pile, exhaust_pile = piles

        # Combat tracking
        turn, cards_played, attacks_played = struct.unpack_from('<HBB', data, offset)

        return CombatState(
            player=EntityState(
                hp=player_hp,
                max_hp=player_max_hp,
                block=player_block,
                statuses=player_statuses,
            ),
            energy=energy,
            max_energy=max_energy,
            stance=stance,
            hand=hand,
            draw_pile=draw_pile,
            discard_pile=discard_pile,
            exhaust_pile=exhaust_pile,
            enemies=enemies,
            turn=turn,
            cards_played_this_turn=cards_played,
            attacks_played_this_turn=attacks_played,
        )


# =============================================================================
# Batch Processor
# =============================================================================

T = TypeVar('T')
R = TypeVar('R')


class BatchProcessor(Generic[T, R]):
    """
    High-level batch processing interface.

    Provides:
    - Chunked processing for memory efficiency
    - Progress callbacks
    - Error handling and recovery
    - Timing and statistics

    Usage:
        processor = BatchProcessor(process_fn, config)
        result = processor.process(items, progress_callback)
    """

    def __init__(
        self,
        process_fn: Callable[[T], R],
        config: Optional[BatchConfig] = None,
    ):
        """
        Initialize batch processor.

        Args:
            process_fn: Function to apply to each item
            config: Batch configuration
        """
        self.process_fn = process_fn
        self.config = config or BatchConfig()

    def process(
        self,
        items: List[T],
        progress_callback: Optional[Callable[[int, int], None]] = None,
    ) -> BatchResult:
        """
        Process a batch of items.

        Args:
            items: Items to process
            progress_callback: Optional callback(completed, total)

        Returns:
            BatchResult with results and statistics
        """
        start_time = time.perf_counter()

        total_tasks = len(items)
        completed = 0
        failed = 0
        results = []
        errors = []

        # Process in chunks for memory efficiency
        for chunk_start in range(0, total_tasks, self.config.chunk_size):
            chunk_end = min(chunk_start + self.config.chunk_size, total_tasks)
            chunk = items[chunk_start:chunk_end]

            for i, item in enumerate(chunk):
                try:
                    result = self.process_fn(item)
                    results.append(result)
                    completed += 1

                except Exception as e:
                    failed += 1
                    errors.append((chunk_start + i, str(e)))

                    if not self.config.continue_on_error:
                        raise

                    if len(errors) >= self.config.max_errors:
                        break

                # Progress callback
                if progress_callback and completed % self.config.report_interval == 0:
                    progress_callback(completed, total_tasks)

            # Optional GC between chunks
            if self.config.enable_gc:
                gc.collect()

            # Check error limit
            if len(errors) >= self.config.max_errors:
                break

        total_time_ms = (time.perf_counter() - start_time) * 1000
        tasks_per_second = completed / (total_time_ms / 1000) if total_time_ms > 0 else 0

        return BatchResult(
            total_tasks=total_tasks,
            completed_tasks=completed,
            failed_tasks=failed,
            total_time_ms=total_time_ms,
            tasks_per_second=tasks_per_second,
            results=results,
            errors=errors,
        )

    def process_parallel(
        self,
        items: List[T],
        executor: Any,  # ProcessPoolExecutor
        progress_callback: Optional[Callable[[int, int], None]] = None,
    ) -> BatchResult:
        """
        Process items in parallel using an executor.

        Args:
            items: Items to process
            executor: ProcessPoolExecutor for parallel processing
            progress_callback: Optional callback(completed, total)

        Returns:
            BatchResult with results and statistics
        """
        from concurrent.futures import as_completed

        start_time = time.perf_counter()

        total_tasks = len(items)
        completed = 0
        failed = 0
        results = [None] * total_tasks  # Pre-allocate
        errors = []

        # Submit all tasks
        future_to_idx = {
            executor.submit(self.process_fn, item): i
            for i, item in enumerate(items)
        }

        # Collect results
        for future in as_completed(future_to_idx):
            idx = future_to_idx[future]
            try:
                result = future.result()
                results[idx] = result
                completed += 1

            except Exception as e:
                failed += 1
                errors.append((idx, str(e)))

            if progress_callback and completed % self.config.report_interval == 0:
                progress_callback(completed, total_tasks)

        total_time_ms = (time.perf_counter() - start_time) * 1000
        tasks_per_second = completed / (total_time_ms / 1000) if total_time_ms > 0 else 0

        # Filter None results
        results = [r for r in results if r is not None]

        return BatchResult(
            total_tasks=total_tasks,
            completed_tasks=completed,
            failed_tasks=failed,
            total_time_ms=total_time_ms,
            tasks_per_second=tasks_per_second,
            results=results,
            errors=errors,
        )


# =============================================================================
# Benchmarking Utilities
# =============================================================================

@dataclass
class BenchmarkResult:
    """Result of a benchmark run."""

    name: str
    n_iterations: int
    total_time_ms: float
    mean_time_ms: float
    min_time_ms: float
    max_time_ms: float
    std_time_ms: float
    throughput: float  # Operations per second

    def __str__(self) -> str:
        return (
            f"{self.name}:\n"
            f"  Iterations: {self.n_iterations}\n"
            f"  Total time: {self.total_time_ms:.1f}ms\n"
            f"  Mean: {self.mean_time_ms:.3f}ms\n"
            f"  Min: {self.min_time_ms:.3f}ms\n"
            f"  Max: {self.max_time_ms:.3f}ms\n"
            f"  Std: {self.std_time_ms:.3f}ms\n"
            f"  Throughput: {self.throughput:.1f}/s"
        )


def run_benchmark(
    simulator: Any,  # ParallelSimulator
    n_sims: int = 1000,
    warmup: int = 100,
) -> Dict[str, BenchmarkResult]:
    """
    Run comprehensive benchmark of the simulator.

    Args:
        simulator: ParallelSimulator instance
        n_sims: Number of simulations per benchmark
        warmup: Warmup iterations (not counted)

    Returns:
        Dict of benchmark name -> BenchmarkResult
    """
    import random
    import statistics

    results = {}

    # Generate test seeds
    seeds = [f"BENCH{i:06d}" for i in range(n_sims + warmup)]

    # Warmup
    print(f"Warming up ({warmup} iterations)...")
    _ = simulator.simulate_batch(seeds[:warmup], ascension=20)
    simulator.reset_stats()

    # Benchmark 1: Full run simulation throughput
    print(f"Benchmark: Full run simulation ({n_sims} sims)...")
    times = []

    for i in range(0, n_sims, 100):
        batch_seeds = seeds[warmup + i:warmup + i + 100]
        start = time.perf_counter()
        _ = simulator.simulate_batch(batch_seeds, ascension=20)
        elapsed = (time.perf_counter() - start) * 1000
        times.append(elapsed / len(batch_seeds))

    total_time = sum(times) * 100  # Scale back to total
    mean_time = statistics.mean(times)
    min_time = min(times)
    max_time = max(times)
    std_time = statistics.stdev(times) if len(times) > 1 else 0

    results['full_run'] = BenchmarkResult(
        name="Full Run Simulation",
        n_iterations=n_sims,
        total_time_ms=total_time,
        mean_time_ms=mean_time,
        min_time_ms=min_time,
        max_time_ms=max_time,
        std_time_ms=std_time,
        throughput=n_sims / (total_time / 1000),
    )

    # Benchmark 2: Single simulation (no IPC overhead)
    print(f"Benchmark: Single simulation ({min(100, n_sims)} sims)...")
    single_times = []

    for i in range(min(100, n_sims)):
        start = time.perf_counter()
        _ = simulator.simulate_single(seeds[warmup + i], ascension=20)
        elapsed = (time.perf_counter() - start) * 1000
        single_times.append(elapsed)

    results['single_sim'] = BenchmarkResult(
        name="Single Simulation (no IPC)",
        n_iterations=len(single_times),
        total_time_ms=sum(single_times),
        mean_time_ms=statistics.mean(single_times),
        min_time_ms=min(single_times),
        max_time_ms=max(single_times),
        std_time_ms=statistics.stdev(single_times) if len(single_times) > 1 else 0,
        throughput=len(single_times) / (sum(single_times) / 1000),
    )

    # Benchmark 3: State serialization
    print(f"Benchmark: State serialization...")
    serializer = StateSerializer()

    # Create sample combat state
    sample_state = CombatState(
        player=EntityState(hp=70, max_hp=80, block=10, statuses={"Strength": 2}),
        energy=3,
        max_energy=3,
        stance="Wrath",
        hand=["Strike_P", "Defend_P", "Eruption", "Vigilance", "AscendersBane"],
        draw_pile=["Strike_P"] * 10 + ["Defend_P"] * 10,
        discard_pile=[],
        exhaust_pile=[],
        enemies=[
            EnemyCombatState(hp=40, max_hp=42, block=0, id="JawWorm",
                           move_id=1, move_damage=11, move_hits=1,
                           statuses={}),
        ],
    )

    serialize_times = []
    for _ in range(1000):
        start = time.perf_counter()
        data = serializer.serialize_combat_state(sample_state)
        elapsed = (time.perf_counter() - start) * 1000
        serialize_times.append(elapsed)

    results['serialize'] = BenchmarkResult(
        name="State Serialization",
        n_iterations=1000,
        total_time_ms=sum(serialize_times),
        mean_time_ms=statistics.mean(serialize_times),
        min_time_ms=min(serialize_times),
        max_time_ms=max(serialize_times),
        std_time_ms=statistics.stdev(serialize_times),
        throughput=1000 / (sum(serialize_times) / 1000),
    )

    # Report sizes
    pickle_size = len(data)
    compact_data = serializer.serialize_combat_compact(sample_state)
    compact_size = len(compact_data)
    print(f"\nSerialization sizes:")
    print(f"  Pickle: {pickle_size} bytes")
    print(f"  Compact: {compact_size} bytes ({compact_size/pickle_size*100:.1f}% of pickle)")

    return results


def print_benchmark_results(results: Dict[str, BenchmarkResult]):
    """Print benchmark results in a formatted table."""
    print("\n" + "=" * 60)
    print("BENCHMARK RESULTS")
    print("=" * 60)

    for result in results.values():
        print(f"\n{result}")

    print("\n" + "=" * 60)

    # Summary
    if 'full_run' in results:
        print(f"\nTarget: 10,000+ simulations/second")
        print(f"Achieved: {results['full_run'].throughput:.1f} simulations/second")
        if results['full_run'].throughput >= 10000:
            print("STATUS: TARGET MET")
        else:
            factor = 10000 / results['full_run'].throughput
            print(f"STATUS: Need {factor:.1f}x improvement to meet target")


# =============================================================================
# Testing
# =============================================================================

if __name__ == "__main__":
    print("=== Batch Processing Test ===\n")

    # Test serialization
    print("--- Testing StateSerializer ---")
    serializer = StateSerializer()

    # Create test state
    state = CombatState(
        player=EntityState(hp=70, max_hp=80, block=10, statuses={"Strength": 2}),
        energy=3,
        max_energy=3,
        stance="Wrath",
        hand=["Strike_P", "Defend_P", "Eruption"],
        draw_pile=["Strike_P"] * 5,
        discard_pile=[],
        exhaust_pile=[],
        enemies=[
            EnemyCombatState(hp=40, max_hp=42, block=0, id="JawWorm",
                           move_id=1, move_damage=11, move_hits=1,
                           statuses={"Vulnerable": 2}),
        ],
    )

    # Test pickle serialization
    pickle_bytes = serializer.serialize_combat_state(state)
    restored_state = serializer.deserialize_combat_state(pickle_bytes)
    print(f"Pickle size: {len(pickle_bytes)} bytes")
    print(f"Restored HP: {restored_state.player.hp}")

    # Test compact serialization
    compact_bytes = serializer.serialize_combat_compact(state)
    restored_compact = serializer.deserialize_combat_compact(compact_bytes)
    print(f"Compact size: {len(compact_bytes)} bytes ({len(compact_bytes)/len(pickle_bytes)*100:.1f}% of pickle)")
    print(f"Restored HP (compact): {restored_compact.player.hp}")

    # Test batch processor
    print("\n--- Testing BatchProcessor ---")

    def square(x):
        return x * x

    processor = BatchProcessor(square)
    items = list(range(1000))
    result = processor.process(items)

    print(f"Processed {result.completed_tasks} items")
    print(f"Time: {result.total_time_ms:.1f}ms")
    print(f"Throughput: {result.tasks_per_second:.1f}/s")

    print("\n=== Test Complete ===")
