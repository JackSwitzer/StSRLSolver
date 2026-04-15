"""Seed Conquerer: 10x parallel beam search on the same seed.

Runs 10 games with the same seed but different decision strategies,
tracks divergence points, and selects the best path.

Usage:
    from packages.training.conquerer import SeedConquerer

    conq = SeedConquerer(num_paths=10, ascension=20)
    result = conq.conquer("TEST123")
    print(f"Win rate: {result.win_count}/{len(result.paths)}")
    print(f"Best path: {result.best_path_id}, floor {result.paths[result.best_path_id].floors_reached}")
"""

from __future__ import annotations

import hashlib
import time
from concurrent.futures import ProcessPoolExecutor, as_completed
from dataclasses import dataclass
from typing import Any, Dict, List, Optional, Tuple


@dataclass
class PathResult:
    """Result of one decision path."""

    path_id: int
    seed: str
    won: bool
    floors_reached: int
    hp_remaining: int
    total_reward: float
    decision_log: List[Dict[str, Any]]  # [{floor, phase, action_id, alternatives}]
    divergence_points: List[int]  # Floor numbers where this path diverged from path 0


@dataclass
class ConquererResult:
    """Result of conquering a seed with multiple paths."""

    seed: str
    paths: List[PathResult]
    best_path_id: int
    win_count: int
    max_floor: int
    divergence_tree: Dict[str, Any]  # Tree structure of decision divergences
    elapsed_seconds: float


# =========================================================================
# Top-level function for ProcessPoolExecutor pickling
# =========================================================================


def _run_path(
    seed: str,
    path_id: int,
    ascension: int,
    character: str,
    max_steps: int,
) -> PathResult:
    """Run a single path in a worker process.

    Top-level function so ProcessPoolExecutor can pickle it.
    """
    from packages.engine.game import GameRunner

    runner = GameRunner(
        seed=seed,
        ascension=ascension,
        character=character,
        verbose=False,
    )

    decision_log: List[Dict[str, Any]] = []
    steps = 0

    while not runner.game_over and steps < max_steps:
        actions = runner.get_available_action_dicts()
        if not actions:
            break

        action = _pick_action(path_id, actions, runner)

        decision_log.append(
            {
                "floor": getattr(runner.run_state, "floor", 0),
                "phase": str(runner.phase),
                "action_id": action.get("id", ""),
                "alternatives": len(actions),
            }
        )

        runner.take_action_dict(action)
        steps += 1

    obs = runner.get_observation()
    hp = obs.get("run", {}).get("current_hp", 0) or 0
    floor_reached = getattr(runner.run_state, "floor", 0)

    # Reward: 1.0 for win, else floor progress normalized to [0, 1)
    total_reward = 1.0 if runner.game_won else floor_reached / 60.0

    return PathResult(
        path_id=path_id,
        seed=seed,
        won=runner.game_won,
        floors_reached=floor_reached,
        hp_remaining=hp,
        total_reward=total_reward,
        decision_log=decision_log,
        divergence_points=[],  # Filled in post-processing
    )


def _pick_action(
    path_id: int,
    actions: List[Dict[str, Any]],
    runner: Any,
) -> Dict[str, Any]:
    """Pick action based on path strategy.

    Strategies by path_id:
    - 0: Greedy (first action -- baseline)
    - 1-3: Random with different temperatures (0.5, 1.0, 2.0)
    - 4-6: Heuristic offset (pick action at index priority % len)
    - 7-9: Weighted random with floor-seeded RNG
    """
    import random as _random

    if len(actions) == 1:
        return actions[0]

    if path_id == 0:
        # Greedy baseline: always first action
        return actions[0]

    if 1 <= path_id <= 3:
        # Temperature-scaled random sampling
        temperatures = {1: 0.5, 2: 1.0, 3: 2.0}
        temp = temperatures[path_id]
        return _temperature_sample(actions, temp, path_id, runner)

    if 4 <= path_id <= 6:
        # Heuristic: pick action at offset determined by priority
        priority = path_id - 4
        return _heuristic_pick(actions, runner, priority)

    # path_id 7-9: weighted random with different budgets
    return _weighted_random_pick(actions, runner, path_id)


