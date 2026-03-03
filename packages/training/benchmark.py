"""
Benchmark harness for evaluating STS agents on a tiered seed catalog.

Tiered structure:
- 20 easy seeds (agent reaches floor 10+ with heuristic)
- 20 medium seeds (agent reaches floor 5-9)
- 20 hard seeds (agent reaches floor 1-4)
- 40 random seeds (regenerated each eval for generalization)

Usage:
    uv run python -m packages.training.benchmark --agent heuristic --workers 8
    uv run python -m packages.training.benchmark --compare logs/benchmarks/*.json
"""

from __future__ import annotations

import argparse
import hashlib
import json
import multiprocessing as mp
import sys
import time
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any, Callable, Dict, List, Optional, Tuple

# Seed catalog: curated during first run, stored to disk
SEED_CATALOG_PATH = Path("packages/training/benchmark_seeds.json")
BENCHMARK_DIR = Path("logs/benchmarks")

NUM_EASY = 20
NUM_MEDIUM = 20
NUM_HARD = 20
NUM_RANDOM = 40
TOTAL_SEEDS = NUM_EASY + NUM_MEDIUM + NUM_HARD + NUM_RANDOM


@dataclass
class SeedResult:
    """Result of playing one seed."""
    seed: str
    won: bool
    floor: int
    hp_remaining: int
    max_hp: int
    duration_s: float
    deck_size: int
    relic_count: int
    combats_won: int = 0
    act_reached: int = 1


@dataclass
class BenchmarkResult:
    """Aggregated benchmark results."""
    agent_name: str
    timestamp: str = ""
    total_seeds: int = 0
    # Overall
    win_rate: float = 0.0
    avg_floor: float = 0.0
    avg_hp_remaining: float = 0.0
    total_time_s: float = 0.0
    avg_time_per_game_s: float = 0.0
    # Per-tier
    easy_win_rate: float = 0.0
    easy_avg_floor: float = 0.0
    medium_win_rate: float = 0.0
    medium_avg_floor: float = 0.0
    hard_win_rate: float = 0.0
    hard_avg_floor: float = 0.0
    random_win_rate: float = 0.0
    random_avg_floor: float = 0.0
    # Per-seed results
    seed_results: List[Dict] = field(default_factory=list)

    def save(self, path: Optional[Path] = None) -> Path:
        if path is None:
            BENCHMARK_DIR.mkdir(parents=True, exist_ok=True)
            path = BENCHMARK_DIR / f"{self.agent_name}_{self.timestamp}.json"
        path.parent.mkdir(parents=True, exist_ok=True)
        with open(path, "w") as f:
            json.dump(asdict(self), f, indent=2)
        return path

    @classmethod
    def load(cls, path: Path) -> "BenchmarkResult":
        with open(path) as f:
            data = json.load(f)
        seed_results = data.pop("seed_results", [])
        result = cls(**{k: v for k, v in data.items() if k != "seed_results"})
        result.seed_results = seed_results
        return result


def _play_seed(args: Tuple) -> Dict:
    """Play a single seed with the given agent function. Worker function for mp.Pool."""
    seed, agent_name, ascension, character = args

    from packages.engine.game import GameRunner, GamePhase

    try:
        runner = GameRunner(seed=seed, ascension=ascension, character=character, verbose=False)
    except Exception as e:
        return {"seed": seed, "won": False, "floor": 0, "hp_remaining": 0, "max_hp": 72,
                "duration_s": 0, "deck_size": 0, "relic_count": 0, "error": str(e)}

    # Import agent inside worker to avoid pickling issues
    agent = _create_agent(agent_name)

    t0 = time.monotonic()
    step = 0
    combats_won = 0
    in_combat = False

    while not runner.game_over and step < 5000:
        try:
            actions = runner.get_available_actions()
        except Exception:
            break
        if not actions:
            break

        phase = runner.phase

        # Track combat wins
        if phase == GamePhase.COMBAT:
            in_combat = True
        elif in_combat:
            in_combat = False
            combats_won += 1

        try:
            action = agent(runner, actions, phase)
        except Exception:
            action = actions[0]

        if action is None:
            action = actions[0]

        try:
            runner.take_action(action)
        except Exception:
            break
        step += 1

    duration = time.monotonic() - t0
    rs = runner.run_state

    return {
        "seed": seed,
        "won": runner.game_won,
        "floor": getattr(rs, "floor", 0),
        "hp_remaining": getattr(rs, "current_hp", 0),
        "max_hp": getattr(rs, "max_hp", 72),
        "duration_s": round(duration, 2),
        "deck_size": len(getattr(rs, "deck", [])),
        "relic_count": len(getattr(rs, "relics", [])),
        "combats_won": combats_won,
        "act_reached": getattr(rs, "act", 1),
    }


