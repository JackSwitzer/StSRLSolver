"""
Overnight training runner with scheduling and hyperparameter sweep.

Manages long-running training sessions with:
- Time-based headless/visual mode switching
- Hyperparameter sweep scheduling
- Status file writing for monitoring
- Integration with StrategicTrainer + combat self-play
- Multiprocessing for parallel game execution (Phase 2A)

Phase 2A changes (2026-03-12):
- ProcessPoolExecutor for parallel game execution (8 workers -> 15+ games/min)
- PBRS (Potential-Based Reward Shaping) for dense rewards
- episodes.jsonl logging per game
- --batch-size CLI arg for PPO mini-batch size
- Temperature-based exploration during training
"""

from __future__ import annotations

import json
import logging
import time
from collections import deque
from concurrent.futures import ProcessPoolExecutor, as_completed
from datetime import datetime
from pathlib import Path
from typing import Any, Deque, Dict, List, Optional

import numpy as np

logger = logging.getLogger(__name__)

DEFAULT_SWEEP_CONFIGS = [
    {"lr": 3e-4, "batch_size": 256, "mcts_sims": 32, "entropy_coeff": 0.05},
    {"lr": 1e-4, "batch_size": 256, "mcts_sims": 32, "entropy_coeff": 0.05},
    {"lr": 3e-4, "batch_size": 256, "mcts_sims": 64, "entropy_coeff": 0.03},
    {"lr": 1e-4, "batch_size": 512, "mcts_sims": 64, "entropy_coeff": 0.03},
    {"lr": 5e-5, "batch_size": 256, "mcts_sims": 32, "entropy_coeff": 0.05},
    {"lr": 3e-4, "batch_size": 256, "mcts_sims": 16, "entropy_coeff": 0.05},
    {"lr": 1e-4, "batch_size": 256, "mcts_sims": 48, "entropy_coeff": 0.03},
    {"lr": 5e-4, "batch_size": 256, "mcts_sims": 32, "entropy_coeff": 0.05},
]


# ---------------------------------------------------------------------------
# PBRS potential function
# ---------------------------------------------------------------------------

def compute_potential(run_state) -> float:
    """Compute the potential Phi(s) for PBRS.

    Components:
    - floor_pct: progress through the run (floor / 55)
    - hp_pct: current health percentage
    - deck_quality: heuristic for deck composition quality

    Returns a scalar potential value.
    """
    hp_pct = run_state.current_hp / max(run_state.max_hp, 1)
    floor_pct = getattr(run_state, "floor", 0) / 55.0
    deck_size = len(getattr(run_state, "deck", []))
    # Ideal deck is 12-25 cards; penalize bloat
    if 12 <= deck_size <= 25:
        deck_quality = 1.0
    elif deck_size < 12:
        deck_quality = 0.8
    else:
        deck_quality = max(0.5, 1.0 - (deck_size - 25) * 0.02)

    # Relic count bonus (relics are always positive)
    relic_count = len(getattr(run_state, "relics", []))
    relic_bonus = min(relic_count * 0.02, 0.15)

    return 0.45 * floor_pct + 0.30 * hp_pct + 0.15 * deck_quality + 0.10 * relic_bonus


# Event rewards scaled by HP efficiency
EVENT_REWARDS = {
    "combat_win": 0.05,
    "elite_win": 0.15,
    "boss_win": 0.40,
}


# ---------------------------------------------------------------------------
# Worker function — runs in subprocess via ProcessPoolExecutor
# ---------------------------------------------------------------------------

