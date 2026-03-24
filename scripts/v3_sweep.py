"""V3 sweep: 4 configs queued sequentially, 2h each = ~8h total.

A) PPO high-entropy (0.10) — explore heavily after BC
B) PPO medium-entropy (0.05) — standard, see if BC holds
C) PPO no-BC — fresh model, no pretrain, pure RL
D) IQL offline — train on all 18k transitions, no collection

caffeinate keeps Mac awake. Each config auto-checkpoints.

Usage: nohup uv run python scripts/v3_sweep.py > logs/v3_sweep_stdout.log 2>&1 &
"""
import logging
import os
import signal
import subprocess
import sys

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(name)s] %(message)s",
    handlers=[
        logging.StreamHandler(sys.stdout),
        logging.FileHandler("logs/v3_sweep.log"),
    ],
)
logger = logging.getLogger("v3_sweep")


def main():
    from pathlib import Path

    caffeinate = subprocess.Popen(["caffeinate", "-dims"], stdout=subprocess.DEVNULL)
    logger.info("caffeinate PID=%d", caffeinate.pid)

    def cleanup(signum=None, frame=None):
        caffeinate.terminate()
        if signum:
            sys.exit(0)
    signal.signal(signal.SIGINT, cleanup)
    signal.signal(signal.SIGTERM, cleanup)

    from packages.training.training_runner import OvernightRunner

    config = {
        "run_dir": "logs/v3_sweep",
        "max_games": 100000,
        "workers": 10,
        "ascension": 0,
        "temperature": 1.0,
        "sweep_configs": [
            # A) PPO high-entropy — BC warmup + aggressive exploration
            {
                "name": "A_ppo_high_ent",
                "lr": 3e-5,
                "lr_schedule": "cosine_warm_restarts",
                "lr_T_0": 5000,
                "batch_size": 256,
                "entropy_coeff": 0.10,
                "temperature": 1.0,
                "turn_solver_ms": 50.0,
                "collect_games": 500,
                "train_steps": 30,
                "max_hours": 2.0,
                "algorithm": "ppo",
                "bc_warmup": True,
            },
            # B) PPO standard entropy — BC warmup, lower entropy
            {
                "name": "B_ppo_std_ent",
                "lr": 3e-5,
                "lr_schedule": "cosine_warm_restarts",
                "lr_T_0": 5000,
                "batch_size": 256,
                "entropy_coeff": 0.05,
                "temperature": 0.9,
                "turn_solver_ms": 50.0,
                "collect_games": 500,
                "train_steps": 30,
                "max_hours": 2.0,
                "algorithm": "ppo",
                "bc_warmup": True,
            },
            # C) PPO no-BC — no pretrain, pure RL from scratch
            {
                "name": "C_ppo_no_bc",
                "lr": 1e-4,
                "lr_schedule": "cosine_warm_restarts",
                "lr_T_0": 5000,
                "batch_size": 256,
                "entropy_coeff": 0.05,
                "temperature": 1.0,
                "turn_solver_ms": 50.0,
                "collect_games": 500,
                "train_steps": 30,
                "max_hours": 2.0,
                "algorithm": "ppo",
            },
            # D) IQL offline — train on all trajectory data
            {
                "name": "D_iql_offline",
                "lr": 3e-4,
                "lr_schedule": "cosine",
                "batch_size": 256,
                "max_hours": 2.0,
                "algorithm": "iql",
                "iql_expectile": 0.7,
                "iql_temperature": 3.0,
            },
        ],
    }

    logger.info("=" * 60)
    logger.info("V3 SWEEP: 4 configs x 2h = 8h total")
    for i, c in enumerate(config["sweep_configs"]):
        logger.info("  %s: %s (algo=%s, ent=%s, bc=%s)",
            chr(65+i), c["name"], c.get("algorithm","ppo"),
            c.get("entropy_coeff","n/a"), c.get("bc_warmup", False))
    logger.info("=" * 60)

    # Consolidate trajectories
    run_dir = Path(config["run_dir"])
    traj_dir = run_dir / "best_trajectories"
    traj_dir.mkdir(parents=True, exist_ok=True)
    import shutil
    copied = 0
    for src_dir in Path("logs").rglob("best_trajectories"):
        if src_dir == traj_dir:
            continue
        for f in src_dir.glob("traj_F*.npz"):
            dest = traj_dir / f.name
            if not dest.exists():
                shutil.copy2(f, dest)
                copied += 1
    logger.info("Consolidated %d trajectory files", copied)

    # Cap BC to 3 epochs
    import packages.training.strategic_trainer as st_mod
    _orig_bc = st_mod.StrategicTrainer.bc_pretrain
    def _bc_3ep(self, trajectory_dir, epochs=3, max_transitions=48000):
        return _orig_bc(self, trajectory_dir, epochs=3, max_transitions=max_transitions)
    st_mod.StrategicTrainer.bc_pretrain = _bc_3ep

    try:
        runner = OvernightRunner(config)
        runner.run()
    except KeyboardInterrupt:
        logger.info("Interrupted")
    except Exception as e:
        logger.error("Fatal: %s", e, exc_info=True)
    finally:
        cleanup()

    logger.info("V3 sweep complete")


if __name__ == "__main__":
    main()
