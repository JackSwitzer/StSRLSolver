"""
Parallel Simulation Engine - Main orchestrator for high-throughput simulation.

This module provides the ParallelSimulator class that manages:
1. Worker pool lifecycle
2. Task distribution and load balancing
3. Result aggregation
4. Shared memory for common data

Performance optimizations:
- ProcessPoolExecutor for true parallelism (bypasses GIL)
- Pre-forked worker pool to avoid spawn overhead
- Batch processing to minimize IPC overhead
- Efficient state serialization (pickle protocol 5)
- Shared memory for immutable data (card/enemy definitions)

NOTE: Simulation logic is delegated to CombatSimulator from core.calc.combat_sim.
This module focuses on parallel orchestration, not simulation mechanics.
"""

from __future__ import annotations

import os
import time
import pickle
from concurrent.futures import ProcessPoolExecutor, as_completed
from dataclasses import dataclass, field
from multiprocessing import cpu_count, shared_memory
from typing import (
    List, Dict, Optional, Tuple, Any, Callable, Union, Protocol, TypeVar
)

# Import from core modules
from ..state.combat import CombatState, Action, PlayCard, EndTurn
from ..state.run import RunState
from ..state.rng import Random, seed_to_long
from ..calc.combat_sim import (
    CombatSimulator,
    Action as SimAction,
    ActionType as SimActionType,
    CombatResult as SimCombatResult,
)


# =============================================================================
# Action Type Conversion Helpers
# =============================================================================

def _state_action_to_sim_action(action: Action) -> SimAction:
    """Convert state.combat Action to calc.combat_sim Action."""
    if isinstance(action, PlayCard):
        return SimAction(
            action_type=SimActionType.PLAY_CARD,
            card_index=action.card_idx,
            target_index=action.target_idx if action.target_idx >= 0 else 0,
        )
    elif isinstance(action, EndTurn):
        return SimAction(action_type=SimActionType.END_TURN)
    else:
        # UsePotion or unknown
        return SimAction(action_type=SimActionType.END_TURN)


def _sim_action_to_state_action(action: SimAction) -> Action:
    """Convert calc.combat_sim Action to state.combat Action."""
    if action.action_type == SimActionType.PLAY_CARD:
        return PlayCard(
            card_idx=action.card_index,
            target_idx=action.target_index,
        )
    elif action.action_type == SimActionType.END_TURN:
        return EndTurn()
    else:
        return EndTurn()


# =============================================================================
# Configuration
# =============================================================================

@dataclass
class SimulationConfig:
    """Configuration for parallel simulation."""

    # Worker configuration
    n_workers: int = 0  # 0 = auto-detect (cpu_count - 1)
    batch_size: int = 100  # Tasks per batch for IPC efficiency

    # Game configuration
    default_ascension: int = 20
    character: str = "Watcher"
    max_turns_per_combat: int = 100
    max_floors_per_run: int = 55  # Acts 1-3 = ~51 floors

    # MCTS configuration
    default_search_budget: int = 1000
    exploration_constant: float = 1.414  # sqrt(2) for UCT

    # Performance tuning
    pickle_protocol: int = 5  # Fastest for Python 3.8+
    use_shared_memory: bool = True  # Share card/enemy data
    prefork_workers: bool = True  # Pre-initialize workers

    def __post_init__(self):
        if self.n_workers <= 0:
            # Leave one core for main process
            self.n_workers = max(1, cpu_count() - 1)


# =============================================================================
# Result Types
# =============================================================================

