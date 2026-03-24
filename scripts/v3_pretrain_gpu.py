"""V3 GPU-optimized pretrain: max batch size, parallel data loading.

Uses all 24GB unified memory on M4. Trains:
1. Strategic BC (100 epochs, batch 2048, all trajectory data)
2. CombatNet (100 epochs, batch 2048, all combat .npz data)
3. Value head calibration (20 epochs, frozen policy)
4. Quick 50-game eval

Usage: uv run python scripts/v3_pretrain_gpu.py
"""
import logging
import signal
import subprocess
import sys
import time
from pathlib import Path

import numpy as np

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(name)s] %(message)s",
    handlers=[
        logging.StreamHandler(sys.stdout),
        logging.FileHandler("logs/v3_pretrain_gpu.log"),
    ],
)
logger = logging.getLogger("pretrain_gpu")


def load_all_trajectories(max_transitions: int = 200_000) -> dict:
    """Load ALL trajectory .npz files into memory. Returns numpy arrays."""
    obs_list, mask_list, action_list, floor_list = [], [], [], []
    loaded = 0

    # Gather from all best_trajectories + all_trajectories dirs
    traj_files = []
    for pattern in ["logs/**/best_trajectories/traj_F*.npz", "logs/**/all_trajectories/traj_F*.npz"]:
        traj_files.extend(Path(".").glob(pattern))

    # Deduplicate by filename, sort by floor descending (best games first)
    seen = set()
    unique = []
    for f in traj_files:
        if f.name not in seen:
            seen.add(f.name)
            unique.append(f)
    unique.sort(key=lambda p: p.stem, reverse=True)
    logger.info("Found %d unique trajectory files", len(unique))

    for tf in unique:
        if loaded >= max_transitions:
            break
        try:
            data = np.load(tf)
            obs = data["obs"]
            if obs.shape[1] != 480:  # Filter mismatched dims
                continue
            n = len(obs)
            masks = data["masks"]
            actions = data["actions"]
            floors = data["final_floors"]

            # Pad masks if needed
            if masks.shape[1] < 512:
                masks = np.pad(masks, ((0, 0), (0, 512 - masks.shape[1])))

            obs_list.append(obs)
            mask_list.append(masks)
            action_list.append(actions)
            floor_list.append(floors)
            loaded += n
        except Exception as e:
            continue

    if not obs_list:
        return {"obs": np.empty((0, 480)), "masks": np.empty((0, 512)),
                "actions": np.empty(0, dtype=np.int32), "floors": np.empty(0)}

    return {
        "obs": np.concatenate(obs_list),
        "masks": np.concatenate(mask_list),
        "actions": np.concatenate(action_list).astype(np.int64),
        "floors": np.concatenate(floor_list),
        "n_files": len(unique),
    }


def load_all_combat_data() -> dict:
    """Load ALL combat .npz files into memory."""
    obs_list, won_list = [], []

    combat_files = list(Path(".").glob("logs/**/combat_data/combat_*.npz"))
    logger.info("Found %d combat .npz files", len(combat_files))

    for cf in combat_files:
        try:
            data = np.load(cf)
            obs = data["combat_obs"]
            if obs.shape[0] != 298:
                continue
            obs_list.append(obs)
            won_list.append(bool(data["won"]))
        except Exception:
            continue

    if not obs_list:
        return {"obs": np.empty((0, 298)), "won": np.empty(0, dtype=bool)}

    return {
        "obs": np.stack(obs_list),
        "won": np.array(won_list, dtype=bool),
    }


