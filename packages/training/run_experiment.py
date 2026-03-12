"""
Experiment runner: standardized algorithm evaluation and comparison.

Wraps benchmark.py + self_play.py for reproducible experiments with:
- Configurable algorithm selection
- Periodic benchmark snapshots during training
- Checkpoint management
- Experiment comparison (learning curves, final metrics)

Usage:
    # Quick eval of any agent
    uv run python -m packages.training.run_experiment --algo heuristic --episodes 0

    # Self-play training with periodic benchmarks
    uv run python -m packages.training.run_experiment \
      --algo selfplay --episodes 1000 --benchmark-every 200 \
      --checkpoint-dir logs/experiments/exp_001

    # Compare experiments
    uv run python -m packages.training.run_experiment \
      --compare logs/experiments/exp_001 logs/experiments/exp_002
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional

from packages.training.benchmark import (
    BenchmarkResult,
    compare,
    evaluate,
    print_result,
    quick_eval,
)


@dataclass
class ExperimentConfig:
    """Configuration for an experiment run."""
    algo: str
    episodes: int = 0
    benchmark_every: int = 100
    benchmark_seeds: int = 0  # 0 = full catalog
    workers: int = 8
    ascension: int = 20
    character: str = "Watcher"
    checkpoint_dir: str = "logs/experiments/default"

    def save(self, path: Path) -> None:
        with open(path, "w") as f:
            json.dump(asdict(self), f, indent=2)

    @classmethod
    def load(cls, path: Path) -> "ExperimentConfig":
        with open(path) as f:
            return cls(**json.load(f))


@dataclass
class ExperimentResult:
    """Results from a complete experiment."""
    config: Dict[str, Any]
    benchmarks: List[Dict[str, Any]] = field(default_factory=list)
    total_episodes: int = 0
    total_time_s: float = 0.0

    def save(self, path: Path) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        with open(path, "w") as f:
            json.dump(asdict(self), f, indent=2)

    @classmethod
    def load(cls, path: Path) -> "ExperimentResult":
        with open(path) as f:
            data = json.load(f)
        return cls(**data)


def run_benchmark_only(config: ExperimentConfig) -> ExperimentResult:
    """Run a single benchmark evaluation (no training)."""
    exp_dir = Path(config.checkpoint_dir)
    exp_dir.mkdir(parents=True, exist_ok=True)
    config.save(exp_dir / "config.json")

    t0 = time.monotonic()

    if config.benchmark_seeds > 0:
        result = quick_eval(
            config.algo,
            num_seeds=config.benchmark_seeds,
            num_workers=config.workers,
            ascension=config.ascension,
            character=config.character,
        )
    else:
        result = evaluate(
            config.algo,
            num_workers=config.workers,
            ascension=config.ascension,
            character=config.character,
        )

    print_result(result)
    result.save(exp_dir / "benchmark_final.json")

    exp_result = ExperimentResult(
        config=asdict(config),
        benchmarks=[{
            "episode": 0,
            "win_rate": result.win_rate,
            "avg_floor": result.avg_floor,
            "timestamp": result.timestamp,
        }],
        total_episodes=0,
        total_time_s=round(time.monotonic() - t0, 1),
    )
    exp_result.save(exp_dir / "experiment.json")
    return exp_result


def run_selfplay_experiment(config: ExperimentConfig) -> ExperimentResult:
    """Run self-play training with periodic benchmarks."""
    from packages.training.self_play import SelfPlayTrainer

    exp_dir = Path(config.checkpoint_dir)
    exp_dir.mkdir(parents=True, exist_ok=True)
    config.save(exp_dir / "config.json")

    t0 = time.monotonic()
    benchmarks = []

    # Initial benchmark
    print(f"\n--- Initial benchmark (episode 0) ---")
    initial = quick_eval("first", num_seeds=10, num_workers=config.workers)
    print_result(initial)
    benchmarks.append({
        "episode": 0,
        "win_rate": initial.win_rate,
        "avg_floor": initial.avg_floor,
        "timestamp": initial.timestamp,
    })

    # Training loop
    trainer = SelfPlayTrainer(
        num_workers=config.workers,
        checkpoint_dir=exp_dir / "checkpoints",
    )

    episodes_done = 0
    while episodes_done < config.episodes:
        batch_size = min(config.benchmark_every, config.episodes - episodes_done)
        print(f"\n--- Training episodes {episodes_done+1}-{episodes_done+batch_size} ---")

        try:
            trainer.train(num_episodes=batch_size)
        except Exception as e:
            print(f"Training error: {e}")
            break

        episodes_done += batch_size

        # Periodic benchmark
        print(f"\n--- Benchmark at episode {episodes_done} ---")
        bench = quick_eval("first", num_seeds=10, num_workers=config.workers)
        print_result(bench)
        bench.save(exp_dir / f"benchmark_{episodes_done:06d}.json")
        benchmarks.append({
            "episode": episodes_done,
            "win_rate": bench.win_rate,
            "avg_floor": bench.avg_floor,
            "timestamp": bench.timestamp,
        })

    # Final full benchmark
    if config.episodes > 0:
        print(f"\n--- Final benchmark (full catalog) ---")
        try:
            final = evaluate(config.algo, num_workers=config.workers)
            print_result(final)
            final.save(exp_dir / "benchmark_final.json")
            benchmarks.append({
                "episode": episodes_done,
                "win_rate": final.win_rate,
                "avg_floor": final.avg_floor,
                "timestamp": final.timestamp,
                "is_final": True,
            })
        except Exception as e:
            print(f"Final benchmark error: {e}")

    exp_result = ExperimentResult(
        config=asdict(config),
        benchmarks=benchmarks,
        total_episodes=episodes_done,
        total_time_s=round(time.monotonic() - t0, 1),
    )
    exp_result.save(exp_dir / "experiment.json")
    return exp_result


def compare_experiments(*exp_dirs: str) -> str:
    """Compare multiple experiments side by side."""
    lines = []
    results = []

    for exp_dir_str in exp_dirs:
        exp_dir = Path(exp_dir_str)
        exp_path = exp_dir / "experiment.json"
        if not exp_path.exists():
            lines.append(f"Warning: {exp_path} not found, skipping")
            continue

        exp = ExperimentResult.load(exp_path)
        results.append((exp_dir.name, exp))

    if not results:
        return "No valid experiments found."

    # Summary table
    header = f"{'Experiment':<20} {'Episodes':>8} {'Final WR':>8} {'Final Floor':>11} {'Time':>8}"
    lines.append(header)
    lines.append("-" * len(header))

    for name, exp in results:
        final = exp.benchmarks[-1] if exp.benchmarks else {}
        lines.append(
            f"{name:<20} {exp.total_episodes:>8} "
            f"{final.get('win_rate', 0)*100:>7.1f}% "
            f"{final.get('avg_floor', 0):>11.1f} "
            f"{exp.total_time_s:>6.0f}s"
        )

    # Learning curves
    lines.append(f"\n{'Learning Curves':=^60}")
    for name, exp in results:
        lines.append(f"\n  {name}:")
        for b in exp.benchmarks:
            ep = b.get("episode", 0)
            wr = b.get("win_rate", 0) * 100
            fl = b.get("avg_floor", 0)
            marker = " <-- FINAL" if b.get("is_final") else ""
            lines.append(f"    ep {ep:>6}: {wr:>5.1f}% WR, floor {fl:.1f}{marker}")

    return "\n".join(lines)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Run STS RL experiments")
    parser.add_argument("--algo", type=str, default="heuristic",
                        help="Algorithm: random, first, heuristic, mcts64, gumbel16, selfplay")
    parser.add_argument("--episodes", type=int, default=0,
                        help="Training episodes (0 = benchmark only)")
    parser.add_argument("--benchmark-every", type=int, default=200,
                        help="Benchmark interval during training")
    parser.add_argument("--benchmark-seeds", type=int, default=0,
                        help="Quick eval seed count (0 = full catalog)")
    parser.add_argument("--workers", type=int, default=8)
    parser.add_argument("--checkpoint-dir", type=str, default=None,
                        help="Output directory for results")
    parser.add_argument("--compare", nargs="+", type=str, default=None,
                        help="Compare experiment directories")

    args = parser.parse_args()

    if args.compare:
        print(compare_experiments(*args.compare))
        sys.exit(0)

    # Default checkpoint dir
    if args.checkpoint_dir is None:
        ts = time.strftime("%Y%m%d_%H%M%S")
        args.checkpoint_dir = f"logs/experiments/{args.algo}_{ts}"

    config = ExperimentConfig(
        algo=args.algo,
        episodes=args.episodes,
        benchmark_every=args.benchmark_every,
        benchmark_seeds=args.benchmark_seeds,
        workers=args.workers,
        checkpoint_dir=args.checkpoint_dir,
    )

    if args.episodes == 0 or args.algo in ("random", "first", "heuristic", "mcts64", "mcts128", "planner") or args.algo.startswith("gumbel"):
        result = run_benchmark_only(config)
    else:
        result = run_selfplay_experiment(config)

    print(f"\nExperiment saved to {config.checkpoint_dir}/")
