"""
Overnight training runner with scheduling and hyperparameter sweep.

Manages long-running training sessions with:
- Time-based headless/visual mode switching
- Hyperparameter sweep scheduling
- Status file writing for monitoring
- Integration with StrategicTrainer + combat self-play
"""

from __future__ import annotations

import json
import logging
import time
from collections import deque
from datetime import datetime
from pathlib import Path
from typing import Any, Deque, Dict, List, Optional

import numpy as np

logger = logging.getLogger(__name__)

DEFAULT_SWEEP_CONFIGS = [
    {"lr": 3e-4, "batch_size": 32, "mcts_sims": 32, "entropy_coeff": 0.01},
    {"lr": 1e-4, "batch_size": 64, "mcts_sims": 32, "entropy_coeff": 0.01},
    {"lr": 3e-4, "batch_size": 32, "mcts_sims": 64, "entropy_coeff": 0.02},
    {"lr": 1e-4, "batch_size": 64, "mcts_sims": 64, "entropy_coeff": 0.005},
    {"lr": 5e-5, "batch_size": 128, "mcts_sims": 32, "entropy_coeff": 0.01},
    {"lr": 3e-4, "batch_size": 32, "mcts_sims": 16, "entropy_coeff": 0.03},
    {"lr": 1e-4, "batch_size": 32, "mcts_sims": 48, "entropy_coeff": 0.01},
    {"lr": 5e-4, "batch_size": 64, "mcts_sims": 32, "entropy_coeff": 0.01},
]


