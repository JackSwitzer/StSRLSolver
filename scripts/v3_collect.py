"""V3 data collection: pure sim, no training. Builds trajectory dataset.

Runs BC-pretrained model for ~20h, saves ALL trajectories (not just best).
Goal: massive dataset for weekend training of combat net + strategic model.

Data saved:
  logs/v3_collect/all_trajectories/   — every game's transitions
  logs/v3_collect/best_trajectories/  — floor 10+ games (for BC/distillation)
  logs/v3_collect/combat_data/        — per-combat position + outcome pairs
  logs/v3_collect/status.json         — live monitoring

Usage: nohup uv run python scripts/v3_collect.py > logs/v3_collect_stdout.log 2>&1 &
"""
import json
import logging
import multiprocessing as mp
import signal
import subprocess
import sys
import time
from collections import deque
from datetime import datetime
from pathlib import Path

import numpy as np

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(name)s] %(message)s",
    handlers=[
        logging.StreamHandler(sys.stdout),
        logging.FileHandler("logs/v3_collect.log"),
    ],
)
logger = logging.getLogger("v3_collect")


def main():
    # Prevent Mac sleep
    caffeinate = subprocess.Popen(["caffeinate", "-dims"], stdout=subprocess.DEVNULL)
    logger.info("caffeinate PID=%d", caffeinate.pid)

    shutdown = False

    def on_signal(signum, frame):
        nonlocal shutdown
        shutdown = True
        logger.info("Shutdown requested")

    signal.signal(signal.SIGINT, on_signal)
    signal.signal(signal.SIGTERM, on_signal)

    from packages.training.inference_server import InferenceServer
    from packages.training.seed_pool import SeedPool
    from packages.training.strategic_net import StrategicNet, _get_device
    from packages.training.training_config import MODEL_ACTION_DIM, MODEL_HIDDEN_DIM, MODEL_NUM_BLOCKS
    from packages.training.worker import _play_one_game, _worker_init

    # Setup dirs
    run_dir = Path("logs/v3_collect")
    all_traj_dir = run_dir / "all_trajectories"
    best_traj_dir = run_dir / "best_trajectories"
    combat_dir = run_dir / "combat_data"
    for d in [run_dir, all_traj_dir, best_traj_dir, combat_dir]:
        d.mkdir(parents=True, exist_ok=True)

    # Symlink for dashboard
    active_link = Path("logs/active")
    if active_link.is_symlink():
        active_link.unlink()
    active_link.symlink_to(run_dir.resolve())
    logger.info("logs/active -> %s", run_dir)

    # Load BC-pretrained model (best checkpoint)
    device = _get_device()
    model = StrategicNet(
        input_dim=480, hidden_dim=MODEL_HIDDEN_DIM,
        action_dim=MODEL_ACTION_DIM, num_blocks=MODEL_NUM_BLOCKS,
    ).to(device)

    ckpt_path = Path("logs/strategic_checkpoints/best_strategic_floor9.4.pt")
    if not ckpt_path.exists():
        ckpt_path = Path("logs/strategic_checkpoints/latest_strategic.pt")
    if ckpt_path.exists():
        model.load(ckpt_path)
        logger.info("Loaded checkpoint: %s", ckpt_path.name)
    else:
        logger.warning("No checkpoint found — using random model")

    # BC pretrain if no checkpoint
    if not ckpt_path.exists():
        from packages.training.strategic_trainer import StrategicTrainer
        trainer = StrategicTrainer(model=model, lr=3e-5, batch_size=256)
        # Consolidate trajectories
        import shutil
        for src_dir in Path("logs").rglob("best_trajectories"):
            if src_dir == best_traj_dir:
                continue
            for f in src_dir.glob("traj_F*.npz"):
                dest = best_traj_dir / f.name
                if not dest.exists():
                    shutil.copy2(f, dest)
        bc_metrics = trainer.bc_pretrain(best_traj_dir, epochs=3, max_transitions=48000)
        logger.info("BC pretrain: %s", bc_metrics)
    else:
        # Still consolidate trajectories for reference
        import shutil
        copied = 0
        for src_dir in Path("logs").rglob("best_trajectories"):
            if src_dir == best_traj_dir:
                continue
            for f in src_dir.glob("traj_F*.npz"):
                dest = best_traj_dir / f.name
                if not dest.exists():
                    shutil.copy2(f, dest)
                    copied += 1
        logger.info("Consolidated %d existing trajectories", copied)

    param_count = sum(p.numel() for p in model.parameters())
    logger.info("Model: %d params on %s", param_count, device)

    # Inference server
    N_WORKERS = 10
    server = InferenceServer(
        n_workers=N_WORKERS, max_batch_size=32, batch_timeout_ms=15.0,
    )
    server.sync_strategic_from_pytorch(model, version=0)
    server.start()
    logger.info("Inference server started")

    # Worker pool
    ctx = mp.get_context("spawn")
    pool = ctx.Pool(
        processes=N_WORKERS,
        initializer=_worker_init,
        initargs=(server.request_q, server.response_qs, server.slot_q, None),
    )
    logger.info("Pool: %d workers", N_WORKERS)

    # Collection loop
    seed_pool = SeedPool()
    total_games = 0
    total_transitions = 0
    total_combat_positions = 0
    recent_floors = deque(maxlen=200)
    peak_floor = 0
    wins = 0
    start_time = time.monotonic()
    batch_size = 32  # Games per batch (2x workers for pipeline overlap)

    SAVE_FLOOR_THRESHOLD = 1  # Save ALL games (every transition is signal)
    BEST_FLOOR_THRESHOLD = 8  # Save to best_trajectories from floor 8+

    logger.info("=" * 60)
    logger.info("DATA COLLECTION: %d workers, saving floor %d+ trajectories", N_WORKERS, SAVE_FLOOR_THRESHOLD)
    logger.info("=" * 60)

    while not shutdown:
        # Submit batch
        seeds = [seed_pool.get_seed() for _ in range(batch_size)]
        async_results = [
            pool.apply_async(
                _play_one_game,
                (seed, 0, 0.8, total_games, 50.0, False, False, 0),
            )
            for seed in seeds
        ]

        # Collect results
        for ar, seed in zip(async_results, seeds):
            if shutdown:
                break
            try:
                result = ar.get(timeout=300)
            except Exception as e:
                logger.warning("Game %s failed: %s", seed, e)
                continue

            total_games += 1
            floor = result["floor"]
            recent_floors.append(floor)
            peak_floor = max(peak_floor, floor)
            if result["won"]:
                wins += 1

            transitions = result.get("transitions", [])
            total_transitions += len(transitions)

            # Save trajectories from decent games
            if floor >= SAVE_FLOOR_THRESHOLD and transitions:
                _save_trajectory(all_traj_dir, seed, floor, transitions)

            if floor >= BEST_FLOOR_THRESHOLD and transitions:
                _save_trajectory(best_traj_dir, seed, floor, transitions)

            # Save combat data (every game — for CombatNet training)
            combats = result.get("combats", [])
            for combat in combats:
                total_combat_positions += 1
                _save_combat_data(combat_dir, seed, floor, combat, result["won"])

        # Status update
        elapsed = time.monotonic() - start_time
        gpm = total_games / max(elapsed / 60, 0.01)
        avg_floor = sum(recent_floors) / max(len(recent_floors), 1)

        if total_games % batch_size == 0:
            logger.info(
                "Games %d | Floor %.1f | Peak %d | Wins %d | Trans %d | Combat %d | %.0f g/min",
                total_games, avg_floor, peak_floor, wins,
                total_transitions, total_combat_positions, gpm,
            )

        # Write status.json for dashboard
        status = {
            "timestamp": datetime.now().isoformat(),
            "elapsed_hours": round(elapsed / 3600, 2),
            "total_games": total_games,
            "total_wins": wins,
            "win_rate_100": round(sum(1 for f in recent_floors if f >= 55) / max(len(recent_floors), 1) * 100, 1),
            "avg_floor_100": round(avg_floor, 1),
            "games_per_min": round(gpm, 1),
            "peak_floor": peak_floor,
            "current_sweep": 0,
            "total_sweeps": 1,
            "headless": False,
            "construction_failures": 0,
            "gpu_percent": None,
            "sweep_phase": "collecting",
            "config_name": "v3_data_collection",
            "sweep_games": total_games,
            "total_transitions": total_transitions,
            "total_combat_positions": total_combat_positions,
            "all_trajectories": len(list(all_traj_dir.glob("traj_F*.npz"))),
            "best_trajectories": len(list(best_traj_dir.glob("traj_F*.npz"))),
        }
        try:
            (run_dir / "status.json").write_text(json.dumps(status, indent=2))
        except Exception:
            pass

    # Cleanup
    logger.info("Shutting down...")
    pool.terminate()
    pool.join()
    server.stop()
    caffeinate.terminate()

    elapsed = time.monotonic() - start_time
    logger.info("=" * 60)
    logger.info("COLLECTION COMPLETE")
    logger.info("Games: %d, Transitions: %d, Combat positions: %d", total_games, total_transitions, total_combat_positions)
    logger.info("Avg floor: %.1f, Peak: %d, Wins: %d", sum(recent_floors) / max(len(recent_floors), 1), peak_floor, wins)
    logger.info("Duration: %.1f hours, %.0f games/min", elapsed / 3600, total_games / max(elapsed / 60, 0.01))
    logger.info("All trajectories: %d files", len(list(all_traj_dir.glob("traj_F*.npz"))))
    logger.info("Best trajectories: %d files", len(list(best_traj_dir.glob("traj_F*.npz"))))
    logger.info("Combat data: %d files", len(list(combat_dir.glob("combat_*.npz"))))
    logger.info("=" * 60)


