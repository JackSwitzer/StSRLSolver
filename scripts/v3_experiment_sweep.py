"""V3 experiment sweep: 4 pretrain variants, 2h each, sequential.

Each experiment uses the train/collect/retrain loop from v3_train_overnight
but with different hyperparams. Results compared by avg_floor after each iteration.

A) BC high-LR (3e-4) + deep (200ep) — aggressive learning
B) BC low-LR (3e-5) + long (500ep) — slow and steady
C) BC + PPO fine-tune (BC 50ep then PPO on collected data)
D) IQL offline (no collection, pure offline on all 22k transitions)

Usage: nohup uv run python scripts/v3_experiment_sweep.py > logs/v3_experiments.log 2>&1 &
"""
import json
import logging
import multiprocessing as mp
import shutil
import signal
import subprocess
import sys
import time
from pathlib import Path

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(name)s] %(message)s",
    handlers=[
        logging.StreamHandler(sys.stdout),
        logging.FileHandler("logs/v3_experiments.log"),
    ],
)
logger = logging.getLogger("v3_exp")
logging.getLogger("packages.training.turn_solver").setLevel(logging.ERROR)


# ──────────────────────────────────────────────────
# Data loading (shared across experiments)
# ──────────────────────────────────────────────────

def load_strategic_data(extra_dirs=None, max_transitions=200_000):
    obs_list, mask_list, action_list, floor_list = [], [], [], []
    loaded = 0
    files = []
    seen = set()
    search_dirs = list(Path("logs").rglob("best_trajectories/traj_F*.npz"))
    search_dirs += list(Path("logs").rglob("all_trajectories/traj_*.npz"))
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
        "obs": np.concatenate(obs_list), "masks": np.concatenate(mask_list),
        "actions": np.concatenate(action_list).astype(np.int64),
        "floors": np.concatenate(floor_list), "n_files": len(files),
    }


def load_combat_data():
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


# ──────────────────────────────────────────────────
# Training functions
# ──────────────────────────────────────────────────

def make_model(device):
    from packages.training.training_config import MODEL_HIDDEN_DIM, MODEL_NUM_BLOCKS, MODEL_ACTION_DIM
    from packages.training.strategic_net import StrategicNet
    return StrategicNet(input_dim=480, hidden_dim=MODEL_HIDDEN_DIM,
                        action_dim=MODEL_ACTION_DIM, num_blocks=MODEL_NUM_BLOCKS).to(device)


def train_bc(data, device, lr=3e-4, max_epochs=200, patience=20, batch_size=2048, label=""):
    from packages.training.training_config import LR_HEAD_MULTIPLIERS
    model = make_model(device)
    N = len(data["obs"])
    obs_t = torch.from_numpy(data["obs"]).float().to(device)
    mask_t = torch.from_numpy(data["masks"]).bool().to(device)
    action_t = torch.from_numpy(data["actions"]).long().to(device)
    floor_t = torch.from_numpy(data["floors"]).float().to(device)

    perm = torch.randperm(N)
    split = int(N * 0.85)
    train_idx, val_idx = perm[:split], perm[split:]

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
    scheduler = torch.optim.lr_scheduler.CosineAnnealingLR(optimizer, T_max=max_epochs, eta_min=1e-6)

    best_val = float("inf")
    no_improve = 0
    best_state = None

    model.train()
    for epoch in range(1, max_epochs + 1):
        idx = train_idx[torch.randperm(len(train_idx))]
        for start in range(0, len(idx), batch_size):
            batch = idx[start:start + batch_size]
            out = model(obs_t[batch], mask_t[batch])
            bc_loss = F.nll_loss(F.log_softmax(out["policy_logits"], dim=-1), action_t[batch])
            v_loss = F.mse_loss(out["value"], floor_t[batch])
            loss = bc_loss + 0.5 * v_loss
            optimizer.zero_grad()
            loss.backward()
            nn.utils.clip_grad_norm_(model.parameters(), 1.0)
            optimizer.step()
        scheduler.step()

        model.eval()
        with torch.no_grad():
            out = model(obs_t[val_idx], mask_t[val_idx])
            val_bc = F.nll_loss(F.log_softmax(out["policy_logits"], dim=-1), action_t[val_idx]).item()
            val_acc = (out["policy_logits"].argmax(dim=-1) == action_t[val_idx]).float().mean().item() * 100
        model.train()

        if val_bc < best_val:
            best_val = val_bc
            no_improve = 0
            best_state = {k: v.cpu().clone() for k, v in model.state_dict().items()}
        else:
            no_improve += 1

        if epoch % 20 == 0 or epoch <= 3 or no_improve >= patience:
            logger.info("[%s] BC %d/%d: val_loss=%.4f, val_acc=%.1f%% (patience=%d/%d)",
                        label, epoch, max_epochs, val_bc, val_acc, no_improve, patience)

        if no_improve >= patience:
            logger.info("[%s] Early stop at epoch %d", label, epoch)
            break

    if best_state:
        model.load_state_dict(best_state)
        model.to(device)
    return model, {"val_loss": best_val, "val_acc": val_acc, "epochs": epoch}


