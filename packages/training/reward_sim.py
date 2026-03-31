"""Reward simulation: rescore existing trajectories under alternative reward configs.

Loads .npz trajectory files, recomputes returns using different reward weight
configurations, and compares distributions. No games are played -- this is
pure offline analysis of already-collected data.
"""

from __future__ import annotations

import json
import logging
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence

import numpy as np

from .data_utils import TrajectoryData, find_trajectory_files, load_trajectory_file
from .training_config import PBRS_GAMMA

logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Reward configs for A/B comparison
# ---------------------------------------------------------------------------

REWARD_CONFIG_A: Dict[str, Any] = {
    "name": "A_baseline",
    "description": "Current config as-is",
    "combat_win": 0.30,
    "elite_win": 1.50,
    "boss_win": 5.00,
    "floor_milestones": {
        3: 0.25, 6: 1.50, 10: 3.00, 13: 4.00, 15: 6.00, 16: 9.00,
        17: 15.00, 25: 9.00, 33: 15.00, 34: 24.00, 50: 24.00, 51: 36.00, 55: 50.00,
    },
    "f16_hp_bonus_base": 1.50,
    "f16_hp_bonus_per_hp": 0.05,
    "death_penalty_scale": -0.3,
    "death_floor_cutoff": 55,
    "win_reward": 10.0,
    "pbrs_hp_weight": 0.30,
    "boss_hp_progress_scale": 3.0,
}

REWARD_CONFIG_B: Dict[str, Any] = {
    "name": "B_split_milestones",
    "description": "F16 entry=+2, survival=+12, death=-3.0, PBRS hp=0.80",
    "combat_win": 0.30,
    "elite_win": 1.50,
    "boss_win": 5.00,
    "floor_milestones": {
        3: 0.25, 6: 1.50, 10: 3.00, 13: 4.00, 15: 6.00, 16: 2.00,
        17: 12.00, 25: 9.00, 33: 15.00, 34: 24.00, 50: 24.00, 51: 36.00, 55: 50.00,
    },
    "f16_hp_bonus_base": 1.50,
    "f16_hp_bonus_per_hp": 0.05,
    "death_penalty_scale": -3.0,
    "death_floor_cutoff": 55,
    "win_reward": 10.0,
    "pbrs_hp_weight": 0.80,
    "boss_hp_progress_scale": 3.0,
}

REWARD_CONFIG_C: Dict[str, Any] = {
    "name": "C_hp_heavy",
    "description": "PBRS hp_weight=1.0, death=-5.0, no floor milestones",
    "combat_win": 0.30,
    "elite_win": 1.50,
    "boss_win": 5.00,
    "floor_milestones": {},
    "f16_hp_bonus_base": 3.00,
    "f16_hp_bonus_per_hp": 0.10,
    "death_penalty_scale": -5.0,
    "death_floor_cutoff": 55,
    "win_reward": 10.0,
    "pbrs_hp_weight": 1.0,
    "boss_hp_progress_scale": 3.0,
}

REWARD_CONFIG_D: Dict[str, Any] = {
    "name": "D_boss_gradient",
    "description": "No floor milestones, only boss_hp_progress + combat wins",
    "combat_win": 0.50,
    "elite_win": 2.00,
    "boss_win": 8.00,
    "floor_milestones": {},
    "f16_hp_bonus_base": 0.0,
    "f16_hp_bonus_per_hp": 0.0,
    "death_penalty_scale": -1.0,
    "death_floor_cutoff": 55,
    "win_reward": 15.0,
    "pbrs_hp_weight": 0.30,
    "boss_hp_progress_scale": 6.0,
}

ALL_REWARD_CONFIGS = [REWARD_CONFIG_A, REWARD_CONFIG_B, REWARD_CONFIG_C, REWARD_CONFIG_D]


# ---------------------------------------------------------------------------
# Rescoring logic
# ---------------------------------------------------------------------------