def _save_trajectory(traj_dir: Path, seed: str, floor: int, transitions: list):
    """Save a game's transitions as .npz for offline training."""
    fname = f"traj_F{floor:02d}_{seed}.npz"
    path = traj_dir / fname
    if path.exists():
        return  # Don't overwrite

    obs = np.stack([t["obs"] for t in transitions])
    masks = np.stack([t["action_mask"] for t in transitions])
    actions = np.array([t["action"] for t in transitions], dtype=np.int32)
    rewards = np.array([t["reward"] for t in transitions], dtype=np.float32)
    dones = np.array([t["done"] for t in transitions], dtype=bool)
    values = np.array([t["value"] for t in transitions], dtype=np.float32)
    log_probs = np.array([t["log_prob"] for t in transitions], dtype=np.float32)
    final_floors = np.array([t["final_floor"] for t in transitions], dtype=np.float32)
    cleared_act1 = np.array([t["cleared_act1"] for t in transitions], dtype=np.float32)

    np.savez_compressed(
        path, obs=obs, masks=masks, actions=actions, rewards=rewards,
        dones=dones, values=values, log_probs=log_probs,
        final_floors=final_floors, cleared_act1=cleared_act1,
    )


_combat_counter = 0

def _save_combat_data(combat_dir: Path, seed: str, game_floor: int, combat: dict, game_won: bool):
    """Save per-combat summary for CombatNet training.

    Each file: one fight's stats + outcome (did we survive this fight?).
    Uses global counter for unique filenames — never skips duplicates.
    """
    global _combat_counter
    _combat_counter += 1

    floor = combat.get("floor", 0)
    room_type = combat.get("room_type", "monster")
    hp_lost = combat.get("hp_lost", 0)
    boss_max_hp = combat.get("boss_max_hp", 0)
    boss_dmg = combat.get("boss_dmg_dealt", 0)

    # Outcome: survived if game continued past this floor
    survived = game_floor > floor

    # Save full 298-dim combat state vector as .npz for CombatNet training
    combat_state_vec = combat.get("combat_state_vector")
    if combat_state_vec is not None:
        npz_name = f"combat_{_combat_counter:06d}.npz"
        npz_path = combat_dir / npz_name
        try:
            np.savez_compressed(
                npz_path,
                combat_obs=combat_state_vec,
                won=np.array(survived, dtype=bool),
            )
        except Exception:
            pass


if __name__ == "__main__":
    main()