def _play_one_game(
    seed: str,
    ascension: int,
    model_weights: Optional[bytes],
    model_config: Optional[Dict[str, Any]],
    temperature: float,
) -> Dict[str, Any]:
    """Play a single game and return transitions + result.

    This function runs in a worker process. It receives model weights as
    serialized bytes (not a model object) to avoid pickling torch models
    across processes. If model_weights is None, it uses random actions
    for strategic decisions.

    Returns a dict with:
        seed, won, floor, hp, decisions, duration_s,
        transitions: list of dicts with (obs, action_mask, action, reward,
                     done, value, log_prob, final_floor, cleared_act1/2/3)
    """
    import torch
    import torch.nn.functional as F
    import io

    from packages.engine.game import GameRunner, GamePhase
    from packages.training.planner import StrategicPlanner

    # Lazy-import model class and encoder
    from packages.training.strategic_net import StrategicNet
    from packages.training.state_encoder_v2 import RunStateEncoder

    encoder = RunStateEncoder()
    planner = StrategicPlanner()

    # Load model on CPU (MPS/CUDA don't fork well)
    model = None
    if model_weights is not None and model_config is not None:
        try:
            model = StrategicNet(**model_config)
            buf = io.BytesIO(model_weights)
            state_dict = torch.load(buf, map_location="cpu", weights_only=True)
            model.load_state_dict(state_dict)
            model.eval()
        except Exception:
            model = None

    try:
        runner = GameRunner(seed=seed, ascension=ascension, character="Watcher", verbose=False)
    except Exception:
        return {
            "seed": seed, "won": False, "floor": 0, "hp": 0,
            "decisions": 0, "duration_s": 0.0, "transitions": [],
        }

    t0 = time.monotonic()
    step = 0
    prev_floor = 0
    prev_potential = compute_potential(runner.run_state)
    decisions = 0
    transitions: List[Dict[str, Any]] = []

    # Track combat events for event-based rewards
    was_in_combat = False
    combat_room_type = "monster"

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
            was_in_combat = True
            combat_room_type = getattr(runner, "current_room_type", "monster")
            # Combat: use first action (heuristic/planner handles this)
            runner.take_action(actions[0])
        elif len(actions) == 1:
            # Check for combat-end event rewards
            if was_in_combat and phase != GamePhase.COMBAT:
                was_in_combat = False
            runner.take_action(actions[0])
        else:
            # Strategic decision point
            decisions += 1

            # Check if combat just ended (for event rewards)
            combat_just_ended = was_in_combat and phase != GamePhase.COMBAT
            if combat_just_ended:
                was_in_combat = False

            n_actions = len(actions)

            if model is not None:
                # Encode state
                run_obs = encoder.encode(rs)
                mask = np.zeros(model.action_dim, dtype=np.bool_)
                mask[:n_actions] = True

                # Forward pass on CPU
                with torch.no_grad():
                    obs_t = torch.from_numpy(run_obs).float().unsqueeze(0)
                    mask_t = torch.from_numpy(mask).bool().unsqueeze(0)

                    out = model(obs_t, mask_t)
                    logits = out["policy_logits"]
                    value = out["value"].item()

                    if temperature > 0:
                        probs = F.softmax(logits / temperature, dim=-1)
                        dist = torch.distributions.Categorical(probs)
                        action_idx = dist.sample().item()
                    else:
                        action_idx = logits.argmax(dim=-1).item()

                    # Compute log_prob at the actual temperature
                    probs_base = F.softmax(logits, dim=-1)
                    dist_base = torch.distributions.Categorical(probs_base)
                    log_prob = dist_base.log_prob(
                        torch.tensor(action_idx)
                    ).item()

                # Clamp to valid range
                action_idx = min(action_idx, n_actions - 1)

                # --- PBRS reward ---
                # Take action first, then compute Phi(s') - gamma * Phi(s)
                runner.take_action(actions[action_idx])
                new_rs = runner.run_state
                new_potential = compute_potential(new_rs)

                # PBRS: gamma * Phi(s') - Phi(s) preserves optimal policy
                gamma = 1.0
                pbrs_reward = gamma * new_potential - prev_potential

                # Event-based rewards on top of PBRS
                event_reward = 0.0
                if combat_just_ended:
                    rt = combat_room_type.lower() if isinstance(combat_room_type, str) else "monster"
                    if rt in ("elite", "e"):
                        event_reward = EVENT_REWARDS["elite_win"]
                    elif rt in ("boss", "b"):
                        event_reward = EVENT_REWARDS["boss_win"]
                    else:
                        event_reward = EVENT_REWARDS["combat_win"]
                    # Scale by HP efficiency
                    hp_pct = new_rs.current_hp / max(new_rs.max_hp, 1)
                    event_reward *= (0.5 + 0.5 * hp_pct)

                reward = pbrs_reward + event_reward
                prev_potential = new_potential

                # Record transition as numpy-serializable dict
                transitions.append({
                    "obs": run_obs,
                    "action_mask": mask,
                    "action": action_idx,
                    "reward": reward,
                    "done": False,
                    "value": value,
                    "log_prob": log_prob,
                    "final_floor": 0.0,
                    "cleared_act1": 0.0,
                    "cleared_act2": 0.0,
                    "cleared_act3": 0.0,
                })

            else:
                # No model: use heuristic planner
                if phase == GamePhase.MAP_NAVIGATION:
                    idx = planner.plan_path_choice(runner, actions)
                elif phase == GamePhase.REST:
                    idx = planner.plan_rest_site(runner, actions)
                elif phase in (GamePhase.COMBAT_REWARDS, GamePhase.BOSS_REWARDS):
                    idx = planner.plan_card_pick(runner, actions)
                elif phase == GamePhase.SHOP:
                    idx = planner.plan_shop_action(runner, actions)
                elif phase == GamePhase.EVENT:
                    idx = planner.plan_event_choice(runner, actions)
                else:
                    idx = 0
                runner.take_action(actions[min(idx, n_actions - 1)])
                prev_potential = compute_potential(runner.run_state)

        step += 1
        prev_floor = current_floor

    # Game ended
    duration = time.monotonic() - t0
    rs = runner.run_state
    won = runner.game_won
    final_floor = getattr(rs, "floor", 0)
    final_hp = getattr(rs, "current_hp", 0)

    cleared_acts = [
        final_floor >= 17,
        final_floor >= 34,
        final_floor >= 51,
    ]

    # Terminal reward on last transition
    if transitions:
        if won:
            transitions[-1]["reward"] += 1.0
        else:
            progress = final_floor / 55.0
            transitions[-1]["reward"] += -0.5 * (1 - progress)
        transitions[-1]["done"] = True

    # Backfill aux targets
    for t in transitions:
        t["final_floor"] = final_floor / 55.0
        t["cleared_act1"] = float(cleared_acts[0])
        t["cleared_act2"] = float(cleared_acts[1])
        t["cleared_act3"] = float(cleared_acts[2])

    return {
        "seed": seed,
        "won": won,
        "floor": final_floor,
        "hp": final_hp,
        "decisions": decisions,
        "duration_s": round(duration, 2),
        "transitions": transitions,
    }