def train_strategic_bc(data: dict, epochs: int = 100, batch_size: int = 2048):
    """Train strategic model via BC with max GPU utilization."""
    import torch
    import torch.nn as nn
    import torch.nn.functional as F
    from packages.training.strategic_net import StrategicNet, _get_device
    from packages.training.training_config import MODEL_HIDDEN_DIM, MODEL_NUM_BLOCKS, MODEL_ACTION_DIM, LR_HEAD_MULTIPLIERS

    device = _get_device()
    model = StrategicNet(
        input_dim=480, hidden_dim=MODEL_HIDDEN_DIM,
        action_dim=MODEL_ACTION_DIM, num_blocks=MODEL_NUM_BLOCKS,
    ).to(device)
    logger.info("Model: %d params on %s", sum(p.numel() for p in model.parameters()), device)

    N = len(data["obs"])
    logger.info("Training BC on %d transitions, batch=%d, epochs=%d", N, batch_size, epochs)

    # Move all data to GPU tensors upfront (fits in unified memory)
    obs_t = torch.from_numpy(data["obs"]).float().to(device)
    mask_t = torch.from_numpy(data["masks"]).bool().to(device)
    action_t = torch.from_numpy(data["actions"]).long().to(device)
    floor_t = torch.from_numpy(data["floors"]).float().to(device)

    data_mb = (obs_t.nbytes + mask_t.nbytes + action_t.nbytes + floor_t.nbytes) / 1024**2
    logger.info("Data on GPU: %.0f MB", data_mb)

    # Per-head LR optimizer
    param_groups = [
        {"params": list(model.input_proj.parameters()) + list(model.trunk.parameters()),
         "lr": 3e-4 * LR_HEAD_MULTIPLIERS.get("trunk", 1.0)},
        {"params": list(model.policy_head.parameters()),
         "lr": 3e-4 * LR_HEAD_MULTIPLIERS.get("policy", 2.0)},
        {"params": list(model.value_head.parameters()),
         "lr": 3e-4 * LR_HEAD_MULTIPLIERS.get("value", 3.0)},
        {"params": list(model.floor_head.parameters()) + list(model.act_head.parameters()),
         "lr": 3e-4 * LR_HEAD_MULTIPLIERS.get("auxiliary", 1.0)},
    ]
    optimizer = torch.optim.AdamW(param_groups, eps=1e-5, weight_decay=0.01)
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(optimizer, T_max=epochs, eta_min=1e-5)

    model.train()
    t0 = time.monotonic()

    for epoch in range(1, epochs + 1):
        indices = torch.randperm(N, device=device)
        total_loss = 0.0
        total_correct = 0
        n_batches = 0

        for start in range(0, N, batch_size):
            end = min(start + batch_size, N)
            idx = indices[start:end]

            out = model(obs_t[idx], mask_t[idx])
            log_probs = F.log_softmax(out["policy_logits"], dim=-1)

            # BC loss: cross-entropy on actions
            bc_loss = F.nll_loss(log_probs, action_t[idx])

            # Value loss: predict final floor
            value_loss = F.mse_loss(out["value"], floor_t[idx])

            # Floor prediction auxiliary
            floor_loss = F.mse_loss(out["floor_pred"], floor_t[idx])

            loss = bc_loss + 0.5 * value_loss + 0.25 * floor_loss

            optimizer.zero_grad()
            loss.backward()
            nn.utils.clip_grad_norm_(model.parameters(), 1.0)
            optimizer.step()

            total_loss += loss.item()
            total_correct += (out["policy_logits"].argmax(dim=-1) == action_t[idx]).sum().item()
            n_batches += 1

        scheduler.step()
        acc = total_correct / N * 100
        avg_loss = total_loss / n_batches
        lr = optimizer.param_groups[0]["lr"]

        if epoch % 10 == 0 or epoch <= 5 or epoch == epochs:
            elapsed = time.monotonic() - t0
            logger.info("BC epoch %d/%d: loss=%.4f, acc=%.1f%%, lr=%.1e (%.0fs)",
                        epoch, epochs, avg_loss, acc, lr, elapsed)

    # Save
    ckpt_dir = Path("logs/strategic_checkpoints")
    ckpt_dir.mkdir(parents=True, exist_ok=True)
    ckpt_path = ckpt_dir / "bc_100ep_v3.pt"
    model.save(ckpt_path)
    logger.info("Saved: %s", ckpt_path)

    # Also save as winner/latest
    import shutil
    shutil.copy2(ckpt_path, ckpt_dir / "bc_winner_v3.pt")
    shutil.copy2(ckpt_path, ckpt_dir / "latest_strategic.pt")

    return model, {"bc_accuracy": acc, "bc_loss": avg_loss, "transitions": N}