def collect_games(model, n_games, device, label=""):
    from packages.training.inference_server import InferenceServer
    from packages.training.seed_pool import SeedPool
    from packages.training.training_config import SOLVER_BUDGETS, MCTS_COMBAT_ENABLED
    from packages.training.worker import _play_one_game, _worker_init

    # Config-driven solver budget (adapter overrides per room type at runtime)
    _default_solver_ms = SOLVER_BUDGETS["monster"][0]  # 50ms base for monsters

    N_W = 10
    server = InferenceServer(n_workers=N_W, max_batch_size=32, batch_timeout_ms=15.0)
    server.sync_strategic_from_pytorch(model, version=0)
    server.start()
    ctx = mp.get_context("spawn")
    shm = getattr(server, "shm_info", None)
    pool = ctx.Pool(processes=N_W, initializer=_worker_init,
                    initargs=(server.request_q, server.response_qs, server.slot_q, shm))

    sp = SeedPool()
    traj_dir = Path(f"logs/v3_exp_{label}/trajectories")
    combat_dir = Path(f"logs/v3_exp_{label}/combat_data")
    traj_dir.mkdir(parents=True, exist_ok=True)
    combat_dir.mkdir(parents=True, exist_ok=True)

    floors, saved_t, saved_c = [], 0, 0
    ars = [(sp.get_seed(), pool.apply_async(_play_one_game, (sp.get_seed(), 0, 0.8, 0, _default_solver_ms, False, MCTS_COMBAT_ENABLED, 0)))
           for _ in range(n_games)]

    for seed, ar in ars:
        try:
            r = ar.get(timeout=120)
        except Exception:
            continue
        floors.append(r["floor"])
        trans = r.get("transitions", [])
        if trans:
            saved_t += 1
            obs = np.stack([t["obs"] for t in trans])
            masks = np.stack([t["action_mask"] for t in trans])
            np.savez_compressed(
                traj_dir / f"traj_{saved_t:06d}_F{r['floor']:02d}.npz",
                obs=obs, masks=masks,
                actions=np.array([t["action"] for t in trans], dtype=np.int32),
                rewards=np.array([t["reward"] for t in trans], dtype=np.float32),
                dones=np.array([t["done"] for t in trans], dtype=bool),
                values=np.array([t["value"] for t in trans], dtype=np.float32),
                log_probs=np.array([t["log_prob"] for t in trans], dtype=np.float32),
                final_floors=np.array([t["final_floor"] for t in trans], dtype=np.float32),
                cleared_act1=np.array([t["cleared_act1"] for t in trans], dtype=np.float32),
            )
        for c in r.get("combats", []):
            vec = c.get("combat_state_vector")
            if vec is not None:
                saved_c += 1
                survived = r["floor"] > c.get("floor", 0)
                np.savez_compressed(combat_dir / f"combat_{saved_c:06d}.npz",
                                    combat_obs=vec, won=np.array(survived, dtype=bool))

    pool.terminate(); pool.join(); server.stop()
    avg = sum(floors) / max(len(floors), 1)
    logger.info("[%s] Collected %d games: avg_floor=%.1f, trajs=%d", label, len(floors), avg, saved_t)
    return avg, traj_dir