def rescore_trajectory(traj: TrajectoryData, reward_config: Dict[str, Any]) -> Dict[str, Any]:
    """Recompute returns for a trajectory under a new reward config.

    Since we don't have the full game state for each timestep, we approximate
    by scaling the existing rewards based on config differences. The floor
    information comes from the trajectory metadata.

    Returns a dict with:
        total_return: sum of rescored rewards
        mean_reward: mean per-step reward
        floor: max floor reached
        floor_reward: total reward from floor milestones
        hp_at_boss: estimated HP bonus at floor 16 (if reached)
    """
    floor = traj.floor
    T = len(traj.rewards)
    if T == 0:
        return {"total_return": 0.0, "mean_reward": 0.0, "floor": floor,
                "floor_reward": 0.0, "hp_at_boss": 0.0}

    # Compute floor milestone rewards for this config
    milestones = reward_config.get("floor_milestones", {})
    floor_reward = 0.0
    for mf, mr in milestones.items():
        mf_int = int(mf)
        if floor >= mf_int:
            floor_reward += mr

    # F16 HP bonus (approximate: use a proxy from existing rewards)
    hp_at_boss = 0.0
    if floor >= 16:
        hp_base = reward_config.get("f16_hp_bonus_base", 0.0)
        hp_per = reward_config.get("f16_hp_bonus_per_hp", 0.0)
        # Estimate HP at boss: use midpoint (40 HP) as proxy since we don't
        # have exact HP from the trajectory
        estimated_hp = 40.0
        hp_at_boss = hp_base + hp_per * estimated_hp

    # Boss HP progress
    boss_hp_reward = 0.0
    boss_scale = reward_config.get("boss_hp_progress_scale", 3.0)
    if floor >= 16:
        # Approximate: if died at boss, ~50% damage dealt
        if floor == 16:
            boss_hp_reward = 0.5 * boss_scale
        elif floor >= 17:
            boss_hp_reward = 1.0 * boss_scale

    # Terminal reward
    terminal = 0.0
    won = floor >= 55
    if won:
        terminal = reward_config.get("win_reward", 10.0)
    else:
        progress = floor / reward_config.get("death_floor_cutoff", 55)
        terminal = reward_config.get("death_penalty_scale", -0.3) * (1 - progress)

    # Combat win approximation: ~1 combat per floor
    combat_wins = max(0, floor - 1)  # Rough estimate
    combat_reward = combat_wins * reward_config.get("combat_win", 0.3)

    # PBRS component (scale existing PBRS by hp_weight ratio)
    baseline_hp_weight = 0.30
    config_hp_weight = reward_config.get("pbrs_hp_weight", 0.30)
    hp_scale = config_hp_weight / max(baseline_hp_weight, 1e-8)
    # Use original rewards as base, scale the PBRS portion
    original_return = float(np.sum(traj.rewards))
    # Rough decomposition: ~60% of reward is PBRS, ~40% is events
    pbrs_portion = original_return * 0.6
    event_portion = original_return * 0.4
    rescored_pbrs = pbrs_portion * hp_scale

    total_return = (
        rescored_pbrs
        + floor_reward
        + hp_at_boss
        + boss_hp_reward
        + terminal
        + combat_reward
    )

    return {
        "total_return": round(float(total_return), 4),
        "mean_reward": round(float(total_return / T), 6),
        "floor": floor,
        "floor_reward": round(float(floor_reward), 4),
        "hp_at_boss": round(float(hp_at_boss), 4),
        "boss_hp_reward": round(float(boss_hp_reward), 4),
        "terminal": round(float(terminal), 4),
        "combat_reward": round(float(combat_reward), 4),
    }


def load_trajectories_for_sim(
    dirs: Sequence[Path],
    max_files: int = 500,
) -> List[TrajectoryData]:
    """Load trajectory files for reward simulation (no dim filtering)."""
    files = find_trajectory_files(list(dirs), min_floor=0)[:max_files]
    trajectories = []
    for f in files:
        traj = load_trajectory_file(f)
        if traj is not None:
            trajectories.append(traj)
    logger.info("Loaded %d trajectories for reward simulation", len(trajectories))
    return trajectories