def train_combat_net(data: dict, epochs: int = 100, batch_size: int = 2048):
    """Train CombatNet on encoded combat states."""
    import torch
    import torch.nn.functional as F
    from packages.training.combat_net import CombatNet

    N = len(data["obs"])
    if N < 50:
        logger.warning("Only %d combat positions — skipping CombatNet", N)
        return None, {}

    device = torch.device("mps") if torch.backends.mps.is_available() else torch.device("cpu")
    model = CombatNet(input_dim=298, hidden_dim=256, num_layers=3).to(device)
    logger.info("CombatNet: %d params, %d positions, batch=%d",
                sum(p.numel() for p in model.parameters()), N, batch_size)

    # Data to GPU
    obs_t = torch.from_numpy(data["obs"]).float().to(device)
    won_t = torch.from_numpy(data["won"].astype(np.float32)).to(device)

    optimizer = torch.optim.Adam(model.parameters(), lr=1e-3)
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(optimizer, T_max=epochs, eta_min=1e-5)

    model.train()
    t0 = time.monotonic()

    for epoch in range(1, epochs + 1):
        indices = torch.randperm(N, device=device)
        total_loss = 0.0
        total_correct = 0
        n_batches = 0

        for start in range(0, N, batch_size):
            end = min(start + batch_size, N)
            idx = indices[start:end]

            pred = model(obs_t[idx]).squeeze(-1)
            loss = F.binary_cross_entropy(pred, won_t[idx])

            optimizer.zero_grad()
            loss.backward()
            optimizer.step()

            total_loss += loss.item()
            total_correct += ((pred > 0.5) == won_t[idx].bool()).sum().item()
            n_batches += 1

        scheduler.step()
        acc = total_correct / N * 100

        if epoch % 10 == 0 or epoch <= 3 or epoch == epochs:
            logger.info("CombatNet epoch %d/%d: loss=%.4f, acc=%.1f%% (%.0fs)",
                        epoch, epochs, total_loss / n_batches, acc,
                        time.monotonic() - t0)

    # Save where worker auto-loads
    save_dir = Path("logs/active")
    save_dir.mkdir(parents=True, exist_ok=True)
    model.save(save_dir / "combat_net.pt")
    logger.info("CombatNet saved: logs/active/combat_net.pt")

    return model, {"loss": total_loss / n_batches, "accuracy": acc, "positions": N}


