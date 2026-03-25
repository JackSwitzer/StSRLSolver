"""V3 1-hour test: BC pretrain on 18k transitions, then PPO collect/train loop.

Usage: uv run python scripts/v3_1h_test.py
"""
import logging
import sys
import time

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(name)s] %(message)s",
    handlers=[
        logging.StreamHandler(sys.stdout),
        logging.FileHandler("logs/v3_1h_test.log"),
    ],
)
logger = logging.getLogger("v3_1h")


def main():
    from pathlib import Path

    from packages.training.training_runner import OvernightRunner

    # V3 PPO config: BC warmup + boss HP progress + adaptive budgets
    config = {
        "run_dir": "logs/v3_1h_test",
        "max_games": 50000,
        "workers": 8,
        "ascension": 0,
        "temperature": 0.9,
        "max_hours_per_config": 1.0,
        "sweep_configs": [
            {
                "name": "v3_ppo_bc_warmup",
                "lr": 3e-5,
                "lr_schedule": "cosine_warm_restarts",
                "lr_T_0": 5000,
                "batch_size": 256,
                "entropy_coeff": 0.05,
                "temperature": 0.9,
                "turn_solver_ms": 50.0,
                "collect_games": 200,     # Smaller batches for faster iteration
                "train_steps": 20,
                "max_hours": 1.0,
                "algorithm": "ppo",
                "bc_warmup": True,        # Force BC pretrain on 18k transitions
                "boss_hp_progress": True,
            },
        ],
    }

    logger.info("=" * 60)
    logger.info("V3 1-HOUR TEST: BC warmup + PPO + adaptive budgets")
    logger.info("=" * 60)
    logger.info("Config: %s", config["sweep_configs"][0]["name"])
    logger.info("Workers: %d, LR: %.1e, Collect: %d, Train steps: %d",
                config["workers"],
                config["sweep_configs"][0]["lr"],
                config["sweep_configs"][0]["collect_games"],
                config["sweep_configs"][0]["train_steps"])

    # Consolidate trajectories from all runs into the test run dir
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
    logger.info("Consolidated %d trajectory files into %s", copied, traj_dir)

    # Run
    runner = OvernightRunner(config)
    try:
        runner.run()
    except KeyboardInterrupt:
        logger.info("Interrupted — shutting down gracefully")
    except Exception as e:
        logger.error("Fatal: %s", e, exc_info=True)

    logger.info("V3 1h test complete")


if __name__ == "__main__":
    main()
