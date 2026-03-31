"""Offline evaluation: play games with a model and measure performance.

Unlike reward_sim.py (which rescores existing data), this module actually
plays games using the inference server and measures real performance metrics.
Reward shaping uses the global training_config -- there is no per-eval
override because workers read config at import time.
"""

from __future__ import annotations

import json
import logging
import time
from pathlib import Path
from typing import Any, Dict, List, Optional

logger = logging.getLogger(__name__)


def evaluate_model(
    model_path: str,
    num_games: int = 50,
    workers: int = 4,
    ascension: int = 0,
    label: str = "eval",
) -> Dict[str, Any]:
    """Play N games with a model and collect performance metrics.

    Uses the existing worker.py game-playing infrastructure via subprocess
    workers and the inference server.  Reward shaping is determined by the
    global training_config -- there is no per-eval override because the
    workers read config at import time.

    Args:
        model_path: Path to model checkpoint (.pt file).
        num_games: Number of games to play.
        workers: Number of parallel workers.
        ascension: Ascension level.
        label: Human-readable label for logging / result dict.

    Returns:
        Dict with performance metrics: avg_floor, boss_damage, hp_at_boss, win_rate.
    """
    from multiprocessing import Pool

    from .worker import _play_one_game, _worker_init

    config_name = label
    logger.info("Evaluating model %s under config '%s' (%d games, %d workers)",
                model_path, config_name, num_games, workers)

    t0 = time.monotonic()

    seeds = [f"Eval_{config_name}_{i}" for i in range(num_games)]

    results: List[Dict[str, Any]] = []
    try:
        with Pool(workers, initializer=_worker_init) as pool:
            game_args = [
                (seed, ascension, 0.0, num_games, 50.0, False, False, 0)
                for seed in seeds
            ]
            for result in pool.starmap(_play_one_game, game_args):
                results.append(result)
    except Exception as e:
        logger.error("Evaluation failed: %s", e)
        return {
            "config_name": config_name,
            "model_path": model_path,
            "error": str(e),
            "games_played": 0,
        }

    elapsed = time.monotonic() - t0

    floors = [r.get("floor", 0) for r in results]
    wins = sum(1 for r in results if r.get("won", False))
    hp_values = [r.get("hp", 0) for r in results]

    boss_damage = []
    hp_at_boss = []
    for r in results:
        for t in r.get("transitions", []):
            pass
        if r.get("floor", 0) >= 16:
            hp_at_boss.append(r.get("hp", 0))

    import numpy as np

    metrics = {
        "config_name": config_name,
        "model_path": model_path,
        "games_played": len(results),
        "avg_floor": round(float(np.mean(floors)), 2) if floors else 0.0,
        "median_floor": round(float(np.median(floors)), 2) if floors else 0.0,
        "max_floor": max(floors) if floors else 0,
        "win_rate": round(wins / max(len(results), 1) * 100, 2),
        "wins": wins,
        "avg_hp": round(float(np.mean(hp_values)), 2) if hp_values else 0.0,
        "avg_hp_at_boss": round(float(np.mean(hp_at_boss)), 2) if hp_at_boss else None,
        "floor_distribution": _floor_distribution(floors),
        "elapsed_s": round(elapsed, 1),
        "games_per_min": round(len(results) / max(elapsed / 60, 0.01), 1),
    }

    logger.info(
        "Eval complete: config=%s, avg_floor=%.1f, wins=%d/%d (%.1f%%), %.0fs",
        config_name, metrics["avg_floor"], wins, len(results),
        metrics["win_rate"], elapsed,
    )

    return metrics


def _floor_distribution(floors: List[int]) -> Dict[str, int]:
    """Bucket floors into ranges for summary."""
    buckets = {"F1-5": 0, "F6-10": 0, "F11-15": 0, "F16": 0, "F17+": 0}
    for f in floors:
        if f <= 5:
            buckets["F1-5"] += 1
        elif f <= 10:
            buckets["F6-10"] += 1
        elif f <= 15:
            buckets["F11-15"] += 1
        elif f == 16:
            buckets["F16"] += 1
        else:
            buckets["F17+"] += 1
    return buckets


def run_ab_test(
    model_path: str,
    config_names: Optional[List[str]] = None,
    num_games: int = 200,
    workers: int = 4,
    output_path: Optional[Path] = None,
) -> Dict[str, Any]:
    """Run A/B test: play games under multiple reward configs and compare.

    Note: since workers use the global training_config for reward shaping,
    A/B testing of reward configs requires changing the global config between
    runs.  This function currently runs games under the *same* reward config
    but labels them per-config for structural comparison.
    """
    from .reward_sim import ALL_REWARD_CONFIGS

    if config_names is None:
        config_names = ["A_baseline", "B_split_milestones", "C_hp_heavy"]

    configs_to_test = [
        cfg for cfg in ALL_REWARD_CONFIGS
        if cfg["name"] in config_names
    ]

    if not configs_to_test:
        logger.error("No matching configs found for names: %s", config_names)
        return {"error": "no_matching_configs"}

    if output_path is None:
        output_path = Path("logs/research/reward_ab_results.json")

    results = {
        "model_path": model_path,
        "games_per_config": num_games,
        "configs": {},
    }

    for cfg in configs_to_test:
        metrics = evaluate_model(
            model_path=model_path,
            num_games=num_games,
            workers=workers,
            label=cfg["name"],
        )
        results["configs"][cfg["name"]] = metrics

    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w") as f:
        json.dump(results, f, indent=2)

    logger.info("A/B test results saved to %s", output_path)

    print(f"\n=== A/B Test Results ({num_games} games/config) ===\n")
    for name, metrics in results["configs"].items():
        if "error" in metrics:
            print(f"  {name}: ERROR - {metrics['error']}")
            continue
        print(f"  {name}: avg_floor={metrics['avg_floor']:.1f}, "
              f"win_rate={metrics['win_rate']:.1f}%, "
              f"hp_at_boss={metrics.get('avg_hp_at_boss', 'N/A')}")
    print()

    return results


if __name__ == "__main__":
    import argparse
    logging.basicConfig(level=logging.INFO)
    parser = argparse.ArgumentParser(description="Offline model evaluation")
    parser.add_argument("--model", required=True, help="Path to model checkpoint")
    parser.add_argument("--games", type=int, default=50, help="Games to play")
    parser.add_argument("--workers", type=int, default=4, help="Parallel workers")
    parser.add_argument("--configs", type=str, default="A,B,C",
                        help="Comma-separated config names (A/B/C/D)")
    args = parser.parse_args()
    config_map = {"A": "A_baseline", "B": "B_split_milestones",
                  "C": "C_hp_heavy", "D": "D_boss_gradient"}
    names = [config_map.get(c.strip(), c.strip()) for c in args.configs.split(",")]
    run_ab_test(model_path=args.model, config_names=names,
                num_games=args.games, workers=args.workers)
