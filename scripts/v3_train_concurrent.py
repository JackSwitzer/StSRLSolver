"""Concurrent BC training + game collection: GPU and CPU work simultaneously.

Architecture:
    Training thread (MPS GPU): continuously trains BC on growing trajectory dataset
    Collection thread (CPU + MLX): plays games, saves trajectories to shared directory

    ┌─────────────────────────────┐     ┌──────────────────────────────┐
    │ TRAINING THREAD (GPU)       │     │ COLLECTION THREAD (CPU)      │
    │                             │     │                              │
    │ while not done:             │     │ while not done:              │
    │   load latest .npz files    │     │   play 32 games (batch)      │
    │   train BC 10 epochs        │     │   save .npz to shared dir    │
    │   update shared checkpoint  │────>│   every 500 games: resync    │
    │   log metrics               │     │   log metrics                │
    └─────────────────────────────┘     └──────────────────────────────┘

Current sequential approach: GPU active 18% (train 40s, idle 180s during collection).
This script: GPU active ~90% (training continuously while collection runs in parallel).

Both MLX inference (collection) and PyTorch MPS (training) coexist on Apple Silicon
unified memory. InferenceServer runs MLX in a daemon thread; PyTorch trains on MPS
in the training thread. Proven to work on M-series chips.

Usage:
    nohup uv run python scripts/v3_train_concurrent.py > logs/v3_concurrent_stdout.log 2>&1 &
"""
import json
import logging
import multiprocessing as mp
import signal
import subprocess
import sys
import threading
import time
from collections import deque
from datetime import datetime
from pathlib import Path

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F

# ---------------------------------------------------------------------------
# Logging
# ---------------------------------------------------------------------------
Path("logs").mkdir(exist_ok=True)
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(name)s] %(message)s",
    handlers=[
        logging.StreamHandler(sys.stdout),
        logging.FileHandler("logs/v3_concurrent.log"),
    ],
)
logger = logging.getLogger("v3_concurrent")
logging.getLogger("packages.training.turn_solver").setLevel(logging.ERROR)

# ---------------------------------------------------------------------------
# Global shutdown flag
# ---------------------------------------------------------------------------
_shutdown = threading.Event()


# ---------------------------------------------------------------------------
# Data loading
# ---------------------------------------------------------------------------

def load_trajectories_from_dir(traj_dir: Path, max_transitions: int = 500_000):
    """Load all .npz trajectory files from a single directory.

    Returns dict with obs, masks, actions, floors arrays or None if empty.
    """
    obs_list, mask_list, action_list, floor_list = [], [], [], []
    loaded = 0
    n_files = 0

    files = sorted(traj_dir.glob("traj_*.npz"), key=lambda p: p.stat().st_mtime, reverse=True)
    for tf in files:
        if loaded >= max_transitions:
            break
        try:
            data = np.load(tf)
            obs = data["obs"]
            if obs.shape[1] != 480:
                continue
            masks = data["masks"]
            if masks.shape[1] < 512:
                masks = np.pad(masks, ((0, 0), (0, 512 - masks.shape[1])))
            obs_list.append(obs)
            mask_list.append(masks)
            action_list.append(data["actions"])
            floor_list.append(data["final_floors"])
            loaded += len(obs)
            n_files += 1
        except Exception:
            continue

    if not obs_list:
        return None
    return {
        "obs": np.concatenate(obs_list),
        "masks": np.concatenate(mask_list),
        "actions": np.concatenate(action_list).astype(np.int64),
        "floors": np.concatenate(floor_list),
        "n_files": n_files,
    }