def eval_model(model, n_games, device, label=""):
    """Quick eval without saving data."""
    from packages.training.inference_server import InferenceServer
    from packages.training.seed_pool import SeedPool
    from packages.training.training_config import SOLVER_BUDGETS, MCTS_COMBAT_ENABLED
    from packages.training.worker import _play_one_game, _worker_init

    # Config-driven solver budget
    _default_solver_ms = SOLVER_BUDGETS["monster"][0]

    N_W = 10
    server = InferenceServer(n_workers=N_W, max_batch_size=32, batch_timeout_ms=15.0)
    server.sync_strategic_from_pytorch(model, version=0)
    server.start()
    ctx = mp.get_context("spawn")
    shm = getattr(server, "shm_info", None)
    pool = ctx.Pool(processes=N_W, initializer=_worker_init,
                    initargs=(server.request_q, server.response_qs, server.slot_q, shm))
    sp = SeedPool()
    ars = [pool.apply_async(_play_one_game, (sp.get_seed(), 0, 0.5, 0, _default_solver_ms, False, MCTS_COMBAT_ENABLED, 0))
           for _ in range(n_games)]
    floors = []
    for ar in ars:
        try:
            floors.append(ar.get(timeout=120)["floor"])
        except Exception:
            pass
    pool.terminate(); pool.join(); server.stop()
    avg = sum(floors) / max(len(floors), 1)
    logger.info("[%s] Eval %d games: avg=%.1f, peak=%d", label, len(floors), avg, max(floors) if floors else 0)
    return avg


# ──────────────────────────────────────────────────
# Experiments (2h each)
# ──────────────────────────────────────────────────

def experiment_A(data, device, time_limit):
    """High-LR BC (3e-4) with train/collect/retrain loop."""
    label = "A_high_lr"
    t0 = time.monotonic()
    model, metrics = train_bc(data, device, lr=3e-4, max_epochs=200, patience=20, label=label)
    logger.info("[%s] Initial BC: %s", label, metrics)

    iteration = 0
    while time.monotonic() - t0 < time_limit:
        iteration += 1
        avg, traj_dir = collect_games(model, 1000, device, label=f"{label}_i{iteration}")
        new_data = load_strategic_data(extra_dirs=[str(traj_dir)])
        if new_data:
            model, metrics = train_bc(new_data, device, lr=3e-4, max_epochs=200, patience=20, label=f"{label}_i{iteration}")
            logger.info("[%s] Iter %d: %s, collect_floor=%.1f", label, iteration, metrics, avg)

    final_floor = eval_model(model, 100, device, label)
    model.save(Path(f"logs/strategic_checkpoints/exp_{label}.pt"))
    return {"label": label, "final_floor": final_floor, "iterations": iteration, **metrics}


def experiment_B(data, device, time_limit):
    """Low-LR BC (3e-5) with more patience — slow and steady."""
    label = "B_low_lr"
    t0 = time.monotonic()
    model, metrics = train_bc(data, device, lr=3e-5, max_epochs=500, patience=50, label=label)
    logger.info("[%s] Initial BC: %s", label, metrics)

    iteration = 0
    while time.monotonic() - t0 < time_limit:
        iteration += 1
        avg, traj_dir = collect_games(model, 1000, device, label=f"{label}_i{iteration}")
        new_data = load_strategic_data(extra_dirs=[str(traj_dir)])
        if new_data:
            model, metrics = train_bc(new_data, device, lr=3e-5, max_epochs=500, patience=50, label=f"{label}_i{iteration}")
            logger.info("[%s] Iter %d: %s, collect_floor=%.1f", label, iteration, metrics, avg)

    final_floor = eval_model(model, 100, device, label)
    model.save(Path(f"logs/strategic_checkpoints/exp_{label}.pt"))
    return {"label": label, "final_floor": final_floor, "iterations": iteration, **metrics}