def eval_model(n_games: int = 50):
    """Eval the saved model with 50 games."""
    import multiprocessing as mp
    from packages.training.strategic_net import StrategicNet
    from packages.training.inference_server import InferenceServer
    from packages.training.seed_pool import SeedPool
    from packages.training.worker import _play_one_game, _worker_init
    from packages.training.training_config import MODEL_ACTION_DIM, MODEL_HIDDEN_DIM, MODEL_NUM_BLOCKS

    ckpt = Path("logs/strategic_checkpoints/bc_winner_v3.pt")
    if not ckpt.exists():
        logger.warning("No checkpoint for eval")
        return 0

    device = "mps" if __import__("torch").backends.mps.is_available() else "cpu"
    model = StrategicNet(input_dim=480, hidden_dim=MODEL_HIDDEN_DIM,
                         action_dim=MODEL_ACTION_DIM, num_blocks=MODEL_NUM_BLOCKS)
    model.load(ckpt)
    model.to(device)

    N_WORKERS = 10
    server = InferenceServer(n_workers=N_WORKERS, max_batch_size=32, batch_timeout_ms=15.0)
    server.sync_strategic_from_pytorch(model, version=0)
    server.start()

    ctx = mp.get_context("spawn")
    shm_info = getattr(server, "shm_info", None)
    pool = ctx.Pool(processes=N_WORKERS, initializer=_worker_init,
                    initargs=(server.request_q, server.response_qs, server.slot_q, shm_info))

    seed_pool = SeedPool()
    ars = []
    for _ in range(n_games):
        seed = seed_pool.get_seed()
        ars.append(pool.apply_async(_play_one_game, (seed, 0, 0.5, 0, 50.0, False, False, 0)))

    floors = []
    for ar in ars:
        try:
            r = ar.get(timeout=120)
            floors.append(r["floor"])
        except Exception:
            pass

    pool.terminate()
    pool.join()
    server.stop()

    avg = sum(floors) / max(len(floors), 1)
    peak = max(floors) if floors else 0
    wins = sum(1 for f in floors if f >= 55)
    logger.info("EVAL %d games: avg=%.1f, median=%d, peak=%d, wins=%d",
                len(floors), avg, sorted(floors)[len(floors)//2] if floors else 0, peak, wins)
    return avg


def main():
    caffeinate = subprocess.Popen(["caffeinate", "-dims"], stdout=subprocess.DEVNULL)

    def cleanup(sig=None, frame=None):
        caffeinate.terminate()
        if sig:
            sys.exit(0)
    signal.signal(signal.SIGINT, cleanup)
    signal.signal(signal.SIGTERM, cleanup)

    Path("logs/v3_pretrain_gpu").mkdir(parents=True, exist_ok=True)

    logger.info("=" * 60)
    logger.info("V3 GPU PRETRAIN — max batch, full data")
    logger.info("=" * 60)

    # Phase 1: Load data
    logger.info("Loading trajectory data...")
    traj_data = load_all_trajectories(max_transitions=200_000)
    logger.info("Loaded %d transitions from %d files",
                len(traj_data["obs"]), traj_data.get("n_files", 0))

    logger.info("Loading combat data...")
    combat_data = load_all_combat_data()
    logger.info("Loaded %d combat positions", len(combat_data["obs"]))

    # Phase 2: Strategic BC
    logger.info("=" * 60)
    logger.info("PHASE 1: Strategic BC (100 epochs, batch 2048)")
    logger.info("=" * 60)
    t0 = time.monotonic()
    model, bc_metrics = train_strategic_bc(traj_data, epochs=100, batch_size=2048)
    bc_time = time.monotonic() - t0
    logger.info("BC done in %.1f min: %s", bc_time / 60, bc_metrics)

    # Phase 3: CombatNet
    logger.info("=" * 60)
    logger.info("PHASE 2: CombatNet (100 epochs, batch 2048)")
    logger.info("=" * 60)
    t0 = time.monotonic()
    cnet, combat_metrics = train_combat_net(combat_data, epochs=100, batch_size=2048)
    combat_time = time.monotonic() - t0
    logger.info("CombatNet done in %.1f min: %s", combat_time / 60, combat_metrics)

    # Phase 4: Eval
    logger.info("=" * 60)
    logger.info("PHASE 3: Eval (50 games)")
    logger.info("=" * 60)
    avg_floor = eval_model(50)

    # Summary
    logger.info("=" * 60)
    logger.info("SUMMARY")
    logger.info("  Strategic BC: acc=%.1f%%, loss=%.4f, %d transitions",
                bc_metrics.get("bc_accuracy", 0), bc_metrics.get("bc_loss", 0),
                bc_metrics.get("transitions", 0))
    if combat_metrics:
        logger.info("  CombatNet: acc=%.1f%%, loss=%.4f, %d positions",
                    combat_metrics.get("accuracy", 0), combat_metrics.get("loss", 0),
                    combat_metrics.get("positions", 0))
    logger.info("  Eval: avg_floor=%.1f", avg_floor)
    logger.info("  Checkpoints: bc_winner_v3.pt, combat_net.pt")
    logger.info("=" * 60)

    cleanup()


if __name__ == "__main__":
    main()