def load_all_trajectories(extra_dirs=None, max_transitions: int = 500_000):
    """Load trajectories from logs/ and optional extra directories.

    Deduplicates by filename. Most recent files first.
    """
    obs_list, mask_list, action_list, floor_list = [], [], [], []
    loaded = 0
    seen = set()
    files = []

    # Search ALL trajectory dirs — experiments, collection, best, everything
    search_dirs = list(Path("logs").rglob("traj_*.npz"))
    if extra_dirs:
        for d in extra_dirs:
            search_dirs += list(Path(d).rglob("traj_*.npz"))

    for f in sorted(search_dirs, key=lambda p: p.stem, reverse=True):
        if f.name not in seen:
            seen.add(f.name)
            files.append(f)

    for tf in files:
        if loaded >= max_transitions:
            break
        try:
            data = np.load(tf)
            obs = data["obs"]
            if obs.shape[1] != 480:
                continue
            masks = data["masks"]
            if masks.shape[1] < 512:
                masks = np.pad(masks, ((0, 0), (0, 512 - masks.shape[1])))
            obs_list.append(obs)
            mask_list.append(masks)
            action_list.append(data["actions"])
            floor_list.append(data["final_floors"])
            loaded += len(obs)
        except Exception:
            continue

    if not obs_list:
        return None
    return {
        "obs": np.concatenate(obs_list),
        "masks": np.concatenate(mask_list),
        "actions": np.concatenate(action_list).astype(np.int64),
        "floors": np.concatenate(floor_list),
        "n_files": len(files),
    }


# ---------------------------------------------------------------------------
# Model factory
# ---------------------------------------------------------------------------

def make_model(device):
    from packages.training.training_config import MODEL_HIDDEN_DIM, MODEL_NUM_BLOCKS, MODEL_ACTION_DIM
    from packages.training.strategic_net import StrategicNet
    return StrategicNet(
        input_dim=480, hidden_dim=MODEL_HIDDEN_DIM,
        action_dim=MODEL_ACTION_DIM, num_blocks=MODEL_NUM_BLOCKS,
    ).to(device)


def load_checkpoint(device):
    """Load best available checkpoint, or return a fresh model."""
    from packages.training.strategic_net import StrategicNet
    candidates = [
        Path("logs/strategic_checkpoints/latest_strategic.pt"),
        Path("logs/strategic_checkpoints/best_strategic_floor9.4.pt"),
        Path("logs/strategic_checkpoints/bc_winner_v3.pt"),
    ]
    for ckpt in candidates:
        if ckpt.exists():
            logger.info("Loading checkpoint: %s", ckpt)
            return StrategicNet.load(ckpt, device=device)
    logger.warning("No checkpoint found, using fresh model")
    return make_model(device)


# ---------------------------------------------------------------------------
# Shared state between threads
# ---------------------------------------------------------------------------

class SharedState:
    """Thread-safe shared state between training and collection threads."""

    def __init__(self):
        self._lock = threading.Lock()

        # Training -> Collection: latest checkpoint path for resync
        self.latest_checkpoint: Path | None = None
        self.checkpoint_version: int = 0
        self.checkpoint_ready = threading.Event()

        # Collection -> Training: stats for logging
        self.collect_games: int = 0
        self.collect_avg_floor: float = 0.0
        self.collect_peak_floor: int = 0
        self.collect_wins: int = 0
        self.collect_gpm: float = 0.0
        self.collect_total_transitions: int = 0
        self.collect_total_combat: int = 0

        # Training -> Status: training metrics
        self.train_cycle: int = 0
        self.train_epoch: int = 0
        self.train_val_loss: float = float("inf")
        self.train_val_acc: float = 0.0
        self.train_total_transitions: int = 0
        self.train_new_transitions: int = 0

        # Timing
        self.start_time: float = time.monotonic()

    def update_checkpoint(self, path: Path, version: int):
        with self._lock:
            self.latest_checkpoint = path
            self.checkpoint_version = version
        self.checkpoint_ready.set()

    def get_checkpoint(self):
        with self._lock:
            return self.latest_checkpoint, self.checkpoint_version

    def update_collect_stats(self, games, avg_floor, peak_floor, wins, gpm,
                             total_transitions, total_combat):
        with self._lock:
            self.collect_games = games
            self.collect_avg_floor = avg_floor
            self.collect_peak_floor = peak_floor
            self.collect_wins = wins
            self.collect_gpm = gpm
            self.collect_total_transitions = total_transitions
            self.collect_total_combat = total_combat

    def update_train_stats(self, cycle, epoch, val_loss, val_acc,
                           total_transitions, new_transitions):
        with self._lock:
            self.train_cycle = cycle
            self.train_epoch = epoch
            self.train_val_loss = val_loss
            self.train_val_acc = val_acc
            self.train_total_transitions = total_transitions
            self.train_new_transitions = new_transitions

    def snapshot(self) -> dict:
        with self._lock:
            elapsed = time.monotonic() - self.start_time
            return {
                "timestamp": datetime.now().isoformat(),
                "elapsed_hours": round(elapsed / 3600, 2),
                "mode": "concurrent",
                "config_name": "v3_concurrent_bc",
                # Collection stats
                "total_games": self.collect_games,
                "total_wins": self.collect_wins,
                "avg_floor_100": round(self.collect_avg_floor, 1),
                "peak_floor": self.collect_peak_floor,
                "games_per_min": round(self.collect_gpm, 1),
                "total_transitions": self.collect_total_transitions,
                "total_combat_positions": self.collect_total_combat,
                # Training stats
                "train_cycle": self.train_cycle,
                "train_epoch": self.train_epoch,
                "train_val_loss": round(self.train_val_loss, 4),
                "train_val_acc": round(self.train_val_acc, 1),
                "train_transitions": self.train_total_transitions,
                "train_new_transitions": self.train_new_transitions,
                "checkpoint_version": self.checkpoint_version,
                # Sweep compat fields for dashboard
                "sweep_phase": "concurrent",
                "current_sweep": 0,
                "total_sweeps": 1,
                "headless": True,
                "gpu_percent": 90.0,
            }