def experiment_C(data, device, time_limit):
    """BC warmup (50ep) then PPO fine-tune on collected data."""
    label = "C_bc_ppo"
    t0 = time.monotonic()

    # Quick BC warmup
    model, bc_metrics = train_bc(data, device, lr=3e-4, max_epochs=50, patience=15, label=f"{label}_bc")
    logger.info("[%s] BC warmup: %s", label, bc_metrics)

    # PPO fine-tune loop
    from packages.training.strategic_trainer import StrategicTrainer
    trainer = StrategicTrainer(model=model, lr=3e-5, batch_size=256,
                               entropy_coeff=0.10, ppo_epochs=4)

    iteration = 0
    while time.monotonic() - t0 < time_limit:
        iteration += 1
        # Collect
        avg, traj_dir = collect_games(model, 1000, device, label=f"{label}_i{iteration}")

        # Load new transitions into PPO buffer
        trainer.buffer.clear()
        ep_id = 0
        for tf in sorted(traj_dir.glob("traj_*.npz")):
            try:
                d = np.load(tf)
                ep_id += 1
                for i in range(len(d["obs"])):
                    trainer.add_transition(
                        obs=d["obs"][i], action_mask=d["masks"][i],
                        action=int(d["actions"][i]), reward=float(d["rewards"][i]),
                        done=bool(d["dones"][i]), value=float(d["values"][i]),
                        log_prob=float(d["log_probs"][i]), episode_id=ep_id,
                    )
                    buf_t = trainer.buffer[-1]
                    buf_t.final_floor = float(d["final_floors"][i])
            except Exception:
                continue

        # Train PPO
        if len(trainer.buffer) >= 256:
            for _ in range(10):
                m = trainer.train_batch()
            logger.info("[%s] Iter %d: PPO loss=%.4f, clip=%.3f, ent=%.3f, floor=%.1f",
                        label, iteration, m.get("total_loss", 0),
                        m.get("clip_fraction", 0), m.get("entropy", 0), avg)

    final_floor = eval_model(model, 100, device, label)
    model.save(Path(f"logs/strategic_checkpoints/exp_{label}.pt"))
    return {"label": label, "final_floor": final_floor, "iterations": iteration}


def experiment_D(data, device, time_limit):
    """IQL offline — no collection, pure offline training."""
    label = "D_iql"
    t0 = time.monotonic()

    from packages.training.iql_trainer import IQLTrainer
    from packages.training.offline_data import OfflineDataset

    model = make_model(device)

    # Build offline dataset
    N = len(data["obs"])
    next_obs = np.zeros_like(data["obs"])
    next_obs[:-1] = data["obs"][1:]
    dones = np.zeros(N, dtype=bool)
    rewards = data["floors"]  # Use floor as reward signal

    ds = OfflineDataset(
        states=data["obs"], actions=data["actions"],
        rewards=rewards, next_states=next_obs,
        dones=dones, action_masks=data["masks"],
    )

    iql = IQLTrainer(policy=model, input_dim=480, action_dim=model.action_dim, lr=3e-4)

    # Train for the full time limit
    epoch = 0
    while time.monotonic() - t0 < time_limit:
        epoch += 1
        metrics = iql.train_offline(ds, epochs=10)
        logger.info("[%s] Epoch group %d: %s", label, epoch,
                    {k: round(v, 4) if isinstance(v, float) else v for k, v in metrics.items()})

    final_floor = eval_model(model, 100, device, label)
    model.save(Path(f"logs/strategic_checkpoints/exp_{label}.pt"))
    return {"label": label, "final_floor": final_floor, "epochs": epoch * 10}


# ──────────────────────────────────────────────────
# Main
# ──────────────────────────────────────────────────

