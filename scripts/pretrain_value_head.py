"""Pretrain value head on trajectory data (offline, no games needed).

Loads all trajectory .npz files, extracts final_floor and won flag,
trains the value head only (trunk + policy frozen) to predict:
  - final_floor (regression, MSE loss)
  - won (binary, BCE loss via win_loss aux head if enabled)

Uses PopArt normalization from training_config.VALUE_NORM_METHOD.

Usage:
    uv run python scripts/pretrain_value_head.py [--epochs 50] [--batch-size 256]
"""

from __future__ import annotations

import argparse
import logging
import sys
import time
from pathlib import Path

import numpy as np
import torch
import torch.nn.functional as F

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [pretrain_vh] %(levelname)s %(message)s",
    handlers=[logging.StreamHandler(sys.stdout)],
)
logger = logging.getLogger("pretrain_vh")

# Add project root to path
PROJECT_ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(PROJECT_ROOT))


def load_trajectories(
    traj_dir: Path, input_dim: int, max_samples: int = 500_000
) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    """Load observations, final_floor targets, and won flags from .npz files.

    Returns:
        (obs, floors, won) arrays
    """
    traj_files = sorted(traj_dir.glob("traj_F*.npz"), key=lambda p: p.stem, reverse=True)
    if not traj_files:
        logger.warning("No trajectory files found in %s", traj_dir)
        return np.array([]), np.array([]), np.array([])

    obs_list, floor_list, won_list = [], [], []
    loaded = 0

    for tf in traj_files:
        if loaded >= max_samples:
            break
        try:
            data = np.load(tf)
            n = len(data["obs"])
            for i in range(n):
                if loaded >= max_samples:
                    break
                obs_i = data["obs"][i]
                if obs_i.shape[0] != input_dim:
                    continue
                obs_list.append(obs_i)

                # final_floors is already normalized (floor/55.0) in the data
                floor_val = float(data["final_floors"][i]) if "final_floors" in data else 0.0
                floor_list.append(floor_val)

                # Extract won flag: floor >= 55/55 = 1.0 means won
                if "won" in data:
                    won_list.append(float(data["won"][i]) if hasattr(data["won"], "__getitem__") else float(data["won"]))
                else:
                    won_list.append(1.0 if floor_val >= 1.0 else 0.0)
                loaded += 1
        except Exception as e:
            logger.warning("Failed to load %s: %s", tf.name, e)
            continue

    if loaded == 0:
        return np.array([]), np.array([]), np.array([])

    logger.info("Loaded %d samples from %d files", loaded, len(traj_files))
    return (
        np.stack(obs_list).astype(np.float32),
        np.array(floor_list, dtype=np.float32),
        np.array(won_list, dtype=np.float32),
    )


def main():
    parser = argparse.ArgumentParser(description="Pretrain value head on trajectory data")
    parser.add_argument("--epochs", type=int, default=50)
    parser.add_argument("--batch-size", type=int, default=256)
    parser.add_argument("--traj-dir", type=str, default="logs/pretrain_data")
    parser.add_argument("--checkpoint", type=str, default=None,
                        help="Starting checkpoint (default: latest)")
    parser.add_argument("--output", type=str, default="logs/strategic_checkpoints/value_pretrained.pt")
    args = parser.parse_args()

    from packages.training.strategic_net import PopArtLayer, StrategicNet
    from packages.training.training_config import (
        MODEL_ACTION_DIM, VALUE_NORM_METHOD, POPART_BETA, MAX_RETURN,
    )

    # Find best available device
    if torch.backends.mps.is_available():
        device = torch.device("mps")
    elif torch.cuda.is_available():
        device = torch.device("cuda")
    else:
        device = torch.device("cpu")
    logger.info("Device: %s", device)

    # Load or create model
    ckpt_dir = Path("logs/strategic_checkpoints")
    if args.checkpoint:
        ckpt_path = Path(args.checkpoint)
    elif (ckpt_dir / "latest_strategic.pt").exists():
        ckpt_path = ckpt_dir / "latest_strategic.pt"
    else:
        ckpt_path = None

    if ckpt_path and ckpt_path.exists():
        logger.info("Loading checkpoint: %s", ckpt_path)
        model = StrategicNet.load(ckpt_path, device=device)
    else:
        logger.info("No checkpoint found, creating fresh model")
        model = StrategicNet(input_dim=480).to(device)

    # Load trajectory data
    traj_dir = Path(args.traj_dir)
    obs, floors, won = load_trajectories(traj_dir, model.input_dim)
    if len(obs) == 0:
        logger.error("No valid trajectory data found")
        sys.exit(1)

    logger.info("Training data: %d samples, floor range [%.3f, %.3f], win rate %.1f%%",
                len(obs), floors.min(), floors.max(), won.mean() * 100)

    # Convert to tensors
    obs_t = torch.from_numpy(obs).float().to(device)
    floor_t = torch.from_numpy(floors).float().to(device)

    # Create dummy masks
    mask_t = torch.zeros(len(obs), MODEL_ACTION_DIM, dtype=torch.bool, device=device)
    mask_t[:, 0] = True

    # Initialize PopArt
    popart = PopArtLayer(beta=POPART_BETA).to(device)

    # Freeze everything except value head
    for name, param in model.named_parameters():
        if "value" not in name:
            param.requires_grad_(False)

    trainable = sum(p.numel() for p in model.parameters() if p.requires_grad)
    logger.info("Trainable parameters (value head only): %d", trainable)

    optimizer = torch.optim.Adam(
        [p for p in model.parameters() if p.requires_grad],
        lr=3e-4,
    )

    model.train()
    N = len(obs)
    batch_size = args.batch_size
    best_loss = float("inf")
    t0 = time.time()

    for epoch in range(args.epochs):
        indices = torch.randperm(N)
        epoch_loss = 0.0
        n_batches = 0

        for start in range(0, N, batch_size):
            end = min(start + batch_size, N)
            idx = indices[start:end]

            out = model(obs_t[idx], mask_t[idx])
            b_floor = floor_t[idx]

            # Apply normalization
            if VALUE_NORM_METHOD == "popart":
                popart.update(b_floor)
                norm_target = popart.normalize(b_floor)
                value_loss = F.mse_loss(out["value"], norm_target)
            elif VALUE_NORM_METHOD == "clip":
                clipped = b_floor.clamp(0.0, MAX_RETURN) / MAX_RETURN
                value_loss = F.mse_loss(out["value"], clipped)
            else:
                value_loss = F.mse_loss(out["value"], b_floor)

            loss = value_loss

            optimizer.zero_grad()
            loss.backward()
            torch.nn.utils.clip_grad_norm_(model.parameters(), 0.5)
            optimizer.step()

            epoch_loss += loss.item()
            n_batches += 1

        avg_loss = epoch_loss / max(n_batches, 1)
        if (epoch + 1) % 5 == 0 or epoch == 0:
            elapsed = time.time() - t0
            logger.info("Epoch %d/%d: loss=%.4f (%.1fs)", epoch + 1, args.epochs, avg_loss, elapsed)

        if avg_loss < best_loss:
            best_loss = avg_loss

    # Unfreeze all parameters
    for param in model.parameters():
        param.requires_grad_(True)

    # Save checkpoint
    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    model.save(output_path, extra={"popart_state_dict": popart.state_dict()})
    logger.info("Saved value-pretrained checkpoint to %s (best_loss=%.4f)", output_path, best_loss)


if __name__ == "__main__":
    main()
