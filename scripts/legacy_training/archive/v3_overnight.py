"""V3 overnight training: BC warmup (3 epochs) -> PPO with high entropy.

Fixes from 1h test:
- BC epochs 10->3 (prevents entropy collapse)
- Entropy coeff 0.05->0.10 (keeps exploration after BC)
- Abort grace period 2000 games (lets PPO stabilize after BC)
- caffeinate prevents Mac sleep

Usage: uv run python scripts/v3_overnight.py
"""
import logging
import os
import signal
import subprocess
import sys
import time

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(name)s] %(message)s",
    handlers=[
        logging.StreamHandler(sys.stdout),
        logging.FileHandler("logs/v3_overnight.log"),
    ],
)
logger = logging.getLogger("v3_overnight")


def main():
    from pathlib import Path

    # Prevent Mac sleep
    caffeinate = subprocess.Popen(["caffeinate", "-dims"], stdout=subprocess.DEVNULL)
    logger.info("caffeinate PID=%d (prevents sleep)", caffeinate.pid)

    def cleanup(signum=None, frame=None):
        caffeinate.terminate()
        logger.info("caffeinate stopped")
        if signum:
            sys.exit(0)

    signal.signal(signal.SIGINT, cleanup)
    signal.signal(signal.SIGTERM, cleanup)

    from packages.training.training_runner import OvernightRunner

    config = {
        "run_dir": "logs/v3_overnight",
        "max_games": 50000,
        "workers": 10,
        "ascension": 0,
        "temperature": 1.0,  # Higher temp for exploration
        "max_hours_per_config": 8.0,
        "sweep_configs": [
            {
                "name": "v3_ppo_overnight",
                "lr": 3e-5,
                "lr_schedule": "cosine_warm_restarts",
                "lr_T_0": 5000,
                "batch_size": 256,
                "entropy_coeff": 0.10,    # 2x higher to counteract BC confidence
                "temperature": 1.0,
                "turn_solver_ms": 50.0,
                "collect_games": 500,
                "train_steps": 30,
                "ppo_epochs": 4,
                "max_hours": 8.0,
                "algorithm": "ppo",
                "bc_warmup": True,
                "boss_hp_progress": True,
            },
        ],
    }

    logger.info("=" * 60)
    logger.info("V3 OVERNIGHT: BC(3ep) -> PPO, entropy=0.10, 10 workers")
    logger.info("=" * 60)

    # Consolidate trajectories
    run_dir = Path(config["run_dir"])
    traj_dir = run_dir / "best_trajectories"
    traj_dir.mkdir(parents=True, exist_ok=True)

    import shutil
    source_dirs = list(Path("logs").rglob("best_trajectories"))
    copied = 0
    for src_dir in source_dirs:
        if src_dir == traj_dir:
            continue
        for f in src_dir.glob("traj_F*.npz"):
            dest = traj_dir / f.name
            if not dest.exists():
                shutil.copy2(f, dest)
                copied += 1
    logger.info("Consolidated %d trajectory files", copied)

    # Monkey-patch BC epochs to 3 (not 10)
    import packages.training.strategic_trainer as st_mod
    _orig_bc = st_mod.StrategicTrainer.bc_pretrain
    def _bc_3ep(self, trajectory_dir, epochs=3, max_transitions=48000):
        return _orig_bc(self, trajectory_dir, epochs=3, max_transitions=max_transitions)
    st_mod.StrategicTrainer.bc_pretrain = _bc_3ep
    logger.info("BC epochs capped at 3")

    try:
        runner = OvernightRunner(config)
        runner.run()
    except KeyboardInterrupt:
        logger.info("Interrupted")
    except Exception as e:
        logger.error("Fatal: %s", e, exc_info=True)
    finally:
        cleanup()

    logger.info("V3 overnight complete")


if __name__ == "__main__":
    main()
