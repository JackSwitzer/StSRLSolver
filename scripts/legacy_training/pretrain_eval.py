"""Evaluate a pretrained checkpoint by playing games.

Spins up inference server + worker pool, plays N games, reports stats.

Usage:
    uv run python scripts/pretrain_eval.py [--checkpoint path] [--games 50]
"""

from __future__ import annotations

import argparse
import logging
import multiprocessing as mp
import sys
import time
from pathlib import Path
from typing import List

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [pretrain_eval] %(levelname)s %(message)s",
    handlers=[logging.StreamHandler(sys.stdout)],
)
logger = logging.getLogger("pretrain_eval")


def main():
    parser = argparse.ArgumentParser(description="Evaluate a pretrained checkpoint")
    parser.add_argument("--checkpoint", type=str, default="logs/strategic_checkpoints/latest_strategic.pt")
    parser.add_argument("--games", type=int, default=50, help="Number of eval games")
    parser.add_argument("--workers", type=int, default=8, help="Parallel workers")
    args = parser.parse_args()

    from packages.training.inference_server import InferenceServer
    from packages.training.seed_pool import SeedPool
    from packages.training.strategic_net import StrategicNet
    from packages.training.training_config import SOLVER_BUDGETS, MCTS_COMBAT_ENABLED
    from packages.training.worker import _play_one_game, _worker_init

    ckpt = Path(args.checkpoint)
    if not ckpt.exists():
        logger.error("Checkpoint not found: %s", ckpt)
        sys.exit(1)

    model = StrategicNet.load(ckpt)
    logger.info("Loaded %s (%d params)", ckpt.name, model.param_count())

    server = InferenceServer(n_workers=args.workers, max_batch_size=32, batch_timeout_ms=15.0)
    server.sync_strategic_from_pytorch(model, version=0)
    server.start()

    ctx = mp.get_context("spawn")
    pool = ctx.Pool(
        processes=args.workers,
        initializer=_worker_init,
        initargs=(server.request_q, server.response_qs, server.slot_q, server.shm_info),
    )

    seed_pool = SeedPool()
    solver_ms = SOLVER_BUDGETS["monster"][0]

    t0 = time.monotonic()
    async_results = []
    for i in range(args.games):
        seed = seed_pool.get_seed()
        ar = pool.apply_async(
            _play_one_game,
            (seed, 0, 0.8, 0, solver_ms, False, MCTS_COMBAT_ENABLED, 0),
        )
        async_results.append(ar)

    floors: List[int] = []
    wins = 0
    for i, ar in enumerate(async_results):
        try:
            result = ar.get(timeout=180)
            floors.append(result["floor"])
            if result["won"]:
                wins += 1
            if (i + 1) % 10 == 0:
                logger.info("Progress: %d/%d, avg_floor=%.1f", i + 1, args.games, sum(floors) / len(floors))
        except Exception as e:
            logger.warning("Game %d failed: %s", i, e)

    pool.terminate()
    pool.join()
    server.stop()

    if floors:
        avg = sum(floors) / len(floors)
        peak = max(floors)
        logger.info("Results (%d games, %.1f min): avg=%.1f, peak=%d, wins=%d (%.1f%%)",
                    len(floors), (time.monotonic() - t0) / 60, avg, peak, wins, wins / len(floors) * 100)
    else:
        logger.error("All games failed")


if __name__ == "__main__":
    main()
