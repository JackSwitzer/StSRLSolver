"""
Parallel Simulation Engine for Slay the Spire RL.

Provides high-throughput parallel simulation for:
1. MCTS tree search (combat-level simulations)
2. Full run simulations for training
3. Batch evaluation of agents

Target: 10,000+ simulations/second on multi-core systems.

Architecture:
- ProcessPoolExecutor for true parallelism (bypasses GIL)
- Shared memory for common data (card pool, enemy definitions)
- Pre-forked worker pools for minimal startup overhead
- Batch processing to minimize IPC overhead

Usage:
    from core.simulation import ParallelSimulator, SimulationConfig

    # Create simulator with 8 workers
    sim = ParallelSimulator(n_workers=8)

    # Run batch of full game simulations
    results = sim.simulate_batch(seeds=["ABC", "DEF", "GHI"], agent=my_agent)

    # Combat-level simulation for MCTS
    states = sim.simulate_combats(combat_states, actions)

    # Per-hand optimization
    best_action = sim.find_best_play(combat_state, search_budget=1000)

    # Benchmarking
    from core.simulation import run_benchmark
    stats = run_benchmark(sim, n_sims=10000)
"""

from .engine import (
    ParallelSimulator,
    SimulationConfig,
    SimulationResult,
    CombatSimResult,
)
from .worker import (
    WorkerPool,
    SimulationWorker,
    WorkerTask,
    TaskType,
)
from .batch import (
    BatchProcessor,
    BatchConfig,
    BatchResult,
    StateSerializer,
    run_benchmark,
)

__all__ = [
    # Main simulator
    "ParallelSimulator",
    "SimulationConfig",
    "SimulationResult",
    "CombatSimResult",
    # Worker management
    "WorkerPool",
    "SimulationWorker",
    "WorkerTask",
    "TaskType",
    # Batch processing
    "BatchProcessor",
    "BatchConfig",
    "BatchResult",
    "StateSerializer",
    "run_benchmark",
]