def _create_agent(agent_name: str) -> Callable:
    """Create an agent function by name."""
    if agent_name == "random":
        import random as _rng
        def agent_fn(runner, actions, phase):
            return _rng.choice(actions)
        return agent_fn

    elif agent_name == "first":
        def agent_fn(runner, actions, phase):
            return actions[0]
        return agent_fn

    elif agent_name == "heuristic":
        from packages.training.planner import StSAgent
        sts_agent = StSAgent(combat_sims=32, temperature=0.0)
        def agent_fn(runner, actions, phase):
            return sts_agent.get_action(runner)
        return agent_fn

    elif agent_name == "mcts64":
        from packages.training.planner import StSAgent
        sts_agent = StSAgent(combat_sims=64, temperature=0.0)
        def agent_fn(runner, actions, phase):
            return sts_agent.get_action(runner)
        return agent_fn

    elif agent_name == "mcts128":
        from packages.training.planner import StSAgent
        sts_agent = StSAgent(combat_sims=128, temperature=0.0)
        def agent_fn(runner, actions, phase):
            return sts_agent.get_action(runner)
        return agent_fn

    elif agent_name.startswith("gumbel"):
        sims = int(agent_name.replace("gumbel", "") or "16")
        from packages.engine.game import GamePhase, CombatAction
        from packages.engine.state.combat import PlayCard, UsePotion, EndTurn
        from packages.training.gumbel_mcts import GumbelMCTS
        from packages.training.planner import StrategicPlanner
        gumbel = GumbelMCTS(num_simulations=sims)
        sp = StrategicPlanner()
        def agent_fn(runner, actions, phase):
            if phase == GamePhase.COMBAT:
                engine = runner.current_combat
                if engine and len(actions) > 1:
                    probs = gumbel.search(engine)
                    if probs:
                        best = max(probs, key=probs.get)
                        if isinstance(best, PlayCard):
                            return CombatAction(action_type="play_card", card_idx=best.card_idx, target_idx=best.target_idx)
                        elif isinstance(best, UsePotion):
                            return CombatAction(action_type="use_potion", potion_idx=best.potion_idx, target_idx=best.target_idx)
                        else:
                            return CombatAction(action_type="end_turn")
                return actions[0]
            elif phase == GamePhase.MAP_NAVIGATION:
                idx = sp.plan_path_choice(runner, actions)
                return actions[min(idx, len(actions) - 1)]
            elif phase == GamePhase.REST:
                idx = sp.plan_rest_site(runner, actions)
                return actions[min(idx, len(actions) - 1)]
            elif phase in (GamePhase.COMBAT_REWARDS, GamePhase.BOSS_REWARDS):
                idx = sp.plan_card_pick(runner, actions)
                return actions[min(idx, len(actions) - 1)]
            elif phase == GamePhase.SHOP:
                idx = sp.plan_shop_action(runner, actions)
                return actions[min(idx, len(actions) - 1)]
            elif phase == GamePhase.EVENT:
                idx = sp.plan_event_choice(runner, actions)
                return actions[min(idx, len(actions) - 1)]
            return actions[0]
        return agent_fn

    elif agent_name == "planner":
        from packages.engine.game import GamePhase, CombatAction
        from packages.training.combat_planner import CombatPlanner
        from packages.training.planner import StrategicPlanner
        cp = CombatPlanner(top_k=5, lookahead_turns=2)
        sp = StrategicPlanner()
        def agent_fn(runner, actions, phase):
            if phase == GamePhase.COMBAT:
                engine = runner.current_combat
                if engine and len(actions) > 1:
                    plan = cp.plan_turn(engine)
                    if plan and plan.card_sequence:
                        for card_id, target_idx in plan.card_sequence:
                            hand = engine.state.hand if engine else []
                            ci = next((i for i, h in enumerate(hand) if h == card_id), None)
                            if ci is not None:
                                t = target_idx if target_idx is not None else -1
                                try:
                                    runner.take_action(CombatAction(action_type="play_card", card_idx=ci, target_idx=t))
                                except Exception:
                                    break
                        if not runner.game_over and runner.phase == GamePhase.COMBAT:
                            try:
                                runner.take_action(CombatAction(action_type="end_turn"))
                            except Exception:
                                pass
                        return None  # Already executed
                return actions[0]
            elif phase == GamePhase.MAP_NAVIGATION:
                idx = sp.plan_path_choice(runner, actions)
                return actions[min(idx, len(actions) - 1)]
            elif phase == GamePhase.REST:
                idx = sp.plan_rest_site(runner, actions)
                return actions[min(idx, len(actions) - 1)]
            elif phase == GamePhase.COMBAT_REWARDS:
                idx = sp.plan_card_pick(runner, actions)
                return actions[min(idx, len(actions) - 1)]
            return actions[0]
        return agent_fn

    raise ValueError(f"Unknown agent: {agent_name}")