class OvernightRunner:
    """Manages overnight training with scheduling and sweep.

    Config keys:
        headless_after_min: Minutes after start to go headless (default 30)
        visual_at: HH:MM time to switch back to visual mode (default "07:30")
        sweep_configs: List of hyperparameter dicts to sweep
        run_dir: Directory for logs and checkpoints
        max_games: Maximum total games to play (default 50000)
        games_per_batch: Games per training batch (default 16)
        workers: Number of parallel workers (default 8)
        ascension: Ascension level (default 0 for initial training)
        eval_every: Games between evaluation runs (default 500)
    """

    def __init__(self, config: Dict[str, Any]):
        self.headless_after_min = config.get("headless_after_min", 30)
        self.visual_at = config.get("visual_at", "07:30")
        self.sweep_configs = config.get("sweep_configs", DEFAULT_SWEEP_CONFIGS)
        self.run_dir = Path(config.get("run_dir", "logs/overnight"))
        self.max_games = config.get("max_games", 50000)
        self.games_per_batch = config.get("games_per_batch", 16)
        self.workers = config.get("workers", 8)
        self.ascension = config.get("ascension", 0)
        self.eval_every = config.get("eval_every", 500)

        self.run_dir.mkdir(parents=True, exist_ok=True)
        self._start_time = time.monotonic()
        self._start_datetime = datetime.now()
        self._current_sweep_idx = 0
        self._games_per_sweep = self.max_games // max(len(self.sweep_configs), 1)

        # Stats tracking
        self.total_games = 0
        self.total_wins = 0
        self.recent_floors: Deque[int] = deque(maxlen=100)
        self.recent_wins: Deque[bool] = deque(maxlen=100)
        self.sweep_results: List[Dict[str, Any]] = []

    def should_be_headless(self) -> bool:
        """Check if we should be in headless mode based on schedule."""
        elapsed_min = (time.monotonic() - self._start_time) / 60.0
        if elapsed_min < self.headless_after_min:
            return False

        # Check if we've hit the visual_at time
        now = datetime.now()
        try:
            hour, minute = map(int, self.visual_at.split(":"))
            if now.hour == hour and now.minute >= minute:
                return False
            if now.hour > hour:
                return False
        except (ValueError, AttributeError):
            pass

        return True

    def get_current_sweep_config(self) -> Optional[Dict[str, Any]]:
        """Return current hyperparameter config from sweep schedule."""
        if not self.sweep_configs:
            return None
        if self._current_sweep_idx >= len(self.sweep_configs):
            return None
        return self.sweep_configs[self._current_sweep_idx]

    def _advance_sweep(self) -> bool:
        """Advance to next sweep config. Returns False if sweep is done."""
        self._current_sweep_idx += 1
        return self._current_sweep_idx < len(self.sweep_configs)

    def write_status(self, stats: Dict[str, Any]) -> None:
        """Write status.json for monitoring."""
        status = {
            "timestamp": datetime.now().isoformat(),
            "elapsed_hours": round((time.monotonic() - self._start_time) / 3600, 2),
            "total_games": self.total_games,
            "total_wins": self.total_wins,
            "win_rate_100": round(sum(self.recent_wins) / max(len(self.recent_wins), 1) * 100, 1),
            "avg_floor_100": round(sum(self.recent_floors) / max(len(self.recent_floors), 1), 1),
            "current_sweep": self._current_sweep_idx,
            "total_sweeps": len(self.sweep_configs),
            "headless": self.should_be_headless(),
            **stats,
        }
        status_path = self.run_dir / "status.json"
        status_path.write_text(json.dumps(status, indent=2))

    def _record_game(self, won: bool, floor: int) -> None:
        """Record a game result."""
        self.total_games += 1
        if won:
            self.total_wins += 1
        self.recent_floors.append(floor)
        self.recent_wins.append(won)

    def run(self) -> Dict[str, Any]:
        """Main overnight loop.

        Integrates with StrategicTrainer to train the strategic model
        while using the combat solver for combat phases.
        """
        import torch
        from .strategic_net import StrategicNet, _get_device
        from .strategic_trainer import StrategicTrainer
        from .state_encoder_v2 import RunStateEncoder
        from .self_play import SeedPool

        device = _get_device()
        encoder = RunStateEncoder()

        # Initialize model
        model = StrategicNet(input_dim=encoder.RUN_DIM).to(device)
        logger.info(
            "Strategic model: %d parameters, device=%s",
            model.param_count(), device,
        )

        seed_pool = SeedPool(max_plays=5)
        best_avg_floor = 0.0

        for sweep_idx, sweep_config in enumerate(self.sweep_configs):
            self._current_sweep_idx = sweep_idx
            lr = sweep_config.get("lr", 1e-4)
            trainer = StrategicTrainer(
                model=model,
                lr=lr,
                entropy_coeff=sweep_config.get("entropy_coeff", 0.01),
                clip_epsilon=sweep_config.get("clip_epsilon", 0.2),
                batch_size=sweep_config.get("batch_size", 32),
            )

            sweep_games = 0
            sweep_start = time.monotonic()

            logger.info(
                "Sweep %d/%d: lr=%.1e, ent=%.3f, clip=%.2f",
                sweep_idx + 1, len(self.sweep_configs),
                lr,
                sweep_config.get("entropy_coeff", 0.01),
                sweep_config.get("clip_epsilon", 0.2),
            )

            while sweep_games < self._games_per_sweep and self.total_games < self.max_games:
                batch_results = self._play_batch(
                    model, encoder, seed_pool, trainer,
                )

                for result in batch_results:
                    self._record_game(result["won"], result["floor"])
                    sweep_games += 1

                # Train if enough transitions
                if len(trainer.buffer) >= trainer.batch_size:
                    metrics = trainer.train_batch()
                    trainer.decay_entropy()

                    # Logging
                    avg_floor = sum(self.recent_floors) / max(len(self.recent_floors), 1)
                    wr = sum(self.recent_wins) / max(len(self.recent_wins), 1)

                    logger.info(
                        "Games %d | WR %.1f%% | Floor %.1f | Loss %.4f | Trans %d",
                        self.total_games, wr * 100, avg_floor,
                        metrics.get("total_loss", 0), metrics.get("num_transitions", 0),
                    )

                    # Checkpoint on improvement
                    if trainer.maybe_checkpoint(avg_floor):
                        best_avg_floor = avg_floor
                        logger.info("New best avg floor: %.1f", avg_floor)

                # Write status
                self.write_status({
                    "sweep_config": sweep_config,
                    "sweep_games": sweep_games,
                    "train_steps": trainer.train_steps,
                })

            # Record sweep results
            sweep_elapsed = time.monotonic() - sweep_start
            avg_floor = sum(self.recent_floors) / max(len(self.recent_floors), 1)
            self.sweep_results.append({
                "config": sweep_config,
                "games": sweep_games,
                "avg_floor": round(avg_floor, 1),
                "win_rate": round(sum(self.recent_wins) / max(len(self.recent_wins), 1) * 100, 1),
                "duration_min": round(sweep_elapsed / 60, 1),
                "train_steps": trainer.train_steps,
            })

        # Final save
        model.save(self.run_dir / "final_strategic.pt")
        self._write_summary()

        return {
            "total_games": self.total_games,
            "total_wins": self.total_wins,
            "best_avg_floor": best_avg_floor,
            "sweep_results": self.sweep_results,
        }

    def _play_batch(
        self,
        model,
        encoder,
        seed_pool,
        trainer,
    ) -> List[Dict[str, Any]]:
        """Play a batch of games, collecting strategic transitions.

        Uses the existing self_play worker for combat, but intercepts
        non-combat decision points for strategic training.
        """
        from packages.engine.game import GameRunner, GamePhase
        from packages.training.planner import StrategicPlanner

        import torch
        import torch.nn.functional as F

        results = []
        planner = StrategicPlanner()

        for _ in range(self.games_per_batch):
            seed = seed_pool.get_seed()
            try:
                runner = GameRunner(seed=seed, ascension=self.ascension, character="Watcher", verbose=False)
            except Exception:
                continue

            step = 0
            prev_floor = 0
            hp_before_combat = runner.run_state.current_hp
            game_transitions_start = len(trainer.buffer)

            while not runner.game_over and step < 5000:
                try:
                    actions = runner.get_available_actions()
                except Exception:
                    break
                if not actions:
                    break

                phase = runner.phase
                rs = runner.run_state
                current_floor = getattr(rs, "floor", 0)

                if phase == GamePhase.COMBAT:
                    # Combat: use heuristic planner (combat solver handles this)
                    runner.take_action(actions[0])
                elif len(actions) == 1:
                    runner.take_action(actions[0])
                else:
                    # Strategic decision point: encode, predict, record
                    run_obs = encoder.encode(rs)

                    # Build action mask (simple: one slot per action index)
                    n_actions = len(actions)
                    mask = np.zeros(model.action_dim, dtype=np.bool_)
                    mask[:n_actions] = True

                    # Forward pass
                    with torch.no_grad():
                        obs_t = torch.from_numpy(run_obs).float().unsqueeze(0)
                        mask_t = torch.from_numpy(mask).bool().unsqueeze(0)
                        device = next(model.parameters()).device
                        obs_t = obs_t.to(device)
                        mask_t = mask_t.to(device)

                        out = model(obs_t, mask_t)
                        logits = out["policy_logits"]
                        value = out["value"].item()
                        probs = F.softmax(logits, dim=-1)
                        dist = torch.distributions.Categorical(probs)
                        action_idx = dist.sample().item()
                        log_prob = dist.log_prob(torch.tensor(action_idx, device=device)).item()

                    # Clamp to valid range
                    action_idx = min(action_idx, n_actions - 1)

                    # Compute reward
                    reward = 0.0
                    if current_floor > prev_floor:
                        reward += STRATEGIC_REWARDS["floor_cleared"]

                    # Record transition
                    trainer.add_transition(
                        obs=run_obs,
                        action_mask=mask,
                        action=action_idx,
                        reward=reward,
                        done=False,
                        value=value,
                        log_prob=log_prob,
                    )

                    runner.take_action(actions[action_idx])

                step += 1
                prev_floor = current_floor

            # Game ended — backfill
            rs = runner.run_state
            won = runner.game_won
            final_floor = getattr(rs, "floor", 0)
            cleared_acts = [
                final_floor >= 17,  # act 1 boss ~floor 17
                final_floor >= 34,  # act 2 boss ~floor 34
                final_floor >= 51,  # act 3 boss ~floor 51
            ]

            # Terminal reward on last transition
            game_transitions = trainer.buffer[game_transitions_start:]
            if game_transitions:
                if won:
                    game_transitions[-1].reward += STRATEGIC_REWARDS["game_win"]
                else:
                    progress = final_floor / 55.0
                    game_transitions[-1].reward += STRATEGIC_REWARDS["game_loss_base"] * (1 - progress)
                game_transitions[-1].done = True

            # Backfill aux targets for this game's transitions
            for t in game_transitions:
                t.final_floor = final_floor / 55.0
                t.cleared_act1 = float(cleared_acts[0])
                t.cleared_act2 = float(cleared_acts[1])
                t.cleared_act3 = float(cleared_acts[2])

            results.append({
                "seed": seed,
                "won": won,
                "floor": final_floor,
                "hp": getattr(rs, "current_hp", 0),
            })
            seed_pool.record_result(seed, {"won": won, "floor": final_floor})

        return results

    def _write_summary(self) -> None:
        """Write a summary of the overnight run."""
        summary = {
            "start": self._start_datetime.isoformat(),
            "end": datetime.now().isoformat(),
            "elapsed_hours": round((time.monotonic() - self._start_time) / 3600, 2),
            "total_games": self.total_games,
            "total_wins": self.total_wins,
            "final_win_rate": round(self.total_wins / max(self.total_games, 1) * 100, 1),
            "final_avg_floor": round(sum(self.recent_floors) / max(len(self.recent_floors), 1), 1),
            "sweep_results": self.sweep_results,
        }
        (self.run_dir / "summary.json").write_text(json.dumps(summary, indent=2))
        logger.info("Overnight run complete. Summary written to %s", self.run_dir / "summary.json")


