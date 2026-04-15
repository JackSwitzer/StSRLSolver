"""Behavioral cloning pretrain: imitation learning on best trajectories.

Consolidates floor 10+ trajectory data, runs BC pretrain, calibrates value head.
Uses strategic_trainer.bc_pretrain() and calibrate_value_head() — no reimplementation.

Usage:
    uv run python scripts/pretrain_bc.py [--epochs 100] [--cal-epochs 10]
"""

from __future__ import annotations

import argparse
import logging
import shutil
import sys
import time
from pathlib import Path

import numpy as np

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [pretrain_bc] %(levelname)s %(message)s",
    handlers=[logging.StreamHandler(sys.stdout)],
)
logger = logging.getLogger("pretrain_bc")


def consolidate_trajectories(dest_dir: Path, min_floor: int = 10) -> int:
    """Copy floor {min_floor}+ trajectory .npz files into dest_dir."""
    dest_dir.mkdir(parents=True, exist_ok=True)
    copied = 0
    for src_dir in Path("logs").rglob("*trajectories"):
        if src_dir.resolve() == dest_dir.resolve():
            continue
        for f in src_dir.glob("traj_F*.npz"):
            # Extract floor from filename: traj_F{NN}_...
            try:
                floor = int(f.stem.split("_")[1][1:])
            except (IndexError, ValueError):
                continue
            if floor < min_floor:
                continue
            dest = dest_dir / f.name
            if not dest.exists():
                shutil.copy2(f, dest)
                copied += 1
    return copied


def main():
    parser = argparse.ArgumentParser(description="BC pretrain on best trajectories")
    parser.add_argument("--epochs", type=int, default=100, help="BC training epochs")
    parser.add_argument("--cal-epochs", type=int, default=10, help="Value head calibration epochs")
    parser.add_argument("--min-floor", type=int, default=10, help="Minimum floor for training data")
    parser.add_argument("--max-transitions", type=int, default=48000, help="Max transitions to load")
    parser.add_argument("--lr", type=float, default=3e-5, help="Learning rate")
    args = parser.parse_args()

    from packages.training.strategic_net import StrategicNet, _get_device
    from packages.training.strategic_trainer import StrategicTrainer
    from packages.training.training_config import MODEL_ACTION_DIM, MODEL_HIDDEN_DIM, MODEL_NUM_BLOCKS

    device = _get_device()

    # Consolidate data
    traj_dir = Path("logs/pretrain_data")
    copied = consolidate_trajectories(traj_dir, min_floor=args.min_floor)
    n_files = len(list(traj_dir.glob("traj_F*.npz")))
    logger.info("Trajectory data: %d files (%d newly copied, floor %d+)", n_files, copied, args.min_floor)

    if n_files == 0:
        logger.error("No trajectory data found — cannot pretrain")
        sys.exit(1)

    # Create model + trainer
    model = StrategicNet(
        input_dim=480, hidden_dim=MODEL_HIDDEN_DIM,
        action_dim=MODEL_ACTION_DIM, num_blocks=MODEL_NUM_BLOCKS,
    ).to(device)
    logger.info("Model: %d params on %s", model.param_count(), device)

    trainer = StrategicTrainer(model=model, lr=args.lr, batch_size=256)

    # Phase 1: BC pretrain
    t0 = time.monotonic()
    metrics = trainer.bc_pretrain(traj_dir, epochs=args.epochs, max_transitions=args.max_transitions)
    logger.info("BC done (%.1f min): loss=%.4f, acc=%.1f%%, transitions=%d",
                (time.monotonic() - t0) / 60,
                metrics.get("bc_loss", 0), metrics.get("bc_accuracy", 0),
                metrics.get("bc_transitions", 0))

    # Phase 2: Value head calibration
    t0 = time.monotonic()
    cal = trainer.calibrate_value_head(trajectory_dir=traj_dir, epochs=args.cal_epochs)
    logger.info("Calibration done (%.1f min): loss=%.4f, samples=%d",
                (time.monotonic() - t0) / 60,
                cal.get("calibration_loss", 0), cal.get("calibration_samples", 0))

    # Save
    ckpt_dir = Path("logs/strategic_checkpoints")
    ckpt_dir.mkdir(parents=True, exist_ok=True)
    ckpt_path = ckpt_dir / f"bc_{args.epochs}ep_v4.pt"
    model.save(ckpt_path)
    logger.info("Saved: %s", ckpt_path)

    # Also save as latest
    latest = ckpt_dir / "latest_strategic.pt"
    shutil.copy2(ckpt_path, latest)
    logger.info("Copied to: %s", latest)


if __name__ == "__main__":
    main()
