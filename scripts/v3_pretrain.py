"""V3 deep pretrain: BC 100ep + BC 10ep comparison + CombatNet training.

Standalone -- no training_runner dependency. Runs ~1-2h total.

Phase A: BC 100 epochs on best trajectory data -> bc_100ep_v3.pt
Phase B: BC 10 epochs (comparison) -> bc_10ep_v3.pt
Phase C: CombatNet training on combat state vectors
Phase D: Eval both checkpoints (50 games each)

Usage: uv run python scripts/v3_pretrain.py
"""

from __future__ import annotations

import json
import logging
import multiprocessing as mp
import shutil
import signal
import subprocess
import sys
import time
from collections import deque
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

import numpy as np

# ---------------------------------------------------------------------------
# Logging
# ---------------------------------------------------------------------------

LOG_DIR = Path("logs/v3_pretrain")
LOG_DIR.mkdir(parents=True, exist_ok=True)

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(name)s] %(levelname)s %(message)s",
    handlers=[
        logging.StreamHandler(sys.stdout),
        logging.FileHandler(LOG_DIR / "pretrain.log"),
    ],
)
logger = logging.getLogger("v3_pretrain")

N_WORKERS = 10
GAME_TIMEOUT_S = 180  # 3 min per game for eval


# ---------------------------------------------------------------------------
# Phase A/B: BC pretrain
# ---------------------------------------------------------------------------


def consolidate_trajectories(dest_dir: Path) -> int:
    """Copy all best_trajectories .npz files into dest_dir. Returns count copied."""
    dest_dir.mkdir(parents=True, exist_ok=True)
    copied = 0
    for src in Path("logs").rglob("best_trajectories"):
        if src.resolve() == dest_dir.resolve():
            continue
        for f in src.glob("traj_F*.npz"):
            dest = dest_dir / f.name
            if not dest.exists():
                shutil.copy2(f, dest)
                copied += 1
    # Also pull from all_trajectories dirs (floor 10+ are useful)
    for src in Path("logs").rglob("all_trajectories"):
        for f in src.glob("traj_F1*.npz"):  # F10+
            dest = dest_dir / f.name
            if not dest.exists():
                shutil.copy2(f, dest)
                copied += 1
    return copied


def run_bc_pretrain(epochs: int, label: str) -> Tuple[Any, Dict[str, float], Path]:
    """Run BC pretrain for given epochs, return (model, metrics, ckpt_path)."""
    import torch

    from packages.training.strategic_net import StrategicNet, _get_device
    from packages.training.strategic_trainer import StrategicTrainer
    from packages.training.training_config import (
        MODEL_ACTION_DIM,
        MODEL_HIDDEN_DIM,
        MODEL_NUM_BLOCKS,
    )

    device = _get_device()
    logger.info("[%s] Creating fresh StrategicNet on %s", label, device)

    model = StrategicNet(
        input_dim=480,
        hidden_dim=MODEL_HIDDEN_DIM,
        action_dim=MODEL_ACTION_DIM,
        num_blocks=MODEL_NUM_BLOCKS,
    ).to(device)
    logger.info("[%s] Model: %d params", label, model.param_count())

    # Consolidate trajectory data
    traj_dir = Path("logs/pretrain_data")
    copied = consolidate_trajectories(traj_dir)
    n_files = len(list(traj_dir.glob("traj_F*.npz")))
    logger.info("[%s] Trajectory data: %d files (%d newly copied)", label, n_files, copied)

    if n_files == 0:
        logger.error("[%s] No trajectory data found -- cannot pretrain", label)
        ckpt_path = Path(f"logs/strategic_checkpoints/bc_{epochs}ep_v3.pt")
        model.save(ckpt_path)
        return model, {"bc_loss": 0, "bc_accuracy": 0, "bc_transitions": 0}, ckpt_path

    # Create trainer with low LR for BC
    trainer = StrategicTrainer(model=model, lr=3e-5, batch_size=256)

    # BC pretrain
    t0 = time.monotonic()
    bc_metrics = trainer.bc_pretrain(traj_dir, epochs=epochs, max_transitions=48000)
    bc_elapsed = time.monotonic() - t0
    logger.info(
        "[%s] BC done in %.1f min: loss=%.4f, acc=%.1f%%, transitions=%d",
        label,
        bc_elapsed / 60,
        bc_metrics.get("bc_loss", 0),
        bc_metrics.get("bc_accuracy", 0),
        bc_metrics.get("bc_transitions", 0),
    )

    # Calibrate value head (frozen policy)
    cal_epochs = 10 if epochs >= 50 else 5
    t0 = time.monotonic()
    cal_metrics = trainer.calibrate_value_head(trajectory_dir=traj_dir, epochs=cal_epochs)
    cal_elapsed = time.monotonic() - t0
    logger.info(
        "[%s] Calibration done in %.1f min: loss=%.4f, samples=%d",
        label,
        cal_elapsed / 60,
        cal_metrics.get("calibration_loss", 0),
        cal_metrics.get("calibration_samples", 0),
    )

    # Save checkpoint
    ckpt_path = Path(f"logs/strategic_checkpoints/bc_{epochs}ep_v3.pt")
    ckpt_path.parent.mkdir(parents=True, exist_ok=True)
    model.save(ckpt_path)
    logger.info("[%s] Saved: %s", label, ckpt_path)

    combined_metrics = {**bc_metrics, **cal_metrics}
    return model, combined_metrics, ckpt_path