def _generate_seed_catalog(num_curation_seeds: int = 500) -> Dict[str, List[str]]:
    """Generate tiered seed catalog by running heuristic agent on many seeds."""
    print(f"Curating seed catalog from {num_curation_seeds} seeds...")

    seeds = [f"Bench_{i}" for i in range(num_curation_seeds)]
    args = [(s, "heuristic", 20, "Watcher") for s in seeds]

    with mp.Pool(min(8, mp.cpu_count())) as pool:
        results = pool.map(_play_seed, args)

    # Sort by floor reached
    results.sort(key=lambda r: r["floor"], reverse=True)

    easy = [r["seed"] for r in results[:NUM_EASY]]
    hard = [r["seed"] for r in results[-NUM_HARD:]]
    mid_start = len(results) // 2 - NUM_MEDIUM // 2
    medium = [r["seed"] for r in results[mid_start:mid_start + NUM_MEDIUM]]

    # Random seeds
    random_seeds = [f"Random_{i}_{int(time.time())}" for i in range(NUM_RANDOM)]

    catalog = {
        "easy": easy,
        "medium": medium,
        "hard": hard,
        "random": random_seeds,
        "meta": {
            "generated_at": time.strftime("%Y-%m-%d %H:%M:%S"),
            "curation_seeds": num_curation_seeds,
            "easy_floor_range": f"{results[NUM_EASY-1]['floor']}-{results[0]['floor']}",
            "medium_floor_range": f"{results[mid_start+NUM_MEDIUM-1]['floor']}-{results[mid_start]['floor']}",
            "hard_floor_range": f"{results[-1]['floor']}-{results[-NUM_HARD]['floor']}",
        },
    }

    SEED_CATALOG_PATH.parent.mkdir(parents=True, exist_ok=True)
    with open(SEED_CATALOG_PATH, "w") as f:
        json.dump(catalog, f, indent=2)

    print(f"Saved catalog: easy={catalog['meta']['easy_floor_range']}, "
          f"medium={catalog['meta']['medium_floor_range']}, "
          f"hard={catalog['meta']['hard_floor_range']}")

    return catalog


def load_seed_catalog() -> Dict[str, List[str]]:
    """Load or generate seed catalog."""
    if SEED_CATALOG_PATH.exists():
        with open(SEED_CATALOG_PATH) as f:
            return json.load(f)
    return _generate_seed_catalog()


def evaluate(
    agent_name: str,
    num_workers: int = 8,
    ascension: int = 20,
    character: str = "Watcher",
    seeds: Optional[List[str]] = None,
    tier_labels: Optional[Dict[str, List[str]]] = None,
) -> BenchmarkResult:
    """Evaluate an agent on the benchmark seed catalog.

    Args:
        agent_name: Name of agent to evaluate (random, first, heuristic, mcts64, mcts128, planner)
        num_workers: Number of parallel workers
        ascension: Ascension level
        character: Character to play
        seeds: Override seed list (default: load catalog)
        tier_labels: Override tier assignments

    Returns:
        BenchmarkResult with per-tier and per-seed breakdown
    """
    catalog = None
    if seeds is None:
        catalog = load_seed_catalog()
        all_seeds = catalog["easy"] + catalog["medium"] + catalog["hard"] + catalog["random"]
        tier_map = {}
        for s in catalog.get("easy", []):
            tier_map[s] = "easy"
        for s in catalog.get("medium", []):
            tier_map[s] = "medium"
        for s in catalog.get("hard", []):
            tier_map[s] = "hard"
        for s in catalog.get("random", []):
            tier_map[s] = "random"
    else:
        all_seeds = seeds
        tier_map = {}

    args = [(s, agent_name, ascension, character) for s in all_seeds]

    print(f"Evaluating '{agent_name}' on {len(all_seeds)} seeds with {num_workers} workers...")
    t0 = time.monotonic()

    with mp.Pool(num_workers) as pool:
        results = pool.map(_play_seed, args)

    total_time = time.monotonic() - t0

    # Aggregate
    wins = sum(1 for r in results if r["won"])
    floors = [r["floor"] for r in results]
    hps = [r["hp_remaining"] for r in results]
    n = len(results)

    # Per-tier
    tier_stats = {}
    for tier in ("easy", "medium", "hard", "random"):
        tier_results = [r for r in results if tier_map.get(r["seed"]) == tier]
        if tier_results:
            tw = sum(1 for r in tier_results if r["won"])
            tf = [r["floor"] for r in tier_results]
            tier_stats[tier] = {
                "win_rate": round(tw / len(tier_results), 3),
                "avg_floor": round(sum(tf) / len(tf), 1),
            }
        else:
            tier_stats[tier] = {"win_rate": 0.0, "avg_floor": 0.0}

    timestamp = time.strftime("%Y%m%d_%H%M%S")
    result = BenchmarkResult(
        agent_name=agent_name,
        timestamp=timestamp,
        total_seeds=n,
        win_rate=round(wins / max(n, 1), 3),
        avg_floor=round(sum(floors) / max(n, 1), 1),
        avg_hp_remaining=round(sum(hps) / max(n, 1), 1),
        total_time_s=round(total_time, 1),
        avg_time_per_game_s=round(total_time / max(n, 1), 2),
        easy_win_rate=tier_stats["easy"]["win_rate"],
        easy_avg_floor=tier_stats["easy"]["avg_floor"],
        medium_win_rate=tier_stats["medium"]["win_rate"],
        medium_avg_floor=tier_stats["medium"]["avg_floor"],
        hard_win_rate=tier_stats["hard"]["win_rate"],
        hard_avg_floor=tier_stats["hard"]["avg_floor"],
        random_win_rate=tier_stats["random"]["win_rate"],
        random_avg_floor=tier_stats["random"]["avg_floor"],
        seed_results=results,
    )

    return result