def compare_configs(
    configs: Optional[List[Dict[str, Any]]] = None,
    data_dirs: Optional[Sequence[Path]] = None,
    max_files: int = 500,
) -> Dict[str, Any]:
    """Rescore all trajectories under each config, compare distributions.

    Args:
        configs: List of reward config dicts. Defaults to ALL_REWARD_CONFIGS.
        data_dirs: Directories to load trajectories from. Defaults to logs/runs/.
        max_files: Maximum trajectory files to load.

    Returns:
        Dict with per-config stats and comparison data.
    """
    if configs is None:
        configs = ALL_REWARD_CONFIGS
    if data_dirs is None:
        data_dirs = [Path("logs/runs")]

    trajectories = load_trajectories_for_sim(data_dirs, max_files=max_files)
    if not trajectories:
        logger.warning("No trajectories found for reward simulation")
        return {"error": "no_trajectories", "configs": [c["name"] for c in configs]}

    results: Dict[str, Any] = {
        "num_trajectories": len(trajectories),
        "configs": {},
    }

    for cfg in configs:
        name = cfg["name"]
        returns = []
        floors = []
        hp_at_boss_values = []
        floor_reward_curve: Dict[int, List[float]] = {}

        for traj in trajectories:
            scored = rescore_trajectory(traj, cfg)
            returns.append(scored["total_return"])
            floors.append(scored["floor"])
            if scored["floor"] >= 16:
                hp_at_boss_values.append(scored["hp_at_boss"])

            # Floor-reward curve: what total reward does each floor level get?
            fl = scored["floor"]
            if fl not in floor_reward_curve:
                floor_reward_curve[fl] = []
            floor_reward_curve[fl].append(scored["total_return"])

        returns_arr = np.array(returns)
        results["configs"][name] = {
            "description": cfg.get("description", ""),
            "mean_return": round(float(np.mean(returns_arr)), 4),
            "std_return": round(float(np.std(returns_arr)), 4),
            "median_return": round(float(np.median(returns_arr)), 4),
            "min_return": round(float(np.min(returns_arr)), 4),
            "max_return": round(float(np.max(returns_arr)), 4),
            "return_at_f16": round(float(np.mean([r for r, f in zip(returns, floors) if f >= 16])), 4) if any(f >= 16 for f in floors) else None,
            "hp_at_boss_mean": round(float(np.mean(hp_at_boss_values)), 4) if hp_at_boss_values else None,
            "hp_gradient_70_vs_10": _hp_gradient(cfg, 70, 10),
            "floor_reward_curve": {
                str(fl): round(float(np.mean(vals)), 4)
                for fl, vals in sorted(floor_reward_curve.items())
            },
            "histogram": _histogram_data(returns_arr),
        }

    return results


def _hp_gradient(config: Dict[str, Any], hp_high: float, hp_low: float) -> float:
    """Compute how much more reward for hp_high vs hp_low at F16."""
    base = config.get("f16_hp_bonus_base", 0.0)
    per_hp = config.get("f16_hp_bonus_per_hp", 0.0)
    reward_high = base + per_hp * hp_high
    reward_low = base + per_hp * hp_low
    return round(reward_high - reward_low, 4)


def _histogram_data(values: np.ndarray, bins: int = 20) -> Dict[str, Any]:
    """Create histogram data for JSON serialization."""
    counts, edges = np.histogram(values, bins=bins)
    return {
        "counts": counts.tolist(),
        "bin_edges": [round(float(e), 4) for e in edges.tolist()],
    }


def run_reward_simulation(output_path: Optional[Path] = None) -> Dict[str, Any]:
    """Run the full reward simulation and save results.

    Args:
        output_path: Where to save results JSON. Defaults to logs/research/reward_sim_results.json.

    Returns:
        The comparison results dict.
    """
    if output_path is None:
        output_path = Path("logs/research/reward_sim_results.json")

    output_path.parent.mkdir(parents=True, exist_ok=True)

    results = compare_configs()
    with open(output_path, "w") as f:
        json.dump(results, f, indent=2)

    logger.info("Reward simulation results saved to %s", output_path)

    # Print summary
    print(f"\n=== Reward Simulation Results ({results.get('num_trajectories', 0)} trajectories) ===\n")
    for name, stats in results.get("configs", {}).items():
        print(f"  {name}: mean={stats['mean_return']:.2f} +/- {stats['std_return']:.2f}")
        if stats.get("return_at_f16") is not None:
            print(f"    Return at F16: {stats['return_at_f16']:.2f}")
        grad = stats.get("hp_gradient_70_vs_10", 0)
        print(f"    HP gradient (70 vs 10 HP): +{grad:.2f}")
    print()

    return results


# ---------------------------------------------------------------------------
# CLI entry point
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    run_reward_simulation()