# ---------------------------------------------------------------------------
# Phase C: CombatNet training
# ---------------------------------------------------------------------------


def train_combat_model() -> Dict[str, float]:
    """Train CombatNet on collected combat state vectors."""
    from packages.training.combat_net import CombatNet, train_combat_net
    from packages.training.strategic_net import _get_device

    device = _get_device()

    # Gather combat .npz files from all collection runs
    combat_dirs = list(Path("logs").rglob("combat_data"))
    games_data: List[Dict[str, Any]] = []

    for combat_dir in combat_dirs:
        for f in combat_dir.glob("combat_*.npz"):
            try:
                data = np.load(f)
                obs = data["combat_obs"]
                won = bool(data["won"])
                games_data.append({"combat_obs": obs, "won": won})
            except Exception as e:
                logger.warning("Failed to load %s: %s", f.name, e)
                continue

    if len(games_data) < 50:
        logger.warning(
            "Only %d combat positions found -- CombatNet needs more data. "
            "Run v3_collect.py first to gather combat states.",
            len(games_data),
        )
        return {"loss": 0.0, "accuracy": 0.0, "samples": len(games_data), "skipped": True}

    logger.info("Training CombatNet on %d combat positions", len(games_data))

    t0 = time.monotonic()
    model, metrics = train_combat_net(games_data, epochs=50, device=device)
    elapsed = time.monotonic() - t0
    logger.info(
        "CombatNet trained in %.1f min: loss=%.4f, acc=%.1f%%",
        elapsed / 60,
        metrics.get("loss", 0),
        metrics.get("accuracy", 0),
    )

    # Save where worker auto-loads
    save_path = Path("logs/active/combat_net.pt")
    save_path.parent.mkdir(parents=True, exist_ok=True)
    model.save(save_path)
    logger.info("CombatNet saved: %s", save_path)

    # Also save a versioned copy
    versioned_path = LOG_DIR / "combat_net_v3.pt"
    model.save(versioned_path)

    return metrics


# ---------------------------------------------------------------------------
# Phase D: Eval
# ---------------------------------------------------------------------------