# ---------------------------------------------------------------------------
# Training thread
# ---------------------------------------------------------------------------

def training_thread(shared: SharedState, traj_dir: Path, checkpoint_dir: Path):
    """Continuously trains BC on available trajectory data.

    Reloads .npz files from traj_dir every cycle to pick up new data from
    the collection thread. Trains for 10 epochs per cycle, validates on 15%
    held-out split. Saves checkpoint every cycle for the collection thread
    to resync from.
    """
    from packages.training.training_config import LR_HEAD_MULTIPLIERS

    device = torch.device("mps") if torch.backends.mps.is_available() else torch.device("cpu")
    logger.info("[TRAIN] Thread started on device=%s", device)

    checkpoint_dir.mkdir(parents=True, exist_ok=True)

    # Wait for initial data to be available
    prev_transitions = 0
    cycle = 0

    while not _shutdown.is_set():
        cycle += 1

        # Load all available trajectories (picks up new ones from collector)
        data = load_all_trajectories(
            extra_dirs=[str(traj_dir)],
            max_transitions=500_000,
        )

        if data is None:
            logger.info("[TRAIN] No data yet, waiting 30s...")
            _shutdown.wait(30)
            continue

        n_transitions = len(data["obs"])
        new_transitions = n_transitions - prev_transitions

        if n_transitions < 100:
            logger.info("[TRAIN] Only %d transitions, waiting 30s...", n_transitions)
            _shutdown.wait(30)
            continue

        logger.info(
            "[TRAIN] Cycle %d: %d transitions (%+d new), %d files",
            cycle, n_transitions, new_transitions, data["n_files"],
        )

        # Build or reload model
        model = make_model(device)

        # Load latest checkpoint if one exists
        ckpt_path, _ver = shared.get_checkpoint()
        if ckpt_path is not None and ckpt_path.exists():
            try:
                ckpt = torch.load(ckpt_path, map_location=device, weights_only=True)
                model.load_state_dict(ckpt["model_state_dict"])
                logger.info("[TRAIN] Resumed from checkpoint v%d", _ver)
            except Exception as e:
                logger.warning("[TRAIN] Failed to load checkpoint: %s", e)
        elif cycle == 1:
            # First cycle: try loading a pre-existing checkpoint
            for cand in [
                Path("logs/strategic_checkpoints/latest_strategic.pt"),
                Path("logs/strategic_checkpoints/best_strategic_floor9.4.pt"),
            ]:
                if cand.exists():
                    try:
                        ckpt = torch.load(cand, map_location=device, weights_only=True)
                        model.load_state_dict(ckpt["model_state_dict"])
                        logger.info("[TRAIN] Loaded initial checkpoint: %s", cand.name)
                        break
                    except Exception as e:
                        logger.warning("[TRAIN] Failed to load %s: %s", cand.name, e)

        # Prepare tensors
        N = len(data["obs"])
        obs_t = torch.from_numpy(data["obs"]).float().to(device)
        mask_t = torch.from_numpy(data["masks"]).bool().to(device)
        action_t = torch.from_numpy(data["actions"]).long().to(device)
        floor_t = torch.from_numpy(data["floors"]).float().to(device)

        # Train/val split
        perm = torch.randperm(N)
        split = int(N * 0.85)
        train_idx, val_idx = perm[:split], perm[split:]

        # Optimizer with per-head LR multipliers
        lr = 3e-4
        batch_size = 2048
        max_epochs = 10
        patience = 5

        param_groups = [
            {"params": list(model.input_proj.parameters()) + list(model.trunk.parameters()),
             "lr": lr * LR_HEAD_MULTIPLIERS.get("trunk", 1.0), "weight_decay": 0.01},
            {"params": list(model.policy_head.parameters()),
             "lr": lr * LR_HEAD_MULTIPLIERS.get("policy", 2.0), "weight_decay": 0.01},
            {"params": list(model.value_head.parameters()),
             "lr": lr * LR_HEAD_MULTIPLIERS.get("value", 3.0), "weight_decay": 0.01},
            {"params": list(model.floor_head.parameters()) + list(model.act_head.parameters()),
             "lr": lr * LR_HEAD_MULTIPLIERS.get("auxiliary", 1.0), "weight_decay": 0.01},
        ]
        optimizer = torch.optim.AdamW(param_groups, eps=1e-5)
        scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(
            optimizer, T_max=max_epochs, eta_min=1e-6,
        )

        best_val = float("inf")
        best_state = None
        no_improve = 0
        final_val_acc = 0.0

        model.train()
        for epoch in range(1, max_epochs + 1):
            if _shutdown.is_set():
                break

            # Train
            idx = train_idx[torch.randperm(len(train_idx))]
            for start in range(0, len(idx), batch_size):
                batch = idx[start:start + batch_size]
                out = model(obs_t[batch], mask_t[batch])
                bc_loss = F.nll_loss(
                    F.log_softmax(out["policy_logits"], dim=-1), action_t[batch],
                )
                v_loss = F.mse_loss(out["value"], floor_t[batch])
                loss = bc_loss + 0.5 * v_loss
                optimizer.zero_grad()
                loss.backward()
                nn.utils.clip_grad_norm_(model.parameters(), 1.0)
                optimizer.step()
            scheduler.step()

            # Validate
            model.eval()
            with torch.no_grad():
                out = model(obs_t[val_idx], mask_t[val_idx])
                val_bc = F.nll_loss(
                    F.log_softmax(out["policy_logits"], dim=-1), action_t[val_idx],
                ).item()
                val_acc = (
                    (out["policy_logits"].argmax(dim=-1) == action_t[val_idx])
                    .float().mean().item() * 100
                )
            model.train()
            final_val_acc = val_acc

            if val_bc < best_val:
                best_val = val_bc
                no_improve = 0
                best_state = {k: v.cpu().clone() for k, v in model.state_dict().items()}
            else:
                no_improve += 1

            logger.info(
                "[TRAIN] Cycle %d Epoch %d/%d: val_loss=%.4f val_acc=%.1f%% (patience=%d/%d)",
                cycle, epoch, max_epochs, val_bc, val_acc, no_improve, patience,
            )

            if no_improve >= patience:
                logger.info("[TRAIN] Early stop at epoch %d", epoch)
                break

        # Restore best weights and save checkpoint
        if best_state is not None:
            model.load_state_dict(best_state)
            model.to(device)

        # Save checkpoint
        ckpt_name = f"concurrent_v{cycle:04d}.pt"
        ckpt_path = checkpoint_dir / ckpt_name
        model.save(ckpt_path)

        # Also save as latest_strategic for other scripts
        latest_path = Path("logs/strategic_checkpoints/latest_strategic.pt")
        latest_path.parent.mkdir(parents=True, exist_ok=True)
        model.save(latest_path)

        logger.info(
            "[TRAIN] Cycle %d done: val_loss=%.4f val_acc=%.1f%% transitions=%d saved=%s",
            cycle, best_val, final_val_acc, n_transitions, ckpt_name,
        )

        # Signal collection thread
        shared.update_checkpoint(ckpt_path, cycle)
        shared.update_train_stats(
            cycle=cycle, epoch=epoch, val_loss=best_val, val_acc=final_val_acc,
            total_transitions=n_transitions, new_transitions=new_transitions,
        )
        prev_transitions = n_transitions

        # Free GPU memory between cycles
        del obs_t, mask_t, action_t, floor_t, model, optimizer, scheduler
        if torch.backends.mps.is_available():
            torch.mps.empty_cache()

        # Brief pause to let collection thread accumulate data
        _shutdown.wait(5)

    logger.info("[TRAIN] Thread exiting")