def compare(*results: BenchmarkResult) -> str:
    """Generate a comparison table for multiple benchmark results."""
    header = f"{'Agent':<16} {'Win%':>6} {'Floor':>6} {'Easy%':>6} {'Med%':>6} {'Hard%':>6} {'Rand%':>6} {'Time':>8}"
    lines = [header, "-" * len(header)]

    for r in results:
        lines.append(
            f"{r.agent_name:<16} {r.win_rate*100:>5.1f}% {r.avg_floor:>6.1f} "
            f"{r.easy_win_rate*100:>5.1f}% {r.medium_win_rate*100:>5.1f}% "
            f"{r.hard_win_rate*100:>5.1f}% {r.random_win_rate*100:>5.1f}% "
            f"{r.avg_time_per_game_s:>6.2f}s"
        )

    return "\n".join(lines)


def print_result(result: BenchmarkResult) -> None:
    """Pretty-print a benchmark result."""
    print(f"\n{'=' * 60}")
    print(f"  Benchmark: {result.agent_name}")
    print(f"  Seeds: {result.total_seeds}  |  Time: {result.total_time_s:.1f}s")
    print(f"{'=' * 60}")
    print(f"  Win Rate:     {result.win_rate*100:.1f}%")
    print(f"  Avg Floor:    {result.avg_floor:.1f}")
    print(f"  Avg HP Left:  {result.avg_hp_remaining:.1f}")
    print(f"  Time/Game:    {result.avg_time_per_game_s:.2f}s")
    print(f"  {'─' * 40}")
    print(f"  Easy:    {result.easy_win_rate*100:>5.1f}% WR  |  Floor {result.easy_avg_floor:.1f}")
    print(f"  Medium:  {result.medium_win_rate*100:>5.1f}% WR  |  Floor {result.medium_avg_floor:.1f}")
    print(f"  Hard:    {result.hard_win_rate*100:>5.1f}% WR  |  Floor {result.hard_avg_floor:.1f}")
    print(f"  Random:  {result.random_win_rate*100:>5.1f}% WR  |  Floor {result.random_avg_floor:.1f}")
    print(f"{'=' * 60}\n")


# CLI
if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Benchmark STS agents")
    parser.add_argument("--agent", type=str, default="heuristic",
                        help="Agent name: random, first, heuristic, mcts64, mcts128, planner")
    parser.add_argument("--workers", type=int, default=8)
    parser.add_argument("--compare", nargs="+", type=str, default=None,
                        help="Compare benchmark result JSON files")
    parser.add_argument("--regenerate-seeds", action="store_true",
                        help="Regenerate the seed catalog")
    args = parser.parse_args()

    if args.regenerate_seeds:
        _generate_seed_catalog()
        sys.exit(0)

    if args.compare:
        results = [BenchmarkResult.load(Path(p)) for p in args.compare]
        print(compare(*results))
        sys.exit(0)

    result = evaluate(args.agent, num_workers=args.workers)
    print_result(result)

    path = result.save()
    print(f"Saved to {path}")
