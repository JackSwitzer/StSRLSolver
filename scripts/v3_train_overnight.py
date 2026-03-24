"""V3 overnight training: proper representation learning.

Phase 1: CombatNet on 155k positions (5 min)
Phase 2: Strategic BC with dropout + validation split (30 min)
Phase 3: Value head calibration on held-out data (10 min)
Phase 4: Collect 2000 games WITH the new model (higher floors = new data)
Phase 5: Retrain BC on old + new data (30 min)
Phase 6: PPO fine-tune on fresh data (remaining time)

Loops phases 4-6 until killed.

Usage: nohup uv run python scripts/v3_train_overnight.py > logs/v3_train_overnight.log 2>&1 &
"""
import json
import logging
import multiprocessing as mp
import signal
import shutil
import subprocess
import sys
import time
from collections import deque
from pathlib import Path

import numpy as np

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(name)s] %(message)s",
    handlers=[
        logging.StreamHandler(sys.stdout),
        logging.FileHandler("logs/v3_train_overnight.log"),
    ],
)
logger = logging.getLogger("v3_train")
logging.getLogger("packages.training.turn_solver").setLevel(logging.ERROR)


def load_strategic_data(max_transitions=200_000):
    """Load trajectory data, sorted by floor (best first)."""
    obs_list, mask_list, action_list, floor_list = [], [], [], []
    loaded = 0

    files = []
    seen = set()
    for f in sorted(Path("logs").rglob("best_trajectories/traj_F*.npz"), key=lambda p: p.stem, reverse=True):
        if f.name not in seen:
            seen.add(f.name)
            files.append(f)
    for f in sorted(Path("logs").rglob("all_trajectories/traj_*.npz"), key=lambda p: p.stem, reverse=True):
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


def load_combat_data():
    """Load all combat .npz files."""
    obs_list, won_list = [], []
    for cf in Path("logs").rglob("combat_data/combat_*.npz"):
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
        return None
    return {"obs": np.stack(obs_list), "won": np.array(won_list, dtype=bool)}


def train_combat_net(combat_data, epochs=50):
    """Train CombatNet on combat positions."""
    import torch
    import torch.nn.functional as F
    from packages.training.combat_net import CombatNet

    device = torch.device("mps") if torch.backends.mps.is_available() else torch.device("cpu")
    model = CombatNet(input_dim=298, hidden_dim=256, num_layers=3).to(device)

    N = len(combat_data["obs"])
    obs_t = torch.from_numpy(combat_data["obs"]).float().to(device)
    won_t = torch.from_numpy(combat_data["won"].astype(np.float32)).to(device)

    # Train/val split (90/10)
    perm = torch.randperm(N)
    split = int(N * 0.9)
    train_idx, val_idx = perm[:split], perm[split:]

    optimizer = torch.optim.Adam(model.parameters(), lr=1e-3, weight_decay=1e-4)
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(optimizer, T_max=epochs, eta_min=1e-5)
    best_val_loss = float("inf")

    model.train()
    for epoch in range(1, epochs + 1):
        # Train
        idx = train_idx[torch.randperm(len(train_idx))]
        for start in range(0, len(idx), 2048):
            batch = idx[start:start+2048]
            pred = model(obs_t[batch]).squeeze(-1)
            loss = F.binary_cross_entropy(pred, won_t[batch])
            optimizer.zero_grad()
            loss.backward()
            optimizer.step()
        scheduler.step()

        # Val
        model.eval()
        with torch.no_grad():
            val_pred = model(obs_t[val_idx]).squeeze(-1)
            val_loss = F.binary_cross_entropy(val_pred, won_t[val_idx]).item()
            val_acc = ((val_pred > 0.5) == won_t[val_idx].bool()).float().mean().item() * 100
        model.train()

        if val_loss < best_val_loss:
            best_val_loss = val_loss
            torch.save(model.state_dict(), "logs/active/combat_net_best.pt")

        if epoch % 10 == 0 or epoch <= 3:
            logger.info("CombatNet %d/%d: val_loss=%.4f, val_acc=%.1f%%", epoch, epochs, val_loss, val_acc)

    # Load best
    model.load_state_dict(torch.load("logs/active/combat_net_best.pt", weights_only=True))
    model.save(Path("logs/active/combat_net.pt"))
    logger.info("CombatNet: best val_loss=%.4f, %d positions", best_val_loss, N)
    return model