def _temperature_sample(
    actions: List[Dict[str, Any]],
    temperature: float,
    path_id: int,
    runner: Any,
) -> Dict[str, Any]:
    """Sample an action using softmax with temperature over uniform logits."""
    import math
    import random as _random

    n = len(actions)
    # Deterministic seed from floor + path_id + step count proxy
    floor = getattr(runner.run_state, "floor", 0)
    h = int(
        hashlib.md5(f"{floor}:{path_id}:{n}".encode()).hexdigest()[:8], 16
    )
    rng = _random.Random(h)

    if temperature <= 0.01:
        return actions[0]

    # Uniform logits -> softmax with temperature gives uniform anyway,
    # but we add position-based bias so higher temp = more exploration
    weights = []
    for i in range(n):
        # Slight positional bias: first actions get higher base score
        logit = (n - i) / n
        w = math.exp(logit / temperature)
        weights.append(w)

    total = sum(weights)
    r = rng.random() * total
    cumulative = 0.0
    for i, w in enumerate(weights):
        cumulative += w
        if r <= cumulative:
            return actions[i]

    return actions[-1]


def _heuristic_pick(
    actions: List[Dict[str, Any]],
    runner: Any,
    priority: int,
) -> Dict[str, Any]:
    """Simple heuristic: score actions by type and pick best for priority level.

    priority 0: prefer attacks/damage in combat, first action otherwise
    priority 1: prefer blocks/defense in combat, skip in rewards
    priority 2: prefer end_turn less, explore more options
    """
    phase_str = str(runner.phase).lower()

    if "combat" in phase_str and not "reward" in phase_str:
        # In combat: score actions
        scored = []
        for i, a in enumerate(actions):
            atype = a.get("type", "")
            score = 0.0

            if atype == "play_card":
                card_id = a.get("params", {}).get("card_id", "")
                card_lower = card_id.lower()

                if priority == 0:
                    # Prefer attacks
                    if "strike" in card_lower or "attack" in atype:
                        score = 2.0
                    else:
                        score = 1.0
                elif priority == 1:
                    # Prefer blocks
                    if "defend" in card_lower or "block" in card_lower:
                        score = 2.0
                    else:
                        score = 1.0
                else:
                    # Balanced: slightly prefer playing cards
                    score = 1.5
            elif atype == "end_turn":
                score = 0.1 if priority != 2 else 0.5

            scored.append((score, i))

        scored.sort(key=lambda x: -x[0])
        return actions[scored[0][1]]

    # Non-combat: use offset
    idx = min(priority, len(actions) - 1)
    return actions[idx]


def _weighted_random_pick(
    actions: List[Dict[str, Any]],
    runner: Any,
    path_id: int,
) -> Dict[str, Any]:
    """Weighted random pick with path-specific seeding."""
    import random as _random

    floor = getattr(runner.run_state, "floor", 0)
    n = len(actions)
    h = int(
        hashlib.md5(f"wr:{floor}:{path_id}:{n}".encode()).hexdigest()[:8], 16
    )
    rng = _random.Random(h)

    # Give slightly higher weight to later actions (more diverse)
    weights = [1.0 + (i * 0.3) for i in range(n)]
    total = sum(weights)
    r = rng.random() * total
    cumulative = 0.0
    for i, w in enumerate(weights):
        cumulative += w
        if r <= cumulative:
            return actions[i]

    return actions[-1]


# =========================================================================
# SeedConquerer
# =========================================================================