# ---------------------------------------------------------------------------
# Collection thread
# ---------------------------------------------------------------------------

def collection_thread(shared: SharedState, traj_dir: Path, combat_dir: Path):
    """Continuously collects games with the current model.

    Runs InferenceServer (MLX) + worker pool. Saves trajectories to traj_dir
    for the training thread to pick up. Periodically resyncs model weights
    from the latest training checkpoint.
    """
    from packages.training.inference_server import InferenceServer
    from packages.training.seed_pool import SeedPool
    from packages.training.strategic_net import StrategicNet
    from packages.training.training_config import MODEL_ACTION_DIM, MODEL_HIDDEN_DIM, MODEL_NUM_BLOCKS, SOLVER_BUDGETS, MCTS_COMBAT_ENABLED
    from packages.training.worker import _play_one_game, _worker_init

    # Config-driven defaults (adapter overrides per room type at runtime)
    _default_solver_ms = SOLVER_BUDGETS["monster"][0]  # 50ms base for monsters

    logger.info("[COLLECT] Thread started")

    traj_dir.mkdir(parents=True, exist_ok=True)
    combat_dir.mkdir(parents=True, exist_ok=True)

    # Load initial model for inference server
    device = torch.device("mps") if torch.backends.mps.is_available() else torch.device("cpu")
    model = StrategicNet(
        input_dim=480, hidden_dim=MODEL_HIDDEN_DIM,
        action_dim=MODEL_ACTION_DIM, num_blocks=MODEL_NUM_BLOCKS,
    ).to(device)

    # Try loading existing checkpoint
    for cand in [
        Path("logs/strategic_checkpoints/latest_strategic.pt"),
        Path("logs/strategic_checkpoints/best_strategic_floor9.4.pt"),
    ]:
        if cand.exists():
            try:
                model = StrategicNet.load(cand, device=device)
                logger.info("[COLLECT] Loaded initial model: %s", cand.name)
                break
            except Exception as e:
                logger.warning("[COLLECT] Failed to load %s: %s", cand.name, e)

    # Start inference server (MLX backend)
    N_WORKERS = 10
    server = InferenceServer(
        n_workers=N_WORKERS, max_batch_size=64, batch_timeout_ms=10.0,
    )
    server.sync_strategic_from_pytorch(model, version=0)
    server.start()
    logger.info("[COLLECT] Inference server started (MLX)")

    # Worker pool
    ctx = mp.get_context("spawn")
    shm = getattr(server, "shm_info", None)
    pool = ctx.Pool(
        processes=N_WORKERS,
        initializer=_worker_init,
        initargs=(server.request_q, server.response_qs, server.slot_q, shm),
    )
    logger.info("[COLLECT] Worker pool: %d processes", N_WORKERS)

    seed_pool = SeedPool()
    total_games = 0
    total_transitions = 0
    total_combat = 0
    recent_floors = deque(maxlen=200)
    peak_floor = 0
    wins = 0
    start_time = time.monotonic()
    traj_counter = 0
    combat_counter = 0
    batch_size = 32
    last_resync_games = 0
    resync_version = 0

    RESYNC_EVERY = 500  # Resync model weights every N games

    logger.info("[COLLECT] Starting collection loop (batch_size=%d, resync_every=%d)",
                batch_size, RESYNC_EVERY)

    while not _shutdown.is_set():
        # Check for model resync
        if total_games - last_resync_games >= RESYNC_EVERY:
            ckpt_path, version = shared.get_checkpoint()
            if ckpt_path is not None and version > resync_version and ckpt_path.exists():
                try:
                    resynced = StrategicNet.load(ckpt_path, device=device)
                    server.sync_strategic_from_pytorch(resynced, version=version)
                    resync_version = version
                    last_resync_games = total_games
                    logger.info(
                        "[COLLECT] Resynced model v%d at game %d",
                        version, total_games,
                    )
                except Exception as e:
                    logger.warning("[COLLECT] Resync failed: %s", e)

        # Submit batch of games
        seeds = [seed_pool.get_seed() for _ in range(batch_size)]
        async_results = [
            pool.apply_async(
                _play_one_game,
                (seed, 0, 0.8, total_games, _default_solver_ms, False, MCTS_COMBAT_ENABLED, 0),
            )
            for seed in seeds
        ]

        # Collect results
        for ar, seed in zip(async_results, seeds):
            if _shutdown.is_set():
                break
            try:
                result = ar.get(timeout=300)
            except Exception as e:
                logger.warning("[COLLECT] Game %s failed: %s", seed, e)
                continue

            total_games += 1
            floor = result["floor"]
            recent_floors.append(floor)
            peak_floor = max(peak_floor, floor)
            if result["won"]:
                wins += 1

            transitions = result.get("transitions", [])
            total_transitions += len(transitions)

            # Save all trajectories (floor 1+)
            if transitions:
                traj_counter += 1
                fname = f"traj_{traj_counter:06d}_F{floor:02d}.npz"
                try:
                    obs = np.stack([t["obs"] for t in transitions])
                    masks = np.stack([t["action_mask"] for t in transitions])
                    np.savez_compressed(
                        traj_dir / fname,
                        obs=obs, masks=masks,
                        actions=np.array([t["action"] for t in transitions], dtype=np.int32),
                        rewards=np.array([t["reward"] for t in transitions], dtype=np.float32),
                        dones=np.array([t["done"] for t in transitions], dtype=bool),
                        values=np.array([t["value"] for t in transitions], dtype=np.float32),
                        log_probs=np.array([t["log_prob"] for t in transitions], dtype=np.float32),
                        final_floors=np.array([t["final_floor"] for t in transitions], dtype=np.float32),
                        cleared_act1=np.array([t["cleared_act1"] for t in transitions], dtype=np.float32),
                    )
                except Exception as e:
                    logger.warning("[COLLECT] Failed to save trajectory: %s", e)

            # Save combat data
            for combat in result.get("combats", []):
                vec = combat.get("combat_state_vector")
                if vec is not None:
                    combat_counter += 1
                    total_combat += 1
                    survived = floor > combat.get("floor", 0)
                    try:
                        np.savez_compressed(
                            combat_dir / f"combat_{combat_counter:06d}.npz",
                            combat_obs=vec, won=np.array(survived, dtype=bool),
                        )
                    except Exception:
                        pass

        # Log progress
        elapsed = time.monotonic() - start_time
        gpm = total_games / max(elapsed / 60, 0.01)
        avg_floor = sum(recent_floors) / max(len(recent_floors), 1)

        if total_games % batch_size == 0:
            logger.info(
                "[COLLECT] Games %d | Floor %.1f | Peak %d | Wins %d | "
                "Trans %d | Combat %d | %.0f g/min | model v%d",
                total_games, avg_floor, peak_floor, wins,
                total_transitions, total_combat, gpm, resync_version,
            )

        # Update shared state
        shared.update_collect_stats(
            games=total_games, avg_floor=avg_floor, peak_floor=peak_floor,
            wins=wins, gpm=gpm, total_transitions=total_transitions,
            total_combat=total_combat,
        )

    # Cleanup
    logger.info("[COLLECT] Shutting down pool...")
    pool.terminate()
    pool.join()
    server.stop()
    logger.info("[COLLECT] Thread exiting (games=%d, transitions=%d)", total_games, total_transitions)