def train_strategic_bc(data, epochs=100, val_split=0.1):
    """BC with train/val split and early stopping."""
    import torch
    import torch.nn as nn
    import torch.nn.functional as F
    from packages.training.strategic_net import StrategicNet
    from packages.training.training_config import MODEL_HIDDEN_DIM, MODEL_NUM_BLOCKS, MODEL_ACTION_DIM, LR_HEAD_MULTIPLIERS

    device = torch.device("mps") if torch.backends.mps.is_available() else torch.device("cpu")
    model = StrategicNet(input_dim=480, hidden_dim=MODEL_HIDDEN_DIM,
                         action_dim=MODEL_ACTION_DIM, num_blocks=MODEL_NUM_BLOCKS).to(device)

    N = len(data["obs"])
    obs_t = torch.from_numpy(data["obs"]).float().to(device)
    mask_t = torch.from_numpy(data["masks"]).bool().to(device)
    action_t = torch.from_numpy(data["actions"]).long().to(device)
    floor_t = torch.from_numpy(data["floors"]).float().to(device)

    # Train/val split
    perm = torch.randperm(N)
    split = int(N * (1 - val_split))
    train_idx, val_idx = perm[:split], perm[split:]

    param_groups = [
        {"params": list(model.input_proj.parameters()) + list(model.trunk.parameters()),
         "lr": 3e-4 * LR_HEAD_MULTIPLIERS.get("trunk", 1.0), "weight_decay": 0.01},
        {"params": list(model.policy_head.parameters()),
         "lr": 3e-4 * LR_HEAD_MULTIPLIERS.get("policy", 2.0), "weight_decay": 0.01},
        {"params": list(model.value_head.parameters()),
         "lr": 3e-4 * LR_HEAD_MULTIPLIERS.get("value", 3.0), "weight_decay": 0.01},
        {"params": list(model.floor_head.parameters()) + list(model.act_head.parameters()),
         "lr": 3e-4 * LR_HEAD_MULTIPLIERS.get("auxiliary", 1.0), "weight_decay": 0.01},
    ]
    optimizer = torch.optim.AdamW(param_groups, eps=1e-5)
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(optimizer, T_max=epochs, eta_min=1e-5)

    best_val_loss = float("inf")
    patience = 20
    no_improve = 0

    model.train()
    t0 = time.monotonic()

    for epoch in range(1, epochs + 1):
        # Train
        idx = train_idx[torch.randperm(len(train_idx))]
        for start in range(0, len(idx), 2048):
            batch = idx[start:start+2048]
            out = model(obs_t[batch], mask_t[batch])
            bc_loss = F.nll_loss(F.log_softmax(out["policy_logits"], dim=-1), action_t[batch])
            value_loss = F.mse_loss(out["value"], floor_t[batch])
            loss = bc_loss + 0.5 * value_loss
            optimizer.zero_grad()
            loss.backward()
            nn.utils.clip_grad_norm_(model.parameters(), 1.0)
            optimizer.step()
        scheduler.step()

        # Val
        model.eval()
        with torch.no_grad():
            out = model(obs_t[val_idx], mask_t[val_idx])
            val_bc = F.nll_loss(F.log_softmax(out["policy_logits"], dim=-1), action_t[val_idx]).item()
            val_acc = (out["policy_logits"].argmax(dim=-1) == action_t[val_idx]).float().mean().item() * 100
            val_vloss = F.mse_loss(out["value"], floor_t[val_idx]).item()
        model.train()

        if val_bc < best_val_loss:
            best_val_loss = val_bc
            no_improve = 0
            ckpt_dir = Path("logs/strategic_checkpoints")
            ckpt_dir.mkdir(parents=True, exist_ok=True)
            model.save(ckpt_dir / "bc_best_val.pt")
        else:
            no_improve += 1

        if epoch % 10 == 0 or epoch <= 5 or epoch == epochs:
            logger.info("BC %d/%d: val_loss=%.4f, val_acc=%.1f%%, val_value=%.4f (%.0fs, patience=%d/%d)",
                        epoch, epochs, val_bc, val_acc, val_vloss,
                        time.monotonic() - t0, no_improve, patience)

        if no_improve >= patience:
            logger.info("Early stopping at epoch %d (no improvement for %d epochs)", epoch, patience)
            break

    # Load best checkpoint
    model.load(Path("logs/strategic_checkpoints/bc_best_val.pt"))
    shutil.copy2("logs/strategic_checkpoints/bc_best_val.pt", "logs/strategic_checkpoints/bc_winner_v3.pt")
    shutil.copy2("logs/strategic_checkpoints/bc_best_val.pt", "logs/strategic_checkpoints/latest_strategic.pt")

    logger.info("BC done: best_val_loss=%.4f, %d transitions", best_val_loss, N)
    return model