class SeedConquerer:
    """Run multiple paths on the same seed with different strategies."""

    def __init__(
        self,
        num_paths: int = 10,
        ascension: int = 20,
        character: str = "Watcher",
        max_steps: int = 3000,
        parallel: bool = True,
        max_workers: Optional[int] = None,
    ):
        self.num_paths = num_paths
        self.ascension = ascension
        self.character = character
        self.max_steps = max_steps
        self.parallel = parallel
        self.max_workers = max_workers

    def conquer(self, seed: str) -> ConquererResult:
        """Run num_paths games on the same seed with different strategies.

        Path strategies:
        - Path 0: Greedy (always pick first action -- baseline)
        - Path 1-3: Random with different temperatures (0.5, 1.0, 2.0)
        - Path 4-6: Heuristic with different priorities (attack/block/balanced)
        - Path 7-9: Weighted random with different seeds
        """
        start = time.time()

        if self.parallel:
            results = self._run_parallel(seed)
        else:
            results = self._run_sequential(seed)

        # Sort by path_id
        results.sort(key=lambda r: r.path_id)

        # Compute divergence points against baseline (path 0)
        baseline_log = results[0].decision_log if results else []
        for r in results[1:]:
            r.divergence_points = _find_divergence_points(baseline_log, r.decision_log)

        # Select best path
        best = self._select_best(results)

        # Build divergence tree
        tree = self._build_divergence_tree(results)

        return ConquererResult(
            seed=seed,
            paths=results,
            best_path_id=best.path_id,
            win_count=sum(1 for r in results if r.won),
            max_floor=max((r.floors_reached for r in results), default=0),
            divergence_tree=tree,
            elapsed_seconds=time.time() - start,
        )

    def _run_parallel(self, seed: str) -> List[PathResult]:
        """Run all paths in parallel using ProcessPoolExecutor."""
        results: List[PathResult] = []
        workers = self.max_workers or min(self.num_paths, 4)

        with ProcessPoolExecutor(max_workers=workers) as executor:
            futures = {
                executor.submit(
                    _run_path,
                    seed,
                    path_id,
                    self.ascension,
                    self.character,
                    self.max_steps,
                ): path_id
                for path_id in range(self.num_paths)
            }
            for future in as_completed(futures):
                results.append(future.result())

        return results

    def _run_sequential(self, seed: str) -> List[PathResult]:
        """Run all paths sequentially."""
        return [
            _run_path(seed, path_id, self.ascension, self.character, self.max_steps)
            for path_id in range(self.num_paths)
        ]

    def _select_best(self, results: List[PathResult]) -> PathResult:
        """Select best path: win > furthest floor > highest HP > highest reward."""
        if not results:
            raise ValueError("No results to select from")

        def sort_key(r: PathResult) -> Tuple[int, int, int, float]:
            return (int(r.won), r.floors_reached, r.hp_remaining, r.total_reward)

        return max(results, key=sort_key)

    def _build_divergence_tree(self, results: List[PathResult]) -> Dict[str, Any]:
        """Build a summary of divergence across paths.

        Returns a dict mapping floor numbers to the set of distinct
        action_ids chosen across paths at that floor.
        """
        floor_actions: Dict[int, Dict[int, str]] = {}  # floor -> {path_id -> action_id}

        for r in results:
            for entry in r.decision_log:
                floor = entry["floor"]
                if floor not in floor_actions:
                    floor_actions[floor] = {}
                # Use first action at each floor per path
                if r.path_id not in floor_actions[floor]:
                    floor_actions[floor][r.path_id] = entry["action_id"]

        # Identify floors where paths diverge
        divergence_tree: Dict[str, Any] = {
            "total_paths": len(results),
            "divergent_floors": {},
        }

        for floor in sorted(floor_actions.keys()):
            actions_at_floor = floor_actions[floor]
            unique_actions = set(actions_at_floor.values())
            if len(unique_actions) > 1:
                divergence_tree["divergent_floors"][str(floor)] = {
                    "unique_actions": len(unique_actions),
                    "path_actions": {
                        str(pid): aid for pid, aid in sorted(actions_at_floor.items())
                    },
                }

        return divergence_tree


def _find_divergence_points(
    baseline_log: List[Dict[str, Any]],
    compare_log: List[Dict[str, Any]],
) -> List[int]:
    """Find floor numbers where compare_log diverged from baseline_log."""
    divergence_floors: List[int] = []
    seen_floors: set = set()

    # Build baseline mapping: (floor, step_within_floor) -> action_id
    baseline_by_floor: Dict[int, List[str]] = {}
    for entry in baseline_log:
        floor = entry["floor"]
        baseline_by_floor.setdefault(floor, []).append(entry["action_id"])

    compare_by_floor: Dict[int, List[str]] = {}
    for entry in compare_log:
        floor = entry["floor"]
        compare_by_floor.setdefault(floor, []).append(entry["action_id"])

    # Compare action sequences per floor
    all_floors = sorted(set(baseline_by_floor.keys()) | set(compare_by_floor.keys()))
    for floor in all_floors:
        b_actions = baseline_by_floor.get(floor, [])
        c_actions = compare_by_floor.get(floor, [])
        if b_actions != c_actions and floor not in seen_floors:
            divergence_floors.append(floor)
            seen_floors.add(floor)

    return divergence_floors