def eval_model(ckpt_path: Path, n_games: int = 50, label: str = "") -> Tuple[float, int]:
    """Play n_games with the model, return (avg_floor, peak_floor).

    Spins up a fresh inference server + worker pool, plays games, then
    tears everything down cleanly (including shared memory).
    """
    from packages.training.inference_server import InferenceServer
    from packages.training.seed_pool import SeedPool
    from packages.training.strategic_net import StrategicNet
    from packages.training.training_config import SOLVER_BUDGETS, MCTS_COMBAT_ENABLED
    from packages.training.worker import _play_one_game, _worker_init

    # Config-driven solver budget
    _default_solver_ms = SOLVER_BUDGETS["monster"][0]

    logger.info("[%s] Loading checkpoint: %s", label, ckpt_path)
    model = StrategicNet.load(ckpt_path)
    logger.info("[%s] Model loaded: %d params", label, model.param_count())

    # Start inference server
    server = InferenceServer(
        n_workers=N_WORKERS,
        max_batch_size=32,
        batch_timeout_ms=15.0,
    )
    server.sync_strategic_from_pytorch(model, version=0)
    server.start()
    logger.info("[%s] Inference server started", label)

    # Create worker pool
    ctx = mp.get_context("spawn")
    pool = ctx.Pool(
        processes=N_WORKERS,
        initializer=_worker_init,
        initargs=(server.request_q, server.response_qs, server.slot_q, server.shm_info),
    )
    logger.info("[%s] Worker pool started: %d workers", label, N_WORKERS)

    # Play games
    seed_pool = SeedPool()
    floors: List[int] = []
    wins = 0
    failures = 0

    t0 = time.monotonic()

    # Submit all games at once for maximum parallelism
    async_results = []
    for i in range(n_games):
        seed = seed_pool.get_seed()
        ar = pool.apply_async(
            _play_one_game,
            (seed, 0, 0.8, 0, _default_solver_ms, False, MCTS_COMBAT_ENABLED, 0),
        )
        async_results.append((i, seed, ar))

    # Collect results
    for i, seed, ar in async_results:
        try:
            result = ar.get(timeout=GAME_TIMEOUT_S)
            floor = result["floor"]
            floors.append(floor)
            if result["won"]:
                wins += 1
            if (i + 1) % 10 == 0:
                avg_so_far = sum(floors) / len(floors)
                logger.info(
                    "[%s] Eval progress: %d/%d games, avg_floor=%.1f",
                    label, i + 1, n_games, avg_so_far,
                )
        except Exception as e:
            logger.warning("[%s] Game %d (seed=%s) failed: %s", label, i, seed, e)
            failures += 1

    eval_elapsed = time.monotonic() - t0

    # Cleanup: terminate pool first, then stop server
    pool.terminate()
    pool.join()
    server.stop()
    logger.info("[%s] Cleanup complete", label)

    # Compute stats
    if not floors:
        logger.warning("[%s] All games failed", label)
        return 0.0, 0

    avg_floor = sum(floors) / len(floors)
    peak = max(floors)
    median = sorted(floors)[len(floors) // 2]
    win_rate = wins / len(floors) * 100

    logger.info(
        "[%s] Eval %d games (%.1f min): avg=%.1f, median=%d, peak=%d, wins=%d (%.1f%%), failures=%d",
        label,
        len(floors),
        eval_elapsed / 60,
        avg_floor,
        median,
        peak,
        wins,
        win_rate,
        failures,
    )

    return avg_floor, peak


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main():
    """Run all pretrain phases: BC 100ep, BC 10ep, CombatNet, eval both."""
    # Prevent Mac from sleeping
    caffeinate = subprocess.Popen(
        ["caffeinate", "-dims"],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    logger.info("caffeinate PID=%d", caffeinate.pid)

    # Graceful shutdown
    shutdown = False

    def on_signal(signum, frame):
        nonlocal shutdown
        shutdown = True
        logger.info("Shutdown requested (signal %d)", signum)

    signal.signal(signal.SIGINT, on_signal)
    signal.signal(signal.SIGTERM, on_signal)

    overall_t0 = time.monotonic()

    logger.info("=" * 60)
    logger.info("V3 DEEP PRETRAIN")
    logger.info("=" * 60)

    # ------------------------------------------------------------------
    # Phase A: BC 100 epochs
    # ------------------------------------------------------------------
    logger.info("")
    logger.info("=" * 60)
    logger.info("PHASE A: BC 100 epochs")
    logger.info("=" * 60)

    model_100, metrics_100, ckpt_100 = run_bc_pretrain(100, "100ep")

    if shutdown:
        logger.info("Shutdown requested -- skipping remaining phases")
        caffeinate.terminate()
        return

    # ------------------------------------------------------------------
    # Phase B: BC 10 epochs (comparison baseline)
    # ------------------------------------------------------------------
    logger.info("")
    logger.info("=" * 60)
    logger.info("PHASE B: BC 10 epochs (comparison)")
    logger.info("=" * 60)

    model_10, metrics_10, ckpt_10 = run_bc_pretrain(10, "10ep")

    if shutdown:
        logger.info("Shutdown requested -- skipping remaining phases")
        caffeinate.terminate()
        return

    # ------------------------------------------------------------------
    # Phase C: CombatNet training
    # ------------------------------------------------------------------
    logger.info("")
    logger.info("=" * 60)
    logger.info("PHASE C: CombatNet training")
    logger.info("=" * 60)

    combat_metrics = train_combat_model()

    if shutdown:
        logger.info("Shutdown requested -- skipping eval phase")
        caffeinate.terminate()
        return

    # ------------------------------------------------------------------
    # Phase D: Eval both checkpoints
    # ------------------------------------------------------------------
    logger.info("")
    logger.info("=" * 60)
    logger.info("PHASE D: Evaluation (50 games each)")
    logger.info("=" * 60)

    floor_100, peak_100 = eval_model(ckpt_100, n_games=50, label="100ep")

    if shutdown:
        logger.info("Shutdown requested -- skipping 10ep eval")
        caffeinate.terminate()
        return

    floor_10, peak_10 = eval_model(ckpt_10, n_games=50, label="10ep")

    # ------------------------------------------------------------------
    # Results
    # ------------------------------------------------------------------
    total_elapsed = time.monotonic() - overall_t0

    logger.info("")
    logger.info("=" * 60)
    logger.info("RESULTS")
    logger.info("=" * 60)
    logger.info(
        "  100ep: bc_acc=%.1f%%, bc_loss=%.4f, avg_floor=%.1f, peak=%d",
        metrics_100.get("bc_accuracy", 0),
        metrics_100.get("bc_loss", 0),
        floor_100,
        peak_100,
    )
    logger.info(
        "  10ep:  bc_acc=%.1f%%, bc_loss=%.4f, avg_floor=%.1f, peak=%d",
        metrics_10.get("bc_accuracy", 0),
        metrics_10.get("bc_loss", 0),
        floor_10,
        peak_10,
    )
    logger.info(
        "  CombatNet: loss=%.4f, acc=%.1f%%, samples=%d",
        combat_metrics.get("loss", 0),
        combat_metrics.get("accuracy", 0),
        combat_metrics.get("samples", 0),
    )
    logger.info("  Total time: %.1f hours", total_elapsed / 3600)

    # Pick winner
    winner = ckpt_100 if floor_100 >= floor_10 else ckpt_10
    winner_label = "100ep" if floor_100 >= floor_10 else "10ep"
    logger.info("")
    logger.info("WINNER: %s (%s)", winner_label, winner.name)

    # Copy winner as the checkpoint for downstream training
    winner_dest = Path("logs/strategic_checkpoints/bc_winner_v3.pt")
    shutil.copy2(winner, winner_dest)
    logger.info("Winner copied to: %s", winner_dest)

    # Also save as latest_strategic.pt for easy pickup
    latest_dest = Path("logs/strategic_checkpoints/latest_strategic.pt")
    shutil.copy2(winner, latest_dest)
    logger.info("Winner copied to: %s", latest_dest)

    # Write summary JSON for dashboard / downstream scripts
    summary = {
        "winner": winner_label,
        "winner_path": str(winner_dest),
        "100ep": {
            "bc_accuracy": metrics_100.get("bc_accuracy", 0),
            "bc_loss": metrics_100.get("bc_loss", 0),
            "bc_transitions": metrics_100.get("bc_transitions", 0),
            "calibration_loss": metrics_100.get("calibration_loss", 0),
            "avg_floor": floor_100,
            "peak_floor": peak_100,
        },
        "10ep": {
            "bc_accuracy": metrics_10.get("bc_accuracy", 0),
            "bc_loss": metrics_10.get("bc_loss", 0),
            "bc_transitions": metrics_10.get("bc_transitions", 0),
            "calibration_loss": metrics_10.get("calibration_loss", 0),
            "avg_floor": floor_10,
            "peak_floor": peak_10,
        },
        "combat_net": {
            "loss": combat_metrics.get("loss", 0),
            "accuracy": combat_metrics.get("accuracy", 0),
            "samples": combat_metrics.get("samples", 0),
        },
        "total_hours": round(total_elapsed / 3600, 2),
    }
    summary_path = LOG_DIR / "results.json"
    summary_path.write_text(json.dumps(summary, indent=2))
    logger.info("Summary written to: %s", summary_path)

    logger.info("=" * 60)
    logger.info("PRETRAIN COMPLETE")
    logger.info("=" * 60)

    caffeinate.terminate()


if __name__ == "__main__":
    main()