def collect_games(model, n_games=2000):
    """Collect games with trained model, save trajectories + combat data."""
    from packages.training.inference_server import InferenceServer
    from packages.training.seed_pool import SeedPool
    from packages.training.worker import _play_one_game, _worker_init

    N_WORKERS = 10
    server = InferenceServer(n_workers=N_WORKERS, max_batch_size=32, batch_timeout_ms=15.0)
    server.sync_strategic_from_pytorch(model, version=0)
    server.start()

    ctx = mp.get_context("spawn")
    shm_info = getattr(server, "shm_info", None)
    pool = ctx.Pool(processes=N_WORKERS, initializer=_worker_init,
                    initargs=(server.request_q, server.response_qs, server.slot_q, shm_info))

    seed_pool = SeedPool()
    traj_dir = Path("logs/v3_train_overnight/trajectories")
    combat_dir = Path("logs/v3_train_overnight/combat_data")
    traj_dir.mkdir(parents=True, exist_ok=True)
    combat_dir.mkdir(parents=True, exist_ok=True)

    floors = []
    saved_trajs = 0
    saved_combat = 0
    t0 = time.monotonic()

    # Submit all games
    ars = []
    for _ in range(n_games):
        seed = seed_pool.get_seed()
        ars.append((seed, pool.apply_async(_play_one_game, (seed, 0, 0.8, 0, 20.0, False, False, 0))))

    for seed, ar in ars:
        try:
            r = ar.get(timeout=120)
        except Exception:
            continue

        floors.append(r["floor"])
        transitions = r.get("transitions", [])

        # Save trajectory
        if transitions:
            saved_trajs += 1
            fname = f"traj_{saved_trajs:06d}_F{r['floor']:02d}.npz"
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
                traj_dir / fname, obs=obs, masks=masks, actions=actions, rewards=rewards,
                dones=dones, values=values, log_probs=log_probs,
                final_floors=final_floors, cleared_act1=cleared_act1,
            )

        # Save combat data
        for combat in r.get("combats", []):
            vec = combat.get("combat_state_vector")
            if vec is not None:
                saved_combat += 1
                survived = r["floor"] > combat.get("floor", 0)
                np.savez_compressed(
                    combat_dir / f"combat_{saved_combat:06d}.npz",
                    combat_obs=vec, won=np.array(survived, dtype=bool),
                )

        if len(floors) % 500 == 0:
            avg = sum(floors) / len(floors)
            logger.info("Collect %d/%d: avg_floor=%.1f, trajs=%d, combat=%d",
                        len(floors), n_games, avg, saved_trajs, saved_combat)

    pool.terminate()
    pool.join()
    server.stop()

    avg = sum(floors) / max(len(floors), 1)
    logger.info("Collected %d games: avg_floor=%.1f, trajs=%d, combat=%d (%.0f g/min)",
                len(floors), avg, saved_trajs, saved_combat,
                len(floors) / max((time.monotonic() - t0) / 60, 0.01))
    return avg


def main():
    caffeinate = subprocess.Popen(["caffeinate", "-dims"], stdout=subprocess.DEVNULL)
    shutdown = False

    def on_signal(sig, frame):
        nonlocal shutdown
        shutdown = True
        logger.info("Shutdown requested")
    signal.signal(signal.SIGINT, on_signal)
    signal.signal(signal.SIGTERM, on_signal)

    Path("logs/v3_train_overnight").mkdir(parents=True, exist_ok=True)
    Path("logs/active").unlink(missing_ok=True)
    Path("logs/active").symlink_to(Path("logs/v3_train_overnight").resolve())

    logger.info("=" * 60)
    logger.info("V3 OVERNIGHT TRAINING — train/collect/retrain loop")
    logger.info("=" * 60)

    # Phase 1: CombatNet
    logger.info("PHASE 1: CombatNet")
    combat_data = load_combat_data()
    if combat_data is not None:
        logger.info("Loaded %d combat positions", len(combat_data["obs"]))
        train_combat_net(combat_data, epochs=50)
    else:
        logger.warning("No combat data found")

    # Phase 2: Strategic BC
    logger.info("PHASE 2: Strategic BC (with validation + early stopping)")
    strat_data = load_strategic_data()
    if strat_data is None:
        logger.error("No strategic data — aborting")
        caffeinate.terminate()
        return
    logger.info("Loaded %d transitions from %d files", len(strat_data["obs"]), strat_data["n_files"])
    model = train_strategic_bc(strat_data, epochs=200, val_split=0.15)

    # Iterative loop: collect with model -> retrain on new data
    iteration = 0
    while not shutdown:
        iteration += 1
        logger.info("=" * 60)
        logger.info("ITERATION %d: collect 2000 games -> retrain", iteration)
        logger.info("=" * 60)

        # Phase 3: Collect with current model
        avg_floor = collect_games(model, n_games=2000)

        if shutdown:
            break

        # Phase 4: Retrain on all data (old + new)
        logger.info("Reloading all data...")
        strat_data = load_strategic_data()
        combat_data = load_combat_data()

        if strat_data is not None:
            logger.info("Retrain BC on %d transitions", len(strat_data["obs"]))
            model = train_strategic_bc(strat_data, epochs=200, val_split=0.15)

        if combat_data is not None:
            logger.info("Retrain CombatNet on %d positions", len(combat_data["obs"]))
            train_combat_net(combat_data, epochs=50)

        # Write status
        status = {
            "iteration": iteration,
            "avg_floor": avg_floor,
            "total_transitions": len(strat_data["obs"]) if strat_data else 0,
            "total_combat": len(combat_data["obs"]) if combat_data else 0,
            "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S"),
        }
        Path("logs/v3_train_overnight/status.json").write_text(json.dumps(status, indent=2))

    caffeinate.terminate()
    logger.info("Overnight training complete")


if __name__ == "__main__":
    main()
