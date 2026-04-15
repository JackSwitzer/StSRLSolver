"""CombatNet pretrain: train combat outcome predictor on collected positions.

Uses combat_net.train_combat_net() — no reimplementation.

Usage:
    uv run python scripts/pretrain_combat.py [--epochs 50]
"""

from __future__ import annotations

import argparse
import logging
import sys
import time
from pathlib import Path
from typing import Any, Dict, List

import numpy as np

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [pretrain_combat] %(levelname)s %(message)s",
    handlers=[logging.StreamHandler(sys.stdout)],
)
logger = logging.getLogger("pretrain_combat")


def main():
    parser = argparse.ArgumentParser(description="Train CombatNet on combat positions")
    parser.add_argument("--epochs", type=int, default=50, help="Training epochs")
    args = parser.parse_args()

    from packages.training.combat_net import train_combat_net
    from packages.training.strategic_net import _get_device

    device = _get_device()

    # Gather combat .npz files from all runs
    games_data: List[Dict[str, Any]] = []
    for combat_dir in Path("logs").rglob("combat_data"):
        for f in combat_dir.glob("combat_*.npz"):
            try:
                data = np.load(f)
                games_data.append({"combat_obs": data["combat_obs"], "won": bool(data["won"])})
            except Exception as e:
                logger.warning("Failed to load %s: %s", f.name, e)

    if len(games_data) < 50:
        logger.error("Only %d combat positions — need more data. Run training first.", len(games_data))
        sys.exit(1)

    logger.info("Training CombatNet on %d positions", len(games_data))
    t0 = time.monotonic()
    model, metrics = train_combat_net(games_data, epochs=args.epochs, device=device)
    logger.info("Done (%.1f min): loss=%.4f, acc=%.1f%%",
                (time.monotonic() - t0) / 60, metrics.get("loss", 0), metrics.get("accuracy", 0))

    # Save
    save_path = Path("logs/strategic_checkpoints/combat_net_v4.pt")
    save_path.parent.mkdir(parents=True, exist_ok=True)
    model.save(save_path)
    logger.info("Saved: %s", save_path)


if __name__ == "__main__":
    main()