# ---------------------------------------------------------------------------
# Status writer (runs in main thread)
# ---------------------------------------------------------------------------

def write_status(shared: SharedState, run_dir: Path):
    """Write status.json for dashboard consumption."""
    status = shared.snapshot()
    try:
        (run_dir / "status.json").write_text(json.dumps(status, indent=2))
    except Exception:
        pass


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    # Prevent Mac sleep
    caffeinate = subprocess.Popen(["caffeinate", "-dims"], stdout=subprocess.DEVNULL)
    logger.info("caffeinate PID=%d", caffeinate.pid)

    def on_signal(signum, frame):
        logger.info("Shutdown requested (signal=%d)", signum)
        _shutdown.set()

    signal.signal(signal.SIGINT, on_signal)
    signal.signal(signal.SIGTERM, on_signal)

    # Setup directories
    run_dir = Path("logs/v3_concurrent")
    traj_dir = run_dir / "all_trajectories"
    combat_dir = run_dir / "combat_data"
    checkpoint_dir = run_dir / "checkpoints"
    for d in [run_dir, traj_dir, combat_dir, checkpoint_dir]:
        d.mkdir(parents=True, exist_ok=True)

    # Symlink logs/active for dashboard
    active_link = Path("logs/active")
    if active_link.is_symlink():
        active_link.unlink()
    active_link.symlink_to(run_dir.resolve())
    logger.info("logs/active -> %s", run_dir)

    # Shared state
    shared = SharedState()

    logger.info("=" * 60)
    logger.info("CONCURRENT BC TRAINING + COLLECTION")
    logger.info("  Training: MPS GPU, BC 10 epochs/cycle, batch=2048")
    logger.info("  Collection: MLX inference, 10 workers, 32 games/batch")
    logger.info("  Resync: every 500 games")
    logger.info("=" * 60)

    # Start threads
    train_t = threading.Thread(
        target=training_thread,
        args=(shared, traj_dir, checkpoint_dir),
        name="TrainingThread",
        daemon=True,
    )
    collect_t = threading.Thread(
        target=collection_thread,
        args=(shared, traj_dir, combat_dir),
        name="CollectionThread",
        daemon=True,
    )

    collect_t.start()
    logger.info("Collection thread started")

    # Small delay so collection thread can start producing data
    time.sleep(5)

    train_t.start()
    logger.info("Training thread started")

    # Main loop: write status and wait
    try:
        while not _shutdown.is_set():
            write_status(shared, run_dir)
            _shutdown.wait(10)
    except KeyboardInterrupt:
        _shutdown.set()

    logger.info("Waiting for threads to finish...")

    # Wait for threads to notice shutdown
    train_t.join(timeout=30)
    collect_t.join(timeout=30)

    if train_t.is_alive():
        logger.warning("Training thread did not exit within 30s")
    if collect_t.is_alive():
        logger.warning("Collection thread did not exit within 30s")

    # Final status
    write_status(shared, run_dir)
    status = shared.snapshot()

    logger.info("=" * 60)
    logger.info("CONCURRENT TRAINING COMPLETE")
    logger.info("  Games: %d", status["total_games"])
    logger.info("  Avg floor: %.1f", status["avg_floor_100"])
    logger.info("  Peak floor: %d", status["peak_floor"])
    logger.info("  Training cycles: %d", status["train_cycle"])
    logger.info("  Final val_loss: %.4f", status["train_val_loss"])
    logger.info("  Final val_acc: %.1f%%", status["train_val_acc"])
    logger.info("  Total transitions: %d", status["total_transitions"])
    logger.info("  Duration: %.1fh", status["elapsed_hours"])
    logger.info("=" * 60)

    caffeinate.terminate()


if __name__ == "__main__":
    main()