# Import for use by overnight.py _play_batch
STRATEGIC_REWARDS = {
    "floor_cleared": 0.01,
    "normal_kill": 0.05,
    "elite_kill": 0.15,
    "boss_kill": 0.40,
    "game_win": 1.0,
    "game_loss_base": -0.3,
    "hp_efficiency_scale": 0.05,
}


def main():
    """CLI entry point for overnight training."""
    import argparse

    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s | %(levelname)s | %(message)s",
        datefmt="%H:%M:%S",
    )

    parser = argparse.ArgumentParser(description="Overnight training runner")
    parser.add_argument("--workers", type=int, default=8, help="Number of parallel workers")
    parser.add_argument("--games", type=int, default=50000, help="Maximum total games")
    parser.add_argument("--batch", type=int, default=16, help="Games per batch")
    parser.add_argument("--ascension", type=int, default=0, help="Ascension level")
    parser.add_argument("--run-dir", type=str, default="logs/overnight", help="Output directory")
    parser.add_argument("--headless-after", type=int, default=30, help="Go headless after N minutes")
    parser.add_argument("--visual-at", type=str, default="07:30", help="Switch to visual at HH:MM")
    args = parser.parse_args()

    runner = OvernightRunner({
        "workers": args.workers,
        "max_games": args.games,
        "games_per_batch": args.batch,
        "ascension": args.ascension,
        "run_dir": args.run_dir,
        "headless_after_min": args.headless_after,
        "visual_at": args.visual_at,
    })

    result = runner.run()
    logger.info(
        "Done: %d games, %d wins (%.1f%%), best floor %.1f",
        result["total_games"], result["total_wins"],
        result["total_wins"] / max(result["total_games"], 1) * 100,
        result["best_avg_floor"],
    )


if __name__ == "__main__":
    main()