# ---------------------------------------------------------------------------
# OvernightRunner — orchestrates training loop + multiprocessing
# ---------------------------------------------------------------------------

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
        ppo_batch_size: PPO mini-batch size (default 256)
        temperature: Exploration temperature for strategic decisions (default 1.0)
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
        self.ppo_batch_size = config.get("ppo_batch_size", 256)
        self.temperature = config.get("temperature", 1.0)

        self.run_dir.mkdir(parents=True, exist_ok=True)
        self._start_time = time.monotonic()
        self._start_datetime = datetime.now()
        self._current_sweep_idx = 0
        self._games_per_sweep = self.max_games // max(len(self.sweep_configs), 1)

        # Episodes log file
        self._episodes_path = self.run_dir / "episodes.jsonl"

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
        elapsed = time.monotonic() - self._start_time
        games_per_min = self.total_games / max(elapsed / 60.0, 0.01)
        status = {
            "timestamp": datetime.now().isoformat(),
            "elapsed_hours": round(elapsed / 3600, 2),
            "total_games": self.total_games,
            "total_wins": self.total_wins,
            "win_rate_100": round(sum(self.recent_wins) / max(len(self.recent_wins), 1) * 100, 1),
            "avg_floor_100": round(sum(self.recent_floors) / max(len(self.recent_floors), 1), 1),
            "games_per_min": round(games_per_min, 1),
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

    def _log_episode(self, result: Dict[str, Any]) -> None:
        """Append one episode to episodes.jsonl."""
        entry = {
            "timestamp": datetime.now().isoformat(),
            "seed": result["seed"],
            "floor": result["floor"],
            "won": result["won"],
            "hp": result["hp"],
            "decisions": result["decisions"],
            "duration_s": result["duration_s"],
            "num_transitions": len(result.get("transitions", [])),
        }
        with open(self._episodes_path, "a") as f:
            f.write(json.dumps(entry) + "\n")

    def _serialize_model_weights(self, model) -> tuple[Optional[bytes], Optional[Dict]]:
        """Serialize model state_dict to bytes for sending to workers."""
        import torch
        import io

        try:
            config = {
                "input_dim": model.input_dim,
                "hidden_dim": model.hidden_dim,
                "action_dim": model.action_dim,
                "num_blocks": model.num_blocks,
            }
            buf = io.BytesIO()
            torch.save(model.state_dict(), buf)
            return buf.getvalue(), config
        except Exception:
            return None, None

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
            batch_size = sweep_config.get("batch_size", self.ppo_batch_size)
            trainer = StrategicTrainer(
                model=model,
                lr=lr,
                entropy_coeff=sweep_config.get("entropy_coeff", 0.05),
                clip_epsilon=sweep_config.get("clip_epsilon", 0.2),
                batch_size=batch_size,
            )

            sweep_games = 0
            sweep_start = time.monotonic()

            logger.info(
                "Sweep %d/%d: lr=%.1e, ent=%.3f, clip=%.2f, batch=%d",
                sweep_idx + 1, len(self.sweep_configs),
                lr,
                sweep_config.get("entropy_coeff", 0.05),
                sweep_config.get("clip_epsilon", 0.2),
                batch_size,
            )

            while sweep_games < self._games_per_sweep and self.total_games < self.max_games:
                batch_t0 = time.monotonic()
                batch_results = self._play_batch(
                    model, encoder, seed_pool, trainer,
                )
                batch_duration = time.monotonic() - batch_t0

                for result in batch_results:
                    self._record_game(result["won"], result["floor"])
                    self._log_episode(result)
                    sweep_games += 1

                games_per_min = len(batch_results) / max(batch_duration / 60.0, 0.01)

                # Train if enough transitions
                if len(trainer.buffer) >= trainer.batch_size:
                    metrics = trainer.train_batch()
                    trainer.decay_entropy()

                    # Logging
                    avg_floor = sum(self.recent_floors) / max(len(self.recent_floors), 1)
                    wr = sum(self.recent_wins) / max(len(self.recent_wins), 1)

                    logger.info(
                        "Games %d | WR %.1f%% | Floor %.1f | Loss %.4f | "
                        "Trans %d | Buffer %d | Ent %.3f | LR %.1e | %.1f g/min",
                        self.total_games, wr * 100, avg_floor,
                        metrics.get("total_loss", 0),
                        metrics.get("num_transitions", 0),
                        len(trainer.buffer),
                        metrics.get("entropy_coeff", 0),
                        metrics.get("lr", 0),
                        games_per_min,
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
                    "buffer_size": len(trainer.buffer),
                    "games_per_min": round(games_per_min, 1),
                    "entropy_coeff": trainer.entropy_coeff,
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
        """Play a batch of games in parallel using ProcessPoolExecutor.

        Workers receive serialized model weights (bytes) and return raw
        transitions as numpy arrays. The main process then adds transitions
        to the trainer buffer.
        """
        # Serialize model weights once for all workers
        model_weights, model_config = self._serialize_model_weights(model)

        # Collect seeds
        seeds = [seed_pool.get_seed() for _ in range(self.games_per_batch)]

        results: List[Dict[str, Any]] = []

        # Use ProcessPoolExecutor for true parallelism
        with ProcessPoolExecutor(max_workers=self.workers) as executor:
            futures = {
                executor.submit(
                    _play_one_game,
                    seed,
                    self.ascension,
                    model_weights,
                    model_config,
                    self.temperature,
                ): seed
                for seed in seeds
            }

            for future in as_completed(futures):
                seed = futures[future]
                try:
                    result = future.result(timeout=120)  # 2 min timeout per game
                except Exception as e:
                    logger.warning("Game %s failed: %s", seed, e)
                    result = {
                        "seed": seed, "won": False, "floor": 0, "hp": 0,
                        "decisions": 0, "duration_s": 0.0, "transitions": [],
                    }

                # Add transitions from this game to trainer buffer
                for t in result.get("transitions", []):
                    trainer.add_transition(
                        obs=t["obs"],
                        action_mask=t["action_mask"],
                        action=t["action"],
                        reward=t["reward"],
                        done=t["done"],
                        value=t["value"],
                        log_prob=t["log_prob"],
                    )
                    # Backfill aux targets
                    buf_t = trainer.buffer[-1]
                    buf_t.final_floor = t["final_floor"]
                    buf_t.cleared_act1 = t["cleared_act1"]
                    buf_t.cleared_act2 = t["cleared_act2"]
                    buf_t.cleared_act3 = t["cleared_act3"]

                seed_pool.record_result(seed, {"won": result["won"], "floor": result["floor"]})
                results.append(result)

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


# Import for backward compat — the canonical source is strategic_trainer.py
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
    parser.add_argument("--batch-size", type=int, default=256, help="PPO mini-batch size")
    parser.add_argument("--ascension", type=int, default=0, help="Ascension level")
    parser.add_argument("--run-dir", type=str, default="logs/overnight", help="Output directory")
    parser.add_argument("--headless-after", type=int, default=30, help="Go headless after N minutes")
    parser.add_argument("--visual-at", type=str, default="07:30", help="Switch to visual at HH:MM")
    parser.add_argument("--temperature", type=float, default=1.0, help="Exploration temperature (0=greedy)")
    args = parser.parse_args()

    runner = OvernightRunner({
        "workers": args.workers,
        "max_games": args.games,
        "games_per_batch": args.batch,
        "ppo_batch_size": args.batch_size,
        "ascension": args.ascension,
        "run_dir": args.run_dir,
        "headless_after_min": args.headless_after,
        "visual_at": args.visual_at,
        "temperature": args.temperature,
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