def main():
    caffeinate = subprocess.Popen(["caffeinate", "-dims"], stdout=subprocess.DEVNULL)
    shutdown = False

    def on_signal(sig, frame):
        nonlocal shutdown
        shutdown = True
    signal.signal(signal.SIGINT, on_signal)
    signal.signal(signal.SIGTERM, on_signal)

    device = torch.device("mps") if torch.backends.mps.is_available() else torch.device("cpu")
    TIME_LIMIT = 2 * 3600  # 2 hours per experiment

    Path("logs/v3_experiments").mkdir(parents=True, exist_ok=True)

    logger.info("=" * 60)
    logger.info("V3 EXPERIMENT SWEEP: 4 variants x 2h each")
    logger.info("=" * 60)

    # Load base data once
    data = load_strategic_data()
    if data is None:
        logger.error("No strategic data!")
        caffeinate.terminate()
        return
    logger.info("Base data: %d transitions from %d files", len(data["obs"]), data["n_files"])

    # Train CombatNet once (shared across experiments)
    combat_data = load_combat_data()
    if combat_data is not None:
        logger.info("CombatNet: %d positions", len(combat_data["obs"]))
        from packages.training.combat_net import CombatNet
        cnet = CombatNet(input_dim=298, hidden_dim=256, num_layers=3).to(device)
        obs_t = torch.from_numpy(combat_data["obs"]).float().to(device)
        won_t = torch.from_numpy(combat_data["won"].astype(np.float32)).to(device)
        opt = torch.optim.Adam(cnet.parameters(), lr=1e-3, weight_decay=1e-4)
        cnet.train()
        for ep in range(30):
            for s in range(0, len(obs_t), 2048):
                pred = cnet(obs_t[s:s+2048]).squeeze(-1)
                loss = F.binary_cross_entropy(pred, won_t[s:s+2048])
                opt.zero_grad(); loss.backward(); opt.step()
        cnet.save(Path("logs/active/combat_net.pt"))
        logger.info("CombatNet trained and saved")

    # Run experiments
    results = []
    experiments = [
        ("A", "High-LR BC (3e-4) + collect/retrain", experiment_A),
        ("B", "Low-LR BC (3e-5) + slow/steady", experiment_B),
        ("C", "BC warmup + PPO fine-tune", experiment_C),
        ("D", "IQL offline (no collection)", experiment_D),
    ]

    for tag, desc, fn in experiments:
        if shutdown:
            break
        logger.info("=" * 60)
        logger.info("EXPERIMENT %s: %s (2h)", tag, desc)
        logger.info("=" * 60)
        try:
            result = fn(data, device, TIME_LIMIT)
            results.append(result)
            logger.info("[%s] RESULT: %s", tag, result)
        except Exception as e:
            logger.error("[%s] FAILED: %s", tag, e, exc_info=True)
            results.append({"label": tag, "error": str(e)})

    # Summary
    logger.info("=" * 60)
    logger.info("FINAL RESULTS")
    logger.info("=" * 60)
    for r in results:
        logger.info("  %s: floor=%.1f %s",
                     r.get("label", "?"), r.get("final_floor", 0),
                     {k: v for k, v in r.items() if k not in ("label", "final_floor")})

    # Save results
    Path("logs/v3_experiments/results.json").write_text(json.dumps(results, indent=2, default=str))

    # Pick winner
    best = max(results, key=lambda r: r.get("final_floor", 0))
    logger.info("WINNER: %s (floor=%.1f)", best.get("label"), best.get("final_floor", 0))
    winner_ckpt = Path(f"logs/strategic_checkpoints/exp_{best['label']}.pt")
    if winner_ckpt.exists():
        shutil.copy2(winner_ckpt, "logs/strategic_checkpoints/bc_winner_v3.pt")
        shutil.copy2(winner_ckpt, "logs/strategic_checkpoints/latest_strategic.pt")

    caffeinate.terminate()
    logger.info("Experiment sweep complete")


if __name__ == "__main__":
    main()