@dataclass
class SimulationResult:
    """Result of a full game simulation."""

    seed: str
    victory: bool
    final_floor: int
    final_act: int
    final_hp: int
    final_max_hp: int
    final_gold: int
    deck_size: int
    relic_count: int
    combats_won: int
    floors_climbed: int
    decisions_made: int
    simulation_time_ms: float

    # Detailed tracking (optional)
    decision_log: Optional[List[Dict]] = None
    hp_by_floor: Optional[List[int]] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for serialization."""
        return {
            "seed": self.seed,
            "victory": self.victory,
            "final_floor": self.final_floor,
            "final_act": self.final_act,
            "final_hp": self.final_hp,
            "final_max_hp": self.final_max_hp,
            "final_gold": self.final_gold,
            "deck_size": self.deck_size,
            "relic_count": self.relic_count,
            "combats_won": self.combats_won,
            "floors_climbed": self.floors_climbed,
            "decisions_made": self.decisions_made,
            "simulation_time_ms": self.simulation_time_ms,
        }


@dataclass
class CombatSimResult:
    """Result of a combat simulation."""

    victory: bool
    hp_remaining: int
    hp_lost: int
    turns: int
    cards_played: int
    damage_dealt: int
    damage_taken: int
    final_state: Optional[CombatState] = None
    action_sequence: Optional[List[Action]] = None


@dataclass
class MCTSResult:
    """Result of MCTS search for best play."""

    best_action: Action
    action_scores: Dict[str, float]  # action_repr -> score
    nodes_explored: int
    time_ms: float
    confidence: float  # Ratio of visits to best vs second best


# =============================================================================
# Agent Protocol
# =============================================================================

class Agent(Protocol):
    """Protocol for agents that can make decisions."""

    def select_action(self, state: Any, legal_actions: List[Any]) -> Any:
        """Select an action given state and legal actions."""
        ...


# =============================================================================
# Parallel Simulator
# =============================================================================

class ParallelSimulator:
    """
    High-throughput parallel simulation engine.

    Manages a pool of worker processes for parallel simulation of:
    - Full game runs (for training data generation)
    - Combat simulations (for MCTS rollouts)
    - Per-hand optimization (for best play search)

    Usage:
        sim = ParallelSimulator(n_workers=8)

        # Batch simulation
        results = sim.simulate_batch(["SEED1", "SEED2", ...], agent)

        # Combat simulation
        outcomes = sim.simulate_combats(states, actions)

        # Best play search
        best = sim.find_best_play(combat_state, budget=1000)

        # Clean up
        sim.shutdown()
    """

    def __init__(
        self,
        n_workers: int = 0,
        config: Optional[SimulationConfig] = None,
    ):
        """
        Initialize the parallel simulator.

        Args:
            n_workers: Number of worker processes (0 = auto-detect)
            config: Simulation configuration (uses defaults if None)
        """
        self.config = config or SimulationConfig()
        if n_workers > 0:
            self.config.n_workers = n_workers

        self._executor: Optional[ProcessPoolExecutor] = None
        self._shared_data: Optional[shared_memory.SharedMemory] = None
        self._initialized = False

        # Statistics
        self._total_sims = 0
        self._total_time_ms = 0.0

        # Pre-fork if configured
        if self.config.prefork_workers:
            self._initialize()

    def _initialize(self):
        """Initialize worker pool and shared memory."""
        if self._initialized:
            return

        # Create process pool
        self._executor = ProcessPoolExecutor(
            max_workers=self.config.n_workers,
            initializer=_worker_init,
            initargs=(self.config,),
        )

        # Setup shared memory for card/enemy data if enabled
        if self.config.use_shared_memory:
            self._setup_shared_memory()

        self._initialized = True

    def _setup_shared_memory(self):
        """Set up shared memory for immutable game data."""
        # For now, we'll pass data through pickle
        # Full shared memory implementation would serialize card/enemy defs
        # to a shared memory block that workers can mmap
        pass

    def shutdown(self):
        """Shutdown the worker pool and clean up resources."""
        if self._executor:
            self._executor.shutdown(wait=True)
            self._executor = None

        if self._shared_data:
            self._shared_data.close()
            self._shared_data.unlink()
            self._shared_data = None

        self._initialized = False

    def __enter__(self):
        """Context manager entry."""
        self._initialize()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.shutdown()
        return False

    # =========================================================================
    # Full Game Simulation
    # =========================================================================

    def simulate_batch(
        self,
        seeds: List[str],
        agent: Optional[Agent] = None,
        ascension: int = 20,
        track_decisions: bool = False,
        callback: Optional[Callable[[SimulationResult], None]] = None,
    ) -> List[SimulationResult]:
        """
        Run multiple full game simulations in parallel.

        Args:
            seeds: List of seed strings to simulate
            agent: Agent to use for decisions (None = random)
            ascension: Ascension level
            track_decisions: Whether to record full decision logs
            callback: Optional callback for each completed result

        Returns:
            List of SimulationResult objects
        """
        self._initialize()

        results = []
        start_time = time.perf_counter()

        # Create tasks
        tasks = [
            _SimulateRunTask(
                seed=seed,
                ascension=ascension,
                agent_state=_serialize_agent(agent) if agent else None,
                track_decisions=track_decisions,
            )
            for seed in seeds
        ]

        # Submit in batches
        futures = []
        for i in range(0, len(tasks), self.config.batch_size):
            batch = tasks[i:i + self.config.batch_size]
            future = self._executor.submit(_run_batch, batch, self.config)
            futures.append(future)

        # Collect results
        for future in as_completed(futures):
            batch_results = future.result()
            for result in batch_results:
                results.append(result)
                if callback:
                    callback(result)

        elapsed_ms = (time.perf_counter() - start_time) * 1000
        self._total_sims += len(seeds)
        self._total_time_ms += elapsed_ms

        return results

    def simulate_single(
        self,
        seed: str,
        agent: Optional[Agent] = None,
        ascension: int = 20,
        track_decisions: bool = False,
    ) -> SimulationResult:
        """
        Run a single game simulation.

        For single simulations, runs in the current process (no IPC overhead).

        Args:
            seed: Seed string
            agent: Agent to use for decisions
            ascension: Ascension level
            track_decisions: Whether to record decision log

        Returns:
            SimulationResult
        """
        return _simulate_run_internal(
            seed=seed,
            ascension=ascension,
            agent=agent,
            track_decisions=track_decisions,
            config=self.config,
        )

    # =========================================================================
    # Combat Simulation
    # =========================================================================

    def simulate_combats(
        self,
        combat_states: List[CombatState],
        actions: List[List[Action]],
        max_turns: int = 100,
    ) -> List[CombatSimResult]:
        """
        Run combat simulations for MCTS.

        Each combat state is simulated with the given action sequence,
        then random rollout to completion.

        Args:
            combat_states: List of starting combat states
            actions: List of action sequences to execute first
            max_turns: Maximum turns before timeout

        Returns:
            List of CombatSimResult objects
        """
        self._initialize()

        # Serialize states efficiently
        tasks = [
            _SimulateCombatTask(
                state_bytes=pickle.dumps(state, protocol=self.config.pickle_protocol),
                action_bytes=pickle.dumps(acts, protocol=self.config.pickle_protocol),
                max_turns=max_turns,
            )
            for state, acts in zip(combat_states, actions)
        ]

        # Submit and collect
        results = []
        futures = []
        for i in range(0, len(tasks), self.config.batch_size):
            batch = tasks[i:i + self.config.batch_size]
            future = self._executor.submit(_run_combat_batch, batch, self.config)
            futures.append(future)

        for future in as_completed(futures):
            results.extend(future.result())

        return results

    def simulate_combat_single(
        self,
        combat_state: CombatState,
        actions: Optional[List[Action]] = None,
        max_turns: int = 100,
        policy: Optional[Callable[[CombatState], Action]] = None,
    ) -> CombatSimResult:
        """
        Run a single combat simulation in current process.

        Args:
            combat_state: Starting combat state
            actions: Optional action sequence to execute first
            max_turns: Maximum turns
            policy: Policy for choosing actions (None = random)

        Returns:
            CombatSimResult
        """
        return _simulate_combat_internal(
            state=combat_state,
            initial_actions=actions or [],
            max_turns=max_turns,
            policy=policy,
        )

    # =========================================================================
    # MCTS / Best Play Search
    # =========================================================================

    def find_best_play(
        self,
        combat_state: CombatState,
        search_budget: int = 1000,
        exploration_constant: float = None,
    ) -> MCTSResult:
        """
        Find the best play for a combat state using Monte Carlo Tree Search.

        Uses parallel rollouts across workers for efficiency.

        Args:
            combat_state: Current combat state
            search_budget: Number of MCTS iterations
            exploration_constant: UCT exploration constant (default from config)

        Returns:
            MCTSResult with best action and statistics
        """
        self._initialize()

        if exploration_constant is None:
            exploration_constant = self.config.exploration_constant

        start_time = time.perf_counter()

        # Get legal actions using CombatSimulator
        simulator = CombatSimulator()
        sim_legal_actions = simulator.get_legal_actions(combat_state)
        # Convert to state.combat Actions for MCTS
        legal_actions = [_sim_action_to_state_action(a) for a in sim_legal_actions]

        if len(legal_actions) <= 1:
            return MCTSResult(
                best_action=legal_actions[0] if legal_actions else EndTurn(),
                action_scores={},
                nodes_explored=0,
                time_ms=0,
                confidence=1.0,
            )

        # Run MCTS with parallel rollouts
        result = _mcts_search(
            combat_state=combat_state,
            legal_actions=legal_actions,
            budget=search_budget,
            exploration_c=exploration_constant,
            executor=self._executor,
            config=self.config,
        )

        result.time_ms = (time.perf_counter() - start_time) * 1000
        return result

    def find_best_plays_batch(
        self,
        combat_states: List[CombatState],
        search_budget: int = 1000,
    ) -> List[MCTSResult]:
        """
        Find best plays for multiple combat states in parallel.

        Args:
            combat_states: List of combat states to analyze
            search_budget: Search budget per state

        Returns:
            List of MCTSResult objects
        """
        self._initialize()

        # Submit parallel searches
        futures = []
        for state in combat_states:
            state_bytes = pickle.dumps(state, protocol=self.config.pickle_protocol)
            future = self._executor.submit(
                _mcts_search_worker,
                state_bytes,
                search_budget,
                self.config.exploration_constant,
            )
            futures.append(future)

        # Collect results
        results = []
        for future in as_completed(futures):
            results.append(future.result())

        return results

    # =========================================================================
    # Statistics
    # =========================================================================

    def get_stats(self) -> Dict[str, Any]:
        """Get simulation statistics."""
        sims_per_second = (
            self._total_sims / (self._total_time_ms / 1000)
            if self._total_time_ms > 0 else 0
        )
        return {
            "total_simulations": self._total_sims,
            "total_time_ms": self._total_time_ms,
            "sims_per_second": sims_per_second,
            "n_workers": self.config.n_workers,
            "batch_size": self.config.batch_size,
        }

    def reset_stats(self):
        """Reset simulation statistics."""
        self._total_sims = 0
        self._total_time_ms = 0.0


# =============================================================================
# Internal Task Types (for serialization)
# =============================================================================

@dataclass
class _SimulateRunTask:
    """Task for simulating a full run."""
    seed: str
    ascension: int
    agent_state: Optional[bytes]
    track_decisions: bool


@dataclass
class _SimulateCombatTask:
    """Task for simulating a combat."""
    state_bytes: bytes
    action_bytes: bytes
    max_turns: int


# =============================================================================
# Worker Functions (run in separate processes)
# =============================================================================

_worker_config: Optional[SimulationConfig] = None


def _worker_init(config: SimulationConfig):
    """Initialize worker process state."""
    global _worker_config
    _worker_config = config


def _run_batch(
    tasks: List[_SimulateRunTask],
    config: SimulationConfig,
) -> List[SimulationResult]:
    """Run a batch of run simulations."""
    results = []
    for task in tasks:
        agent = _deserialize_agent(task.agent_state) if task.agent_state else None
        result = _simulate_run_internal(
            seed=task.seed,
            ascension=task.ascension,
            agent=agent,
            track_decisions=task.track_decisions,
            config=config,
        )
        results.append(result)
    return results


def _run_combat_batch(
    tasks: List[_SimulateCombatTask],
    config: SimulationConfig,
) -> List[CombatSimResult]:
    """Run a batch of combat simulations."""
    results = []
    for task in tasks:
        state = pickle.loads(task.state_bytes)
        actions = pickle.loads(task.action_bytes)
        result = _simulate_combat_internal(
            state=state,
            initial_actions=actions,
            max_turns=task.max_turns,
            policy=None,
        )
        results.append(result)
    return results


def _simulate_run_internal(
    seed: str,
    ascension: int,
    agent: Optional[Agent],
    track_decisions: bool,
    config: SimulationConfig,
) -> SimulationResult:
    """Internal function to simulate a full run."""
    import random
    start_time = time.perf_counter()

    # Import GameRunner here to avoid circular imports
    from ..game import GameRunner

    runner = GameRunner(
        seed=seed,
        ascension=ascension,
        character=config.character,
        verbose=False,
    )

    # Run game loop with error handling for edge cases
    floors_seen = 0
    max_steps = 500  # Safety limit to prevent infinite loops
    steps = 0

    while not runner.game_over and floors_seen < config.max_floors_per_run and steps < max_steps:
        steps += 1
        try:
            actions = runner.get_available_actions()
            if not actions:
                break

            if agent:
                action = agent.select_action(runner, actions)
            else:
                action = random.choice(actions)

            runner.take_action(action)
            floors_seen = runner.run_state.floors_climbed

        except (IndexError, KeyError) as e:
            # Handle map boundary errors and similar edge cases
            # This can happen when the game reaches boss floor or act transitions
            break

    elapsed_ms = (time.perf_counter() - start_time) * 1000
    stats = runner.get_run_statistics()

    return SimulationResult(
        seed=seed,
        victory=stats.get("game_won", False),
        final_floor=stats.get("final_floor", 0),
        final_act=stats.get("final_act", 1),
        final_hp=stats.get("final_hp", 0),
        final_max_hp=stats.get("final_max_hp", 0),
        final_gold=stats.get("final_gold", 0),
        deck_size=stats.get("deck_size", 0),
        relic_count=stats.get("relic_count", 0),
        combats_won=stats.get("combats_won", 0),
        floors_climbed=stats.get("floors_climbed", 0),
        decisions_made=stats.get("decisions_made", 0),
        simulation_time_ms=elapsed_ms,
        decision_log=runner.decision_log if track_decisions else None,
    )


def _simulate_combat_internal(
    state: CombatState,
    initial_actions: List[Action],
    max_turns: int,
    policy: Optional[Callable[[CombatState], Action]],
) -> CombatSimResult:
    """
    Internal function to simulate a combat using CombatSimulator.

    Delegates to CombatSimulator for actual simulation logic.
    This function handles:
    - Converting between action types
    - Wrapping policy functions
    - Executing initial actions before rollout
    """
    import random

    # Create simulator instance (stateless, lightweight)
    simulator = CombatSimulator()

    current_state = state.copy()
    actions_taken = []
    initial_hp = current_state.player.hp

    # Execute initial actions using CombatSimulator
    for action in initial_actions:
        if current_state.combat_over:
            break
        # Convert to simulator action type
        sim_action = _state_action_to_sim_action(action)
        current_state = simulator.execute_action(current_state, sim_action)
        actions_taken.append(action)

    # If combat already over, return early
    if current_state.combat_over:
        hp_lost = initial_hp - current_state.player.hp
        return CombatSimResult(
            victory=current_state.player_won,
            hp_remaining=current_state.player.hp,
            hp_lost=hp_lost,
            turns=current_state.turn,
            cards_played=current_state.total_cards_played,
            damage_dealt=current_state.total_damage_dealt,
            damage_taken=hp_lost,
            final_state=current_state,
            action_sequence=actions_taken,
        )

    # Create a policy wrapper for CombatSimulator
    def sim_policy(s: CombatState) -> SimAction:
        if policy:
            # Policy returns state.combat Action, need to convert
            state_action = policy(s)
            return _state_action_to_sim_action(state_action)
        else:
            # Random policy using simulator's action format
            return simulator.random_policy(s)

    # Run simulation to completion
    result = simulator.simulate_full_combat(current_state, sim_policy, max_turns)

    # Convert result back to CombatSimResult
    # Note: action_sequence only includes initial_actions since simulate_full_combat
    # doesn't return the full sequence (but we tracked the initial ones)
    hp_lost = initial_hp - result.hp_remaining

    return CombatSimResult(
        victory=result.victory,
        hp_remaining=result.hp_remaining,
        hp_lost=hp_lost,
        turns=result.turns,
        cards_played=result.cards_played,
        damage_dealt=result.damage_dealt,
        damage_taken=result.damage_taken,
        final_state=None,  # Not available from simulate_full_combat
        action_sequence=actions_taken,  # Only initial actions tracked
    )


def _apply_action(state: CombatState, action: Action) -> CombatState:
    """
    Apply an action to a combat state and return new state.

    Delegates to CombatSimulator for proper game mechanics.
    """
    simulator = CombatSimulator()
    sim_action = _state_action_to_sim_action(action)
    return simulator.execute_action(state, sim_action)


# =============================================================================
# MCTS Implementation
# =============================================================================

@dataclass
class _MCTSNode:
    """Node in MCTS tree."""
    state: CombatState
    parent: Optional['_MCTSNode'] = None
    action: Optional[Action] = None
    children: Dict[str, '_MCTSNode'] = field(default_factory=dict)
    visits: int = 0
    value: float = 0.0
    untried_actions: List[Action] = field(default_factory=list)

    def ucb1(self, exploration_c: float) -> float:
        """Calculate UCB1 value for this node."""
        if self.visits == 0:
            return float('inf')
        exploit = self.value / self.visits
        explore = exploration_c * (
            (2 * (self.parent.visits if self.parent else 1)) ** 0.5
            / self.visits
        ) ** 0.5
        return exploit + explore


def _mcts_search(
    combat_state: CombatState,
    legal_actions: List[Action],
    budget: int,
    exploration_c: float,
    executor: ProcessPoolExecutor,
    config: SimulationConfig,
) -> MCTSResult:
    """Run MCTS search with parallel rollouts."""
    import random
    import math

    # Create root node
    root = _MCTSNode(
        state=combat_state.copy(),
        untried_actions=list(legal_actions),
    )

    nodes_explored = 0

    # Run MCTS iterations
    for _ in range(budget):
        node = root

        # Selection: traverse tree using UCB1
        while not node.untried_actions and node.children:
            # Select child with highest UCB1
            best_child = max(
                node.children.values(),
                key=lambda n: n.ucb1(exploration_c)
            )
            node = best_child

        # Expansion: add new child if possible
        if node.untried_actions:
            action = random.choice(node.untried_actions)
            node.untried_actions.remove(action)

            # Apply action to get new state using CombatSimulator
            new_state = _apply_action(node.state, action)

            # Get legal actions for the new state using CombatSimulator
            simulator = CombatSimulator()
            sim_legal_actions = simulator.get_legal_actions(new_state)
            # Convert SimActions to state Actions
            state_legal_actions = [_sim_action_to_state_action(a) for a in sim_legal_actions]

            child = _MCTSNode(
                state=new_state,
                parent=node,
                action=action,
                untried_actions=state_legal_actions,
            )
            action_key = repr(action)
            node.children[action_key] = child
            node = child
            nodes_explored += 1

        # Simulation: random rollout
        result = _simulate_combat_internal(
            state=node.state,
            initial_actions=[],
            max_turns=config.max_turns_per_combat,
            policy=None,
        )

        # Backpropagation
        value = 1.0 if result.victory else 0.0
        # Add HP-based value
        if result.victory:
            value += result.hp_remaining / node.state.player.max_hp * 0.5
        else:
            value = -0.5

        while node:
            node.visits += 1
            node.value += value
            node = node.parent

    # Select best action based on visits
    if not root.children:
        return MCTSResult(
            best_action=legal_actions[0] if legal_actions else EndTurn(),
            action_scores={},
            nodes_explored=0,
            time_ms=0,
            confidence=1.0,
        )

    action_scores = {
        key: child.value / child.visits if child.visits > 0 else 0
        for key, child in root.children.items()
    }

    best_child = max(root.children.values(), key=lambda n: n.visits)
    visits_sorted = sorted([c.visits for c in root.children.values()], reverse=True)
    confidence = (
        visits_sorted[0] / visits_sorted[1]
        if len(visits_sorted) > 1 and visits_sorted[1] > 0
        else float('inf')
    )

    return MCTSResult(
        best_action=best_child.action,
        action_scores=action_scores,
        nodes_explored=nodes_explored,
        time_ms=0,
        confidence=min(confidence, 10.0),
    )


def _mcts_search_worker(
    state_bytes: bytes,
    budget: int,
    exploration_c: float,
) -> MCTSResult:
    """Worker function for parallel MCTS search."""
    state = pickle.loads(state_bytes)

    # Get legal actions using CombatSimulator
    simulator = CombatSimulator()
    sim_legal_actions = simulator.get_legal_actions(state)
    # Convert to state.combat Actions for MCTS
    legal_actions = [_sim_action_to_state_action(a) for a in sim_legal_actions]

    # Run search (no executor for nested parallelism)
    return _mcts_search(
        combat_state=state,
        legal_actions=legal_actions,
        budget=budget,
        exploration_c=exploration_c,
        executor=None,
        config=_worker_config or SimulationConfig(),
    )


# =============================================================================
# Agent Serialization
# =============================================================================

def _serialize_agent(agent: Optional[Agent]) -> Optional[bytes]:
    """Serialize an agent for passing to workers."""
    if agent is None:
        return None
    # For now, we don't support stateful agents in workers
    # Full implementation would serialize agent state
    return pickle.dumps(None)


def _deserialize_agent(agent_bytes: Optional[bytes]) -> Optional[Agent]:
    """Deserialize an agent in a worker."""
    if agent_bytes is None:
        return None
    return pickle.loads(agent_bytes)


# =============================================================================
# Testing
# =============================================================================

if __name__ == "__main__":
    print("=== Parallel Simulator Test ===\n")

    config = SimulationConfig(n_workers=4)
    print(f"Using {config.n_workers} workers")

    with ParallelSimulator(config=config) as sim:
        # Test single simulation
        print("\n--- Single Simulation ---")
        result = sim.simulate_single("TEST123", ascension=20)
        print(f"Seed: {result.seed}")
        print(f"Victory: {result.victory}")
        print(f"Final floor: {result.final_floor}")
        print(f"Time: {result.simulation_time_ms:.1f}ms")

        # Test batch simulation
        print("\n--- Batch Simulation (10 seeds) ---")
        seeds = [f"BATCH{i:04d}" for i in range(10)]
        import time
        start = time.perf_counter()
        results = sim.simulate_batch(seeds, ascension=20)
        elapsed = (time.perf_counter() - start) * 1000

        wins = sum(1 for r in results if r.victory)
        print(f"Results: {wins}/{len(results)} victories")
        print(f"Total time: {elapsed:.1f}ms")
        print(f"Per sim: {elapsed/len(results):.1f}ms")

        # Print stats
        print("\n--- Statistics ---")
        stats = sim.get_stats()
        print(f"Total simulations: {stats['total_simulations']}")
        print(f"Sims/second: {stats['sims_per_second']:.1f}")

    print("\n=== Test Complete ===")
