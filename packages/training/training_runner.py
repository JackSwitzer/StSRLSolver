"""Training runner with scheduling and hyperparameter sweep.

Orchestrates the training loop:
- Multiprocessing worker pool for parallel game execution
- Centralized GPU inference server for batched forward passes
- Phased collect/train loop with experience replay
- Hot-reloadable configuration via signal handler
- Checkpoint management with warm restart support
"""

from __future__ import annotations

import gc
import json
import logging
import math
import multiprocessing as mp
import signal
import time
from collections import deque
from datetime import datetime
from pathlib import Path
from typing import Any, Deque, Dict, List, Optional, Tuple

import numpy as np

from .episode_log import log_episode
from .replay_buffer import TrajectoryReplayBuffer
from .reward_config import (
    EVENT_REWARDS,
    FLOOR_MILESTONES,
    REWARD_WEIGHTS,
    UPGRADE_REWARDS,
)
from .sweep_config import ASCENSION_BREAKPOINTS, DEFAULT_SWEEP_CONFIGS, WEEKEND_SWEEP_CONFIGS, OVERNIGHT_SWEEP_CONFIGS
from .training_config import EXPLORE_TEMP_MULTIPLIER, EXPLORE_GAME_RATIO
from .training_config import ABORT_CLIP_FRACTION, ABORT_VALUE_LOSS, ABORT_ENTROPY_MIN, ABORT_GRACE_GAMES
from .training_config import (
    ALGORITHM,
    MODEL_HIDDEN_DIM,
    MODEL_NUM_BLOCKS,
    REPLAY_BUFFER_SIZE,
    REPLAY_MIN_FLOOR,
    REPLAY_MIX_RATIO,
    STALL_DETECTION_WINDOW,
    STALL_IMPROVEMENT_THRESHOLD,
    TEMPERATURE,
    TRAIN_BATCH_SIZE,
    TRAIN_COLLECT_GAMES,
    TRAIN_GAMES_PER_BATCH,
    TRAIN_MAX_BATCH_INFERENCE,
    TRAIN_PPO_EPOCHS,
    TRAIN_STEPS_PER_PHASE,
    TRAIN_WORKERS,
)
from .worker import _ACTION_DIM, _play_one_game, _worker_init

logger = logging.getLogger(__name__)


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
        self.run_dir = Path(config.get("run_dir", "logs/active"))
        self.max_games = config.get("max_games", 50000)
        self.games_per_batch = config.get("games_per_batch", TRAIN_GAMES_PER_BATCH)
        self.workers = config.get("workers", TRAIN_WORKERS)
        self.ascension = config.get("ascension", 0)
        self.eval_every = config.get("eval_every", 500)
        self.ppo_batch_size = config.get("ppo_batch_size", TRAIN_BATCH_SIZE)
        self.temperature = config.get("temperature", TEMPERATURE)
        self.resume_path = config.get("resume_path", None)
        self.hidden_dim = config.get("hidden_dim", MODEL_HIDDEN_DIM)
        self.num_blocks = config.get("num_blocks", MODEL_NUM_BLOCKS)
        self.max_batch_size = config.get("max_batch_size", TRAIN_MAX_BATCH_INFERENCE)
        self.max_hours_per_config = config.get("max_hours_per_config", None)  # None = auto

        self.run_dir.mkdir(parents=True, exist_ok=True)
        self._start_time = time.monotonic()
        self._start_datetime = datetime.now()
        self._current_sweep_idx = 0
        self._games_per_sweep = self.max_games // max(len(self.sweep_configs), 1)

        # Episodes log file
        self._episodes_path = self.run_dir / "episodes.jsonl"

        # Graceful shutdown flag (set by signal handler)
        self._shutdown_requested = False

        # Stats tracking
        self.total_games = 0
        self.total_wins = 0
        self.recent_floors: Deque[int] = deque(maxlen=100)
        self.recent_wins: Deque[bool] = deque(maxlen=100)
        self.peak_floor = 0
        self.sweep_results: List[Dict[str, Any]] = []
        self._episode_counter = 0  # Unique ID per game for GAE episode separation

        # Stall detection: track avg floor at checkpoints to detect training plateaus
        self._stall_checkpoint_floor = 0.0
        self._stall_checkpoint_games = 0
        self._construction_failures = 0
        self._gpu_cache: Tuple[float, Optional[int]] = (0.0, None)

        # Current sweep config (set by _run_config for epsilon forwarding)
        self._current_sweep_config: Dict[str, Any] = {}

        # Recent episodes for dashboard broadcast
        self._recent_episodes: Deque[Dict[str, Any]] = deque(maxlen=100)

        # Last training metrics for dashboard visibility
        self._last_train_metrics: Dict[str, float] = {}

        # Inference server + persistent pool (created in run())
        self._server = None
        self._executor: Optional[Any] = None

        # Track whether distillation has run this session (prevents re-distilling on config switch)
        self._distilled_this_run = False

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

    @staticmethod
    def _read_gpu_percent() -> Optional[int]:
        """Read GPU utilization from ioreg (Apple Silicon)."""
        import subprocess
        import re
        out = subprocess.check_output(
            ["ioreg", "-r", "-d", "1", "-c", "IOAccelerator"],
            text=True, timeout=2,
        )
        for line in out.splitlines():
            if "Device Utilization" in line:
                m = re.search(r'"Device Utilization %".*?=\s*(\d+)', line)
                if m:
                    return int(m.group(1))
        return None

    def write_status(self, stats: Dict[str, Any]) -> None:
        """Write status.json for monitoring."""
        elapsed = time.monotonic() - self._start_time
        games_per_min = self.total_games / max(elapsed / 60.0, 0.01)

        # Read GPU utilization (cached, refresh every 10s)
        now = time.monotonic()
        if now - self._gpu_cache[0] > 10.0:
            try:
                self._gpu_cache = (now, self._read_gpu_percent())
            except Exception:
                self._gpu_cache = (now, None)
        gpu_pct = self._gpu_cache[1]

        status = {
            "timestamp": datetime.now().isoformat(),
            "elapsed_hours": round(elapsed / 3600, 2),
            "total_games": self.total_games,
            "total_wins": self.total_wins,
            "win_rate_100": round(sum(self.recent_wins) / max(len(self.recent_wins), 1) * 100, 1),
            "avg_floor_100": round(sum(self.recent_floors) / max(len(self.recent_floors), 1), 1),
            "games_per_min": round(games_per_min, 1),
            "peak_floor": self.peak_floor,
            "current_sweep": self._current_sweep_idx,
            "total_sweeps": len(self.sweep_configs),
            "headless": self.should_be_headless(),
            "construction_failures": self._construction_failures,
            "gpu_percent": gpu_pct,
            **stats,
        }

        # Add inference server stats if available
        if self._server is not None:
            try:
                status["inference"] = self._server.get_stats()
            except Exception:
                pass

        status_path = self.run_dir / "status.json"
        status_path.write_text(json.dumps(status, indent=2))

    def _record_game(self, result: Dict[str, Any]) -> None:
        """Record a game result and write recent_episodes.json for dashboard."""
        self.total_games += 1
        if result["won"]:
            self.total_wins += 1
        self.recent_floors.append(result["floor"])
        self.recent_wins.append(result["won"])
        self.peak_floor = max(self.peak_floor, result["floor"])
        if result.get("construction_failure"):
            self._construction_failures += 1

        # Append to recent episodes buffer for dashboard visibility
        ep = {
            "type": "agent_episode",
            "agent_id": 0,
            "seed": result.get("seed", ""),
            "won": result.get("won", False),
            "floors_reached": result.get("floor", 0),
            "hp_remaining": result.get("hp", 0),
            "max_hp": result.get("max_hp", 0),
            "total_steps": result.get("decisions", 0),
            "duration": result.get("duration_s", 0),
            "episode": self.total_games,
            "death_floor": result.get("floor", 0) if not result.get("won") else None,
            "death_enemy": result.get("death_enemy"),
            "combats": result.get("combats", []),
            "events": result.get("events", []),
            "deck_changes": result.get("deck_changes", []),
            "deck_final": result.get("deck_final", []),
            "relics_final": result.get("relics_final", []),
            "path_choices": result.get("path_choices", []),
            "card_picks": result.get("card_picks", []),
        }
        self._recent_episodes.append(ep)
        # Write every 10 games to avoid I/O spam
        if self.total_games % 10 == 0:
            try:
                ep_path = self.run_dir / "recent_episodes.json"
                ep_path.write_text(json.dumps(list(self._recent_episodes), default=str))
            except Exception:
                pass
            # Auto-write floor_curve.json for dashboard
            try:
                curve_path = self.run_dir / "floor_curve.json"
                curve_path.write_text(json.dumps(list(self.recent_floors)))
            except Exception:
                pass

    def _log_episode(self, result: Dict[str, Any]) -> None:
        """Append one episode to episodes.jsonl."""
        cfg = self._current_sweep_config
        config_name = cfg.get("name", "") if cfg else ""
        log_episode(self._episodes_path, result, config_name=config_name)

    def _save_best_trajectory(self, result: Dict[str, Any]) -> None:
        """Save transitions from top runs to disk for future warm-starts."""
        floor = result.get("floor", 0)
        transitions = result.get("transitions", [])
        if floor < 8 or not transitions:
            return

        traj_dir = self.run_dir / "best_trajectories"
        traj_dir.mkdir(exist_ok=True)

        # Keep max 200 trajectory files — replace worst if full
        existing = sorted(traj_dir.glob("traj_F*.npz"), key=lambda p: p.stat().st_mtime)
        if len(existing) >= 200:
            # Parse floor from filename, remove lowest
            floors = []
            for p in existing:
                try:
                    f = int(p.stem.split("_F")[1].split("_")[0])
                    floors.append((f, p))
                except (IndexError, ValueError):
                    floors.append((0, p))
            floors.sort(key=lambda x: x[0])
            if floors[0][0] < floor:
                floors[0][1].unlink()
            else:
                return  # This trajectory isn't better than worst saved

        # Serialize transitions as numpy arrays
        obs = np.array([t["obs"] for t in transitions], dtype=np.float32)
        masks = np.array([t["action_mask"] for t in transitions], dtype=np.bool_)
        actions = np.array([t["action"] for t in transitions], dtype=np.int32)
        rewards = np.array([t["reward"] for t in transitions], dtype=np.float32)
        dones = np.array([t["done"] for t in transitions], dtype=np.bool_)
        values = np.array([t["value"] for t in transitions], dtype=np.float32)
        log_probs = np.array([t["log_prob"] for t in transitions], dtype=np.float32)
        final_floors = np.array([t["final_floor"] for t in transitions], dtype=np.float32)
        cleared_act1 = np.array([t["cleared_act1"] for t in transitions], dtype=np.float32)

        fname = f"traj_F{floor:02d}_{result['seed']}.npz"
        np.savez_compressed(
            traj_dir / fname,
            obs=obs, masks=masks, actions=actions, rewards=rewards,
            dones=dones, values=values, log_probs=log_probs,
            final_floors=final_floors, cleared_act1=cleared_act1,
            floor=np.array([floor]),
        )

    def _pretrain_from_trajectories(self, trainer, model) -> int:
        """Load saved best trajectories and pretrain for a few epochs. Returns steps taken."""
        traj_dir = self.run_dir / "best_trajectories"
        if not traj_dir.exists():
            return 0

        traj_files = sorted(traj_dir.glob("traj_F*.npz"),
                            key=lambda p: p.stem, reverse=True)
        if not traj_files:
            return 0

        logger.info("Pretraining from %d saved trajectories...", len(traj_files))

        # Load all trajectories into trainer buffer
        total_transitions = 0
        for tf in traj_files:
            try:
                data = np.load(tf)
                n = len(data["obs"])
                ep_id = hash(tf.stem) % (2**31)
                for i in range(n):
                    obs_i = data["obs"][i]
                    # Skip mismatched dimensions (older trajectories)
                    if obs_i.shape[0] != model.input_dim:
                        continue
                    mask_i = data["masks"][i]
                    # Pad mask if trajectory was saved with smaller action_dim
                    if mask_i.shape[0] < _ACTION_DIM:
                        mask_i = np.pad(mask_i, (0, _ACTION_DIM - mask_i.shape[0]),
                                        constant_values=False)
                    trainer.add_transition(
                        obs=obs_i,
                        action_mask=mask_i,
                        action=int(data["actions"][i]),
                        reward=float(data["rewards"][i]),
                        done=bool(data["dones"][i]),
                        value=float(data["values"][i]),
                        log_prob=float(data["log_probs"][i]),
                        episode_id=ep_id,
                    )
                    buf_t = trainer.buffer[-1]
                    buf_t.final_floor = float(data["final_floors"][i])
                    buf_t.cleared_act1 = float(data["cleared_act1"][i])
                total_transitions += n
            except Exception as e:
                logger.warning("Failed to load trajectory %s: %s", tf.name, e)

        if total_transitions == 0:
            return 0

        # Run several training epochs on this data
        pretrain_steps = 0
        for epoch in range(3):
            if len(trainer.buffer) >= trainer.batch_size:
                metrics = trainer.train_batch()
                pretrain_steps += 1
                logger.info("Pretrain epoch %d: loss=%.4f, %d transitions",
                            epoch, metrics.get("total_loss", 0), total_transitions)
                # Sync weights to inference server
                if self._server is not None:
                    self._server.sync_strategic_from_pytorch(
                        model, version=trainer.train_steps
                    )

        logger.info("Pretrained %d steps on %d transitions from %d trajectories",
                    pretrain_steps, total_transitions, len(traj_files))
        return pretrain_steps

    def _deep_distillation(self, trainer, model, replay_buffer) -> int:
        """Deep distillation: load ALL trajectories + replay buffer and train intensively.

        Unlike _pretrain_from_trajectories (behavioral cloning warmup, 3 steps),
        this runs 50 full PPO train_batch() calls with 8 epochs each for a
        one-time deep bootstrap before the main collect/train loop begins.

        Returns number of training steps completed.
        """
        from packages.training.strategic_trainer import StrategicTransition

        traj_dir = self.run_dir / "best_trajectories"
        total_loaded = 0
        files_loaded = 0
        files_failed = 0

        # 1) Load ALL .npz trajectory files
        if traj_dir.exists():
            traj_files = sorted(traj_dir.glob("traj_F*.npz"),
                                key=lambda p: p.stem, reverse=True)
            logger.info("Deep distillation: loading %d trajectory files...", len(traj_files))

            for tf in traj_files:
                try:
                    data = np.load(tf)
                    n = len(data["obs"])
                    ep_id = hash(tf.stem) % (2**31)
                    for i in range(n):
                        mask_i = data["masks"][i]
                        if mask_i.shape[0] < _ACTION_DIM:
                            mask_i = np.pad(mask_i, (0, _ACTION_DIM - mask_i.shape[0]),
                                            constant_values=False)
                        st = StrategicTransition(
                            obs=data["obs"][i],
                            action_mask=mask_i,
                            action=int(data["actions"][i]),
                            reward=float(data["rewards"][i]),
                            done=bool(data["dones"][i]),
                            value=float(data["values"][i]),
                            log_prob=float(data["log_probs"][i]),
                            episode_id=ep_id,
                            final_floor=float(data["final_floors"][i]),
                            cleared_act1=float(data["cleared_act1"][i]),
                        )
                        trainer.buffer.append(st)
                    total_loaded += n
                    files_loaded += 1
                except Exception as e:
                    logger.warning("Deep distill: failed to load %s: %s", tf.name, e)
                    files_failed += 1

        # 2) Load all replay buffer transitions
        replay_loaded = 0
        if replay_buffer.size > 0:
            replay_transitions = replay_buffer.sample_transitions(
                n=replay_buffer._total_transitions,  # sample everything
            )
            for t in replay_transitions:
                try:
                    st = StrategicTransition(
                        obs=t["obs"], action_mask=t["action_mask"],
                        action=t["action"], reward=t["reward"],
                        done=t["done"], value=t["value"],
                        log_prob=t["log_prob"],
                        episode_id=t.get("episode_id", 0),
                        final_floor=t.get("final_floor", 0),
                        cleared_act1=t.get("cleared_act1", 0),
                        cleared_act2=t.get("cleared_act2", 0),
                        cleared_act3=t.get("cleared_act3", 0),
                    )
                    trainer.buffer.append(st)
                    replay_loaded += 1
                except (KeyError, TypeError) as e:
                    logger.debug("Deep distill: skip replay transition: %s", e)
                    continue

        if total_loaded + replay_loaded == 0:
            logger.info("Deep distillation: no data to distill from, skipping")
            return 0

        logger.info(
            "Deep distillation: %d transitions from %d files (%d failed) + %d replay. "
            "Buffer size: %d. Starting 50-step intensive training...",
            total_loaded, files_loaded, files_failed, replay_loaded, len(trainer.buffer),
        )

        # 3) Run 50 train_batch() calls with 8 PPO epochs each
        DISTILL_STEPS = 50
        DISTILL_EPOCHS = 8
        orig_epochs = trainer.ppo_epochs
        trainer.ppo_epochs = DISTILL_EPOCHS
        distill_count = 0
        distill_t0 = time.monotonic()

        for step in range(DISTILL_STEPS):
            if len(trainer.buffer) < trainer.batch_size // 2:
                logger.warning("Deep distillation: buffer too small (%d < %d), stopping at step %d",
                               len(trainer.buffer), trainer.batch_size // 2, step)
                break

            try:
                metrics = trainer.train_batch()
                distill_count += 1
            except Exception as e:
                logger.warning("Deep distillation: train_batch failed at step %d: %s", step, e)
                break

            if (step + 1) % 5 == 0 or step == 0:
                elapsed = time.monotonic() - distill_t0
                logger.info(
                    "  Distill step %d/%d: loss=%.4f, policy=%.4f, value=%.4f, "
                    "entropy=%.4f, buffer=%d [%.1fs elapsed]",
                    step + 1, DISTILL_STEPS,
                    metrics.get("total_loss", 0),
                    metrics.get("policy_loss", 0),
                    metrics.get("value_loss", 0),
                    metrics.get("entropy", 0),
                    len(trainer.buffer),
                    elapsed,
                )

        trainer.ppo_epochs = orig_epochs

        # 4) Sync weights to inference server
        if self._server is not None and distill_count > 0:
            self._server.sync_strategic_from_pytorch(
                model, version=trainer.train_steps
            )

        # Clear buffer after distillation — main loop will collect fresh data
        trainer.buffer.clear()

        distill_duration = time.monotonic() - distill_t0
        logger.info(
            "Deep distillation complete: %d steps in %.1fs (%.1f steps/sec). "
            "Total train_steps now: %d",
            distill_count, distill_duration,
            distill_count / max(distill_duration, 0.01),
            trainer.train_steps,
        )
        return distill_count

    def run(self) -> Dict[str, Any]:
        """Main overnight loop.

        Integrates with StrategicTrainer to train the strategic model
        while using the combat solver for combat phases.
        """
        import torch
        from .strategic_net import StrategicNet, _get_device
        from .strategic_trainer import StrategicTrainer
        from .state_encoders import RunStateEncoder
        from .seed_pool import SeedPool

        device = _get_device()
        encoder = RunStateEncoder()

        # Initialize model (optionally resume from checkpoint)
        _warm_checkpoint = None  # Will hold optimizer/scheduler state if available
        if self.resume_path:
            try:
                model = StrategicNet.load(self.resume_path, device=device)
                model.train()
                # Try to load warm-restart state (optimizer, scheduler, etc.)
                ckpt = torch.load(self.resume_path, map_location=device, weights_only=False)
                if "optimizer_state_dict" in ckpt:
                    _warm_checkpoint = ckpt
                    logger.info("Warm resume from %s (train_steps=%d, games=%d)",
                                self.resume_path,
                                ckpt.get("train_steps", 0),
                                ckpt.get("total_games", 0))
                else:
                    logger.info("Cold resume from %s (model weights only)", self.resume_path)
            except Exception as e:
                logger.warning("Failed to resume from %s: %s — starting fresh", self.resume_path, e)
                model = StrategicNet(
                    input_dim=encoder.RUN_DIM,
                    hidden_dim=self.hidden_dim,
                    num_blocks=self.num_blocks,
                ).to(device)
        else:
            model = StrategicNet(
                input_dim=encoder.RUN_DIM,
                hidden_dim=self.hidden_dim,
                num_blocks=self.num_blocks,
            ).to(device)
        assert model.input_dim == encoder.RUN_DIM, (
            f"Model input_dim ({model.input_dim}) != encoder RUN_DIM ({encoder.RUN_DIM})"
        )
        logger.info(
            "Strategic model: %d parameters (hidden=%d, blocks=%d), device=%s",
            model.param_count(), model.hidden_dim, model.num_blocks, device,
        )

        # --- Inference server setup ---
        from packages.training.inference_server import InferenceServer

        from .training_config import INFERENCE_BATCH_TIMEOUT_MS
        self._server = InferenceServer(
            n_workers=self.workers, max_batch_size=self.max_batch_size,
            batch_timeout_ms=INFERENCE_BATCH_TIMEOUT_MS,
        )
        self._server.sync_strategic_from_pytorch(model, version=0)
        self._server.start()
        logger.info("InferenceServer started (workers=%d)", self.workers)

        # --- Signal handlers ---
        def _handle_shutdown(signum, frame):
            sig_name = signal.Signals(signum).name
            logger.info("Graceful shutdown requested (%s), finishing current batch...", sig_name)
            self._shutdown_requested = True

        def _handle_reload(signum, frame):
            """Hot-reload config from {run_dir}/reload.json on SIGUSR1."""
            reload_path = self.run_dir / "reload.json"
            if reload_path.exists():
                try:
                    import json as _json
                    cfg = _json.loads(reload_path.read_text())
                    logger.info("Hot-reload from %s: %s", reload_path, cfg)
                    # --- Training hyperparams ---
                    _t = getattr(self, '_trainer', None)
                    if "entropy_coeff" in cfg and _t:
                        _t.entropy_coeff = cfg["entropy_coeff"]
                        logger.info("  entropy_coeff -> %s", cfg["entropy_coeff"])
                    if "temperature" in cfg:
                        self.temperature = cfg["temperature"]
                        logger.info("  temperature -> %s", cfg["temperature"])
                    if "lr" in cfg and _t:
                        for pg in _t.optimizer.param_groups:
                            pg["lr"] = cfg["lr"]
                        logger.info("  lr -> %s", cfg["lr"])
                    if "clip_epsilon" in cfg and _t:
                        _t.clip_epsilon = cfg["clip_epsilon"]
                        logger.info("  clip_epsilon -> %s", cfg["clip_epsilon"])
                    if "batch_size" in cfg and _t:
                        _t.batch_size = cfg["batch_size"]
                        logger.info("  batch_size -> %s", cfg["batch_size"])

                    # --- Reward weights (unified system) ---
                    if "reward_weights" in cfg:
                        REWARD_WEIGHTS.update(cfg["reward_weights"])
                        # Note: module-level convenience accessors in reward_config
                        # won't update automatically. Use REWARD_WEIGHTS dict directly.
                        EVENT_REWARDS.update({
                            "combat_win": REWARD_WEIGHTS.get("combat_win", 0.05),
                            "elite_win": REWARD_WEIGHTS.get("elite_win", 0.30),
                            "boss_win": REWARD_WEIGHTS.get("boss_win", 0.80),
                        })
                        if "floor_milestones" in REWARD_WEIGHTS:
                            FLOOR_MILESTONES.update(
                                {int(k): v for k, v in REWARD_WEIGHTS["floor_milestones"].items()}
                            )
                        logger.info("  reward_weights -> updated")

                    # --- Direct dict hot-reload ---
                    if "event_rewards" in cfg:
                        EVENT_REWARDS.update(cfg["event_rewards"])
                        logger.info("  event_rewards -> %s", EVENT_REWARDS)
                    if "floor_milestones" in cfg:
                        FLOOR_MILESTONES.update({int(k): v for k, v in cfg["floor_milestones"].items()})
                        logger.info("  floor_milestones -> %s", FLOOR_MILESTONES)
                    if "upgrade_rewards" in cfg:
                        UPGRADE_REWARDS.update(cfg["upgrade_rewards"])
                        logger.info("  upgrade_rewards -> %s", UPGRADE_REWARDS)

                    # --- Replay buffer ---
                    if "replay_mix_ratio" in cfg:
                        # Can't rebind module-level variable from here; use REWARD_WEIGHTS
                        logger.info("  replay_mix_ratio -> %s (note: requires restart)", cfg["replay_mix_ratio"])
                    if "replay_min_floor" in cfg and hasattr(self, '_replay_buffer'):
                        self._replay_buffer.min_floor = cfg["replay_min_floor"]
                        logger.info("  replay_min_floor -> %s", cfg["replay_min_floor"])

                    reload_path.unlink()
                except Exception as e:
                    logger.error("Hot-reload failed: %s", e)

        signal.signal(signal.SIGTERM, _handle_shutdown)
        signal.signal(signal.SIGINT, _handle_shutdown)
        signal.signal(signal.SIGUSR1, _handle_reload)

        seed_pool = SeedPool(max_plays=5)
        best_avg_floor = 0.0

        # Adaptive 3-phase sweep:
        # Phase 1: Each config gets equal games (~25% of total each)
        # Phase 2: Keep top 2 configs, each gets ~15% more games
        # Phase 3: All-in on best config with remaining games
        n_configs = len(self.sweep_configs)
        phase1_games_per = self.max_games // (n_configs * 3)  # ~33% of budget split equally

        config_scores: Dict[int, Dict[str, Any]] = {}  # idx -> {avg_floor, games, ...}

        # Replay buffer for best trajectory distillation
        replay_buffer = TrajectoryReplayBuffer(
            max_trajectories=REPLAY_BUFFER_SIZE,
            min_floor=REPLAY_MIN_FLOOR,
        )
        self._replay_buffer = replay_buffer

        def _run_config(sweep_idx: int, sweep_config: Dict, n_games: int, fork_weights: bool = False) -> Dict[str, Any]:
            """Run a config for n_games, return metrics."""
            nonlocal best_avg_floor, _warm_checkpoint
            # No weight forking — learning accumulates across configs
            self._current_sweep_idx = sweep_idx
            self._current_sweep_config = sweep_config
            lr = sweep_config.get("lr", 1e-4)
            batch_size = sweep_config.get("batch_size", self.ppo_batch_size)
            temp = sweep_config.get("temperature", self.temperature)
            self.temperature = temp

            algorithm = sweep_config.get("algorithm", ALGORITHM)

            if algorithm == "iql":
                from .iql_trainer import IQLTrainer
                trainer = IQLTrainer(
                    policy=model,
                    input_dim=model.input_dim,
                    action_dim=model.action_dim,
                    lr=sweep_config.get("lr", 3e-4),
                    expectile=sweep_config.get("iql_expectile", 0.7),
                    temperature=sweep_config.get("iql_temperature", 3.0),
                )
            elif algorithm == "grpo":
                from .grpo_trainer import GRPOTrainer
                from .training_config import GRPO_CLIP, GRPO_LR, GRPO_ROLLOUTS_CARD, GRPO_ROLLOUTS_OTHER
                # GRPO uses StrategicTrainer for collection (buffer/add_transition)
                # and GRPOTrainer for the actual policy update
                trainer = StrategicTrainer(
                    model=model,
                    lr=lr,
                    entropy_coeff=sweep_config.get("entropy_coeff", 0.05),
                    clip_epsilon=sweep_config.get("grpo_clip", GRPO_CLIP),
                    batch_size=batch_size,
                    lr_schedule=sweep_config.get("lr_schedule", "cosine"),
                    lr_T_max=sweep_config.get("lr_T_max", 30000),
                    lr_T_0=sweep_config.get("lr_T_0", 5000),
                )
                grpo_trainer = GRPOTrainer(
                    model=model,
                    lr=sweep_config.get("grpo_lr", GRPO_LR),
                    clip=sweep_config.get("grpo_clip", GRPO_CLIP),
                    rollouts_card=sweep_config.get("grpo_rollouts_card", GRPO_ROLLOUTS_CARD),
                    rollouts_other=sweep_config.get("grpo_rollouts_other", GRPO_ROLLOUTS_OTHER),
                )
            else:
                # PPO (default)
                clip_eps = sweep_config.get("clip_epsilon", 0.2)
                trainer = StrategicTrainer(
                    model=model,
                    lr=lr,
                    entropy_coeff=sweep_config.get("entropy_coeff", 0.05),
                    clip_epsilon=clip_eps,
                    batch_size=batch_size,
                    lr_schedule=sweep_config.get("lr_schedule", "cosine"),
                    lr_T_max=sweep_config.get("lr_T_max", 30000),
                    lr_T_0=sweep_config.get("lr_T_0", 5000),
                )

            # Store trainer ref for signal handler access (hot-reload)
            self._trainer = trainer

            # Warm restart: restore optimizer + scheduler state from checkpoint
            if _warm_checkpoint is not None:
                try:
                    trainer.optimizer.load_state_dict(_warm_checkpoint["optimizer_state_dict"])
                    if hasattr(trainer, "scheduler") and "scheduler_state_dict" in _warm_checkpoint:
                        trainer.scheduler.load_state_dict(_warm_checkpoint["scheduler_state_dict"])
                    trainer.train_steps = _warm_checkpoint.get("train_steps", 0)
                    if "entropy_coeff" in _warm_checkpoint and hasattr(trainer, "entropy_coeff"):
                        trainer.entropy_coeff = _warm_checkpoint["entropy_coeff"]
                    logger.info("Warm restart: optimizer restored (train_steps=%d, entropy=%.4f)",
                                trainer.train_steps, getattr(trainer, "entropy_coeff", 0.0))
                except Exception as e:
                    logger.warning("Could not restore optimizer state: %s — using fresh optimizer", e)
                _warm_checkpoint = None  # Only restore once

            # Only distill on cold start (no checkpoint). Warm restarts already have
            # trained weights — re-distilling on the same data wastes time.
            # bc_warmup: True in config forces BC even if distillation already ran.
            _force_bc = sweep_config.get("bc_warmup", False)
            if (not self._distilled_this_run or _force_bc) and _warm_checkpoint is None and trainer.train_steps == 0:
                # BC pretrain on best trajectories (only if trainer supports it)
                if hasattr(trainer, 'bc_pretrain'):
                    traj_dir = self.run_dir / "best_trajectories"
                    if traj_dir.exists() and any(traj_dir.glob("traj_F*.npz")):
                        logger.info("=== BC Pretrain ===")
                        bc_metrics = trainer.bc_pretrain(traj_dir, epochs=10)
                        logger.info("BC complete: %s", bc_metrics)
                        if self._server is not None:
                            self._server.sync_strategic_from_pytorch(model, version=trainer.train_steps)
                # Trajectory pretrain + deep distillation require PPO-style .buffer interface
                if hasattr(trainer, 'buffer'):
                    self._pretrain_from_trajectories(trainer, model)
                    self._deep_distillation(trainer, model, replay_buffer)
                self._distilled_this_run = True
            else:
                logger.info("Warm restart (train_steps=%d) — skipping distillation", trainer.train_steps)

            # Create worker pool AFTER distillation — strict GPU phase separation.
            if self._executor is None:
                ctx = mp.get_context("spawn")
                shm_info = getattr(self._server, "shm_info", None)
                self._executor = ctx.Pool(
                    processes=self.workers,
                    initializer=_worker_init,
                    initargs=(
                        self._server.request_q,
                        self._server.response_qs,
                        self._server.slot_q,
                        shm_info,
                    ),
                )
                _mode = "shared_memory" if shm_info else "queue"
                logger.info(
                    "Worker pool started (%d processes, mode=%s) — distillation complete",
                    self.workers, _mode,
                )

            config_name = sweep_config.get("name", f"config_{sweep_idx}")
            sweep_games = 0
            sweep_start = time.monotonic()
            sweep_floors: Deque[int] = deque(maxlen=200)

            ts_ms = sweep_config.get("turn_solver_ms", 50.0)

            # Time limit per config (prevents single config from monopolizing)
            # Per-config max_hours overrides global max_hours_per_config
            config_start_time = time.monotonic()
            _per_config_hours = sweep_config.get("max_hours", None)
            if _per_config_hours is not None:
                max_seconds = _per_config_hours * 3600
            elif self.max_hours_per_config is not None:
                max_seconds = self.max_hours_per_config * 3600
            else:
                n_cfgs = max(len(self.sweep_configs), 1)
                max_seconds = float("inf") if n_cfgs <= 1 else (48 * 3600) / n_cfgs

            logger.info(
                "Config '%s': lr=%.1e, ent=%.3f, batch=%d, temp=%.1f, ts=%.0fms, time_limit=%.1fh",
                config_name, lr,
                sweep_config.get("entropy_coeff", 0.05),
                batch_size, temp, ts_ms,
                max_seconds / 3600,
            )

            # Phased loop: COLLECT games -> TRAIN on best data -> repeat.
            # Per-config overrides take precedence over training_config.py defaults.
            COLLECT_GAMES = sweep_config.get("collect_games", TRAIN_COLLECT_GAMES)
            TRAIN_EPOCHS = sweep_config.get("ppo_epochs", TRAIN_PPO_EPOCHS)
            _TRAIN_STEPS = sweep_config.get("train_steps", TRAIN_STEPS_PER_PHASE)
            games_per_min = 0.0

            # IQL: offline-only training (no game collection)
            if algorithm == "iql":
                from .offline_data import load_trajectories
                traj_dirs = [
                    self.run_dir / "best_trajectories",
                    Path("logs/consolidated"),
                ]
                dataset = load_trajectories(
                    [d for d in traj_dirs if d.exists()],
                    max_transitions=48000,
                )
                if len(dataset) > 0:
                    logger.info("IQL: training on %d offline transitions", len(dataset))
                    iql_metrics = trainer.train_offline(dataset, epochs=50)
                    logger.info("IQL complete: %s", iql_metrics)
                    if self._server is not None:
                        self._server.sync_strategic_from_pytorch(model, version=1)
                else:
                    logger.warning("IQL: no offline data found, skipping")
                # Skip the collect/train loop entirely
                sweep_elapsed = time.monotonic() - sweep_start
                return {
                    "config": sweep_config, "games": 0,
                    "avg_floor": 0, "win_rate": 0,
                    "duration_min": round(sweep_elapsed / 60, 1),
                    "train_steps": getattr(trainer, "train_steps", 0),
                }

            while (sweep_games < n_games
                   and self.total_games < self.max_games
                   and not self._shutdown_requested
                   and (time.monotonic() - config_start_time) < max_seconds):
                # -- COLLECT PHASE --
                collect_t0 = time.monotonic()
                collect_games = 0
                phase_results: List[Dict[str, Any]] = []

                logger.info("=== COLLECT phase: gathering %d games ===", COLLECT_GAMES)
                self.write_status({
                    "sweep_config": sweep_config, "sweep_phase": "collecting",
                    "config_name": config_name, "sweep_games": sweep_games,
                    "train_steps": trainer.train_steps,
                    "buffer_size": len(trainer.buffer),
                    "games_per_min": round(games_per_min, 1),
                    "total_loss": self._last_train_metrics.get("total_loss"),
                    "policy_loss": self._last_train_metrics.get("policy_loss"),
                    "value_loss": self._last_train_metrics.get("value_loss"),
                    "entropy": self._last_train_metrics.get("entropy"),
                })

                while collect_games < COLLECT_GAMES and not self._shutdown_requested:
                    remaining = COLLECT_GAMES - collect_games
                    seeds, async_results = self._submit_batch(seed_pool, max_games=remaining)
                    batch_results = self._collect_batch(seeds, async_results, seed_pool, trainer)
                    for result in batch_results:
                        self._record_game(result)
                        self._log_episode(result)
                        self._save_best_trajectory(result)
                        phase_results.append(result)
                        sweep_games += 1
                        sweep_floors.append(result["floor"])
                        collect_games += 1

                    # Write status after each batch for live monitoring
                    self.write_status({
                        "sweep_config": sweep_config, "sweep_phase": "collecting",
                        "config_name": config_name, "sweep_games": sweep_games,
                        "train_steps": trainer.train_steps,
                        "buffer_size": len(trainer.buffer),
                        "games_per_min": round(games_per_min, 1) if games_per_min else 0,
                        "collect_progress": f"{collect_games}/{COLLECT_GAMES}",
                    })

                collect_duration = time.monotonic() - collect_t0
                games_per_min = collect_games / max(collect_duration / 60.0, 0.01)
                avg_floor_phase = sum(r["floor"] for r in phase_results) / max(len(phase_results), 1)
                logger.info(
                    "  Collected %d games in %.0fs (%.0f g/min), avg floor %.1f, buffer %d",
                    collect_games, collect_duration, games_per_min, avg_floor_phase, len(trainer.buffer),
                )

                if self._shutdown_requested:
                    break

                # -- TRAIN PHASE --
                train_t0 = time.monotonic()
                train_steps_before = trainer.train_steps
                train_count = 0

                # Mix in top trajectory files — always remember the best runs
                from packages.training.strategic_trainer import StrategicTransition
                traj_dir = self.run_dir / "best_trajectories"
                if traj_dir.exists():
                    traj_files = sorted(
                        traj_dir.glob("traj_F*.npz"),
                        key=lambda p: p.stem, reverse=True,
                    )
                    # Take top 10 trajectories (highest floor, most recent)
                    top_trajs = traj_files[:10]
                    mixed_count = 0
                    for tf in top_trajs:
                        try:
                            data = np.load(tf)
                            n_t = len(data["obs"])
                            ep_id = hash(tf.stem) % (2**31)
                            for i in range(n_t):
                                obs_i = data["obs"][i]
                                if obs_i.shape[0] != model.input_dim:
                                    continue
                                mask_i = data["masks"][i]
                                if mask_i.shape[0] < _ACTION_DIM:
                                    mask_i = np.pad(mask_i, (0, _ACTION_DIM - mask_i.shape[0]),
                                                    constant_values=False)
                                st = StrategicTransition(
                                    obs=obs_i,
                                    action_mask=mask_i,
                                    action=int(data["actions"][i]),
                                    reward=float(data["rewards"][i]),
                                    done=bool(data["dones"][i]),
                                    value=float(data["values"][i]),
                                    log_prob=float(data["log_probs"][i]),
                                    episode_id=ep_id,
                                    final_floor=float(data["final_floors"][i]),
                                    cleared_act1=float(data["cleared_act1"][i]),
                                )
                                trainer.buffer.append(st)
                                mixed_count += 1
                        except Exception:
                            continue
                    if mixed_count > 0:
                        logger.info("  Distilled %d transitions from top %d trajectories",
                                    mixed_count, len(top_trajs))

                # Save original epochs, temporarily increase for deeper training
                orig_epochs = trainer.ppo_epochs
                trainer.ppo_epochs = TRAIN_EPOCHS

                logger.info("=== TRAIN phase: %d steps, %d epochs, buffer %d ===",
                            _TRAIN_STEPS, TRAIN_EPOCHS, len(trainer.buffer))
                self.write_status({
                    "sweep_config": sweep_config, "sweep_phase": "training",
                    "config_name": config_name, "sweep_games": sweep_games,
                    "train_steps": trainer.train_steps,
                    "buffer_size": len(trainer.buffer),
                    "games_per_min": round(games_per_min, 1),
                    "total_loss": self._last_train_metrics.get("total_loss"),
                    "policy_loss": self._last_train_metrics.get("policy_loss"),
                    "value_loss": self._last_train_metrics.get("value_loss"),
                    "entropy": self._last_train_metrics.get("entropy"),
                })

                _abort = False
                if algorithm == "grpo":
                    # GRPO: convert buffer transitions to GroupResults, train via GRPOTrainer
                    from .grpo_trainer import GroupSample, GroupResult
                    groups: list = []
                    # Group transitions by episode_id to form rollout groups
                    episode_groups: Dict[int, list] = {}
                    for t in trainer.buffer:
                        episode_groups.setdefault(t.episode_id, []).append(t)
                    for ep_id, transitions in episode_groups.items():
                        samples = []
                        for t in transitions:
                            samples.append(GroupSample(
                                action_idx=t.action,
                                obs=t.obs,
                                action_mask=t.action_mask,
                                log_prob=t.log_prob,
                                total_return=t.final_floor,
                            ))
                        if len(samples) >= 2:
                            group = GroupResult(samples=samples, phase_type="mixed")
                            group.compute_advantages()
                            groups.append(group)
                    if groups:
                        for _ in range(_TRAIN_STEPS):
                            train_metrics = grpo_trainer.train_batch(groups)
                            if math.isnan(train_metrics.get("total_loss", 0)):
                                logger.error("ABORT: NaN loss detected (GRPO)")
                                _abort = True
                                break
                            train_count += 1
                            self._last_train_metrics = {
                                k: v for k, v in train_metrics.items()
                                if isinstance(v, (int, float))
                            }
                        logger.info("GRPO train: %d groups, %d steps", len(groups), train_count)
                else:
                    # PPO (default)
                    for _ in range(_TRAIN_STEPS):
                        if len(trainer.buffer) < trainer.batch_size // 2:
                            break  # Not enough data
                        train_metrics = trainer.train_batch()
                        if math.isnan(train_metrics.get("total_loss", 0)):
                            logger.error("ABORT: NaN loss detected")
                            _abort = True
                            break
                        train_count += 1
                        self._process_train_metrics(
                            train_metrics, trainer, config_name, sweep_floors,
                            games_per_min, best_avg_floor,
                        )

                # Restore original epochs
                trainer.ppo_epochs = orig_epochs

                # Clear buffer after train phase — data has been consumed
                trainer.buffer.clear()

                # Sync weights to inference server
                if self._server is not None and train_count > 0:
                    self._server.sync_strategic_from_pytorch(
                        model, version=trainer.train_steps
                    )

                train_duration = time.monotonic() - train_t0
                steps_done = trainer.train_steps - train_steps_before
                logger.info(
                    "  Trained %d steps in %.1fs (%.1f steps/sec), loss %.4f",
                    steps_done, train_duration,
                    steps_done / max(train_duration, 0.01),
                    self._last_train_metrics.get("total_loss", 0),
                )

                # Checkpoint management
                current_avg = sum(self.recent_floors) / max(len(self.recent_floors), 1)
                if trainer.maybe_checkpoint(current_avg):
                    best_avg_floor = current_avg
                    logger.info("New best avg floor: %.1f", best_avg_floor)
                if self.total_games % 5000 < COLLECT_GAMES:
                    self._check_ascension_bump()

                # Periodic warm checkpoint — also save trainer state for clean shutdown
                _ckpt_extra = {
                    "optimizer_state_dict": trainer.optimizer.state_dict(),
                    "train_steps": trainer.train_steps,
                    "total_games": self.total_games,
                }
                if hasattr(trainer, "scheduler"):
                    _ckpt_extra["scheduler_state_dict"] = trainer.scheduler.state_dict()
                if hasattr(trainer, "entropy_coeff"):
                    _ckpt_extra["entropy_coeff"] = trainer.entropy_coeff
                self._last_trainer_state = _ckpt_extra
                if self.total_games % 2000 < COLLECT_GAMES:
                    model.save(self.run_dir / "periodic_checkpoint.pt", extra=_ckpt_extra)
                    model.save(self.run_dir / "shutdown_checkpoint.pt", extra=_ckpt_extra)

                self.write_status({
                    "sweep_config": sweep_config, "sweep_phase": "adaptive",
                    "config_name": config_name, "sweep_games": sweep_games,
                    "train_steps": trainer.train_steps,
                    "buffer_size": len(trainer.buffer),
                    "replay_buffer": replay_buffer.size,
                    "replay_best_floor": replay_buffer.best_floor,
                    "games_per_min": round(games_per_min, 1),
                    "entropy_coeff": getattr(trainer, "entropy_coeff", None),
                    "total_loss": self._last_train_metrics.get("total_loss"),
                    "policy_loss": self._last_train_metrics.get("policy_loss"),
                    "value_loss": self._last_train_metrics.get("value_loss"),
                    "entropy": self._last_train_metrics.get("entropy"),
                    "floor_pred_loss": self._last_train_metrics.get("floor_pred_loss"),
                    "act_pred_loss": self._last_train_metrics.get("act_pred_loss"),
                    "clip_fraction": self._last_train_metrics.get("clip_fraction"),
                })

                # GC between phases
                gc.collect()

                # Abort criteria — detect training collapse
                if _abort:
                    break
                _clip = self._last_train_metrics.get("clip_fraction", 0)
                _vloss = self._last_train_metrics.get("value_loss", 0)
                _ent = self._last_train_metrics.get("entropy", 1.0)
                if sweep_games > ABORT_GRACE_GAMES and _clip > ABORT_CLIP_FRACTION:
                    logger.warning("ABORT: clip fraction %.3f > %.3f after %d games", _clip, ABORT_CLIP_FRACTION, sweep_games)
                    break
                if sweep_games > ABORT_GRACE_GAMES and _vloss > ABORT_VALUE_LOSS:
                    logger.warning("ABORT: value loss %.3f > %.3f after %d games", _vloss, ABORT_VALUE_LOSS, sweep_games)
                    break
                if _ent < ABORT_ENTROPY_MIN:
                    logger.warning("ABORT: entropy %.4f < %.4f (collapsed)", _ent, ABORT_ENTROPY_MIN)
                    break

            # Final training pass on remaining buffer (PPO only; GRPO handled above)
            if algorithm == "ppo" and hasattr(trainer, "buffer") and len(trainer.buffer) >= trainer.batch_size:
                metrics = trainer.train_batch()
                if self._server is not None:
                    self._server.sync_strategic_from_pytorch(
                        model, version=trainer.train_steps
                    )

            sweep_elapsed = time.monotonic() - sweep_start
            sweep_avg = sum(sweep_floors) / max(len(sweep_floors), 1)

            result_info = {
                "config": sweep_config,
                "games": sweep_games,
                "avg_floor": round(sweep_avg, 1),
                "win_rate": round(sum(self.recent_wins) / max(len(self.recent_wins), 1) * 100, 1),
                "duration_min": round(sweep_elapsed / 60, 1),
                "train_steps": trainer.train_steps,
            }
            self.sweep_results.append(result_info)
            return result_info

        # Phase 1: Explore all configs
        logger.info("=== Phase 1: Exploring %d configs (%d games each) ===",
                     n_configs, phase1_games_per)
        for idx, cfg in enumerate(self.sweep_configs):
            if self._shutdown_requested:
                break
            result = _run_config(idx, cfg, phase1_games_per)
            config_scores[idx] = result

        # With single config, Phase 2+3 just continue training on the same config.
        if not self._shutdown_requested and config_scores:
            remaining = self.max_games - self.total_games
            if remaining > 0:
                best_idx = 0
                best_cfg = self.sweep_configs[best_idx]
                logger.info("=== Continuing training (%d games remaining, replay=%d/%d) ===",
                             remaining, replay_buffer.size, replay_buffer.best_floor)
                _run_config(best_idx, best_cfg, remaining, fork_weights=False)

        # Save checkpoint and clean up
        _warm_state = {"total_games": self.total_games}
        if hasattr(self, '_last_trainer_state'):
            _warm_state.update(self._last_trainer_state)
        if self._shutdown_requested:
            logger.info("Saving warm checkpoint before shutdown...")
            model.save(self.run_dir / "shutdown_checkpoint.pt", extra=_warm_state)
            logger.info("Checkpoint saved to %s", self.run_dir / "shutdown_checkpoint.pt")
        model.save(self.run_dir / "final_strategic.pt", extra=_warm_state)
        self._write_summary()

        # Cleanup inference server and worker pool
        if self._executor is not None:
            self._executor.terminate()
            self._executor.join()
            self._executor = None
        if self._server is not None:
            self._server.stop()
            self._server = None

        if self._shutdown_requested:
            logger.info("Graceful shutdown complete. %d games played.", self.total_games)

        return {
            "total_games": self.total_games,
            "total_wins": self.total_wins,
            "best_avg_floor": best_avg_floor,
            "sweep_results": self.sweep_results,
        }

    def _check_ascension_bump(self) -> None:
        """Check if we should increase ascension based on recent performance."""
        if len(self.recent_floors) < 50:
            return
        avg_floor = sum(self.recent_floors) / len(self.recent_floors)
        wr = sum(self.recent_wins) / max(len(self.recent_wins), 1)
        for min_floor, min_wr, target_asc in ASCENSION_BREAKPOINTS:
            if avg_floor >= min_floor and wr >= min_wr and self.ascension < target_asc:
                logger.info(
                    "Ascension bump: A%d -> A%d (avg_floor=%.1f, WR=%.1f%%)",
                    self.ascension, target_asc, avg_floor, wr * 100,
                )
                self.ascension = target_asc

    def _process_train_metrics(
        self,
        metrics: Dict[str, float],
        trainer,
        config_name: str,
        sweep_floors: Deque[int],
        games_per_min: float,
        best_avg_floor: float,
    ) -> None:
        """Handle post-training bookkeeping: entropy decay, stall detection, logging."""
        self._last_train_metrics = {
            k: v for k, v in metrics.items()
            if isinstance(v, (int, float))
        }

        # Append per-step metrics to perf_log.jsonl for loss curve comparison
        perf_entry = {
            "ts": datetime.now().isoformat(),
            "config_name": config_name,
            "train_step": metrics.get("train_steps", 0),
            "total_games": self.total_games,
            "total_loss": metrics.get("total_loss"),
            "policy_loss": metrics.get("policy_loss"),
            "value_loss": metrics.get("value_loss"),
            "entropy": metrics.get("entropy"),
            "clip_fraction": metrics.get("clip_fraction"),
            "lr": metrics.get("lr"),
            "entropy_coeff": metrics.get("entropy_coeff"),
            "avg_floor": sum(sweep_floors) / max(len(sweep_floors), 1) if sweep_floors else 0,
        }
        try:
            perf_path = self.run_dir / "perf_log.jsonl"
            with open(perf_path, "a") as f:
                f.write(json.dumps(perf_entry) + "\n")
        except OSError:
            pass
        sweep_avg = sum(sweep_floors) / max(len(sweep_floors), 1) if sweep_floors else 0.0
        # Entropy decay only applies to trainers that support it (PPO's StrategicTrainer)
        if hasattr(trainer, "decay_entropy"):
            if sweep_avg > 7.0:
                trainer.decay_entropy(min_coeff=0.02, decay=0.999)
            elif sweep_avg > 5.5:
                trainer.decay_entropy(min_coeff=0.02, decay=0.9999)

        games_since_checkpoint = self.total_games - self._stall_checkpoint_games
        if games_since_checkpoint >= STALL_DETECTION_WINDOW:
            current_avg = sum(self.recent_floors) / max(len(self.recent_floors), 1)
            improvement = current_avg - self._stall_checkpoint_floor
            if improvement < STALL_IMPROVEMENT_THRESHOLD:
                if hasattr(trainer, "entropy_coeff"):
                    old_ent = trainer.entropy_coeff
                    trainer.entropy_coeff = min(0.10, trainer.entropy_coeff + 0.02)
                    logger.warning(
                        "STALL DETECTED: avg floor %.1f -> %.1f over %d games "
                        "(improvement %.1f < %.1f). Entropy bump: %.4f -> %.4f",
                        self._stall_checkpoint_floor, current_avg,
                        games_since_checkpoint, improvement,
                        STALL_IMPROVEMENT_THRESHOLD, old_ent, trainer.entropy_coeff,
                    )
                self._stall_checkpoint_floor = current_avg
                self._stall_checkpoint_games = self.total_games
            else:
                if hasattr(trainer, "entropy_coeff") and trainer.entropy_coeff > 0.05:
                    old_ent = trainer.entropy_coeff
                    trainer.entropy_coeff = max(0.05, trainer.entropy_coeff - 0.005)
                    logger.info(
                        "Floor improving (%.1f -> %.1f): entropy decay %.4f -> %.4f",
                        self._stall_checkpoint_floor, current_avg,
                        old_ent, trainer.entropy_coeff,
                    )
                self._stall_checkpoint_floor = current_avg
                self._stall_checkpoint_games = self.total_games

        avg_floor = sum(self.recent_floors) / max(len(self.recent_floors), 1)
        wr = sum(self.recent_wins) / max(len(self.recent_wins), 1)

        logger.info(
            "[%s] Games %d | Floor %.1f | WR %.1f%% | Loss %.4f | "
            "Ent %.3f | LR %.1e | %.1f g/min",
            config_name, self.total_games, avg_floor, wr * 100,
            metrics.get("total_loss", 0),
            metrics.get("entropy_coeff", 0),
            metrics.get("lr", 0),
            games_per_min,
        )

    def _submit_batch(self, seed_pool, max_games: int = 0) -> Tuple[List[str], List[Any]]:
        """Submit a batch of games to workers. Returns immediately (non-blocking)."""
        batch_size = self.games_per_batch
        if max_games > 0:
            batch_size = min(batch_size, max_games)
        seeds = [seed_pool.get_seed() for _ in range(batch_size)]

        cfg = self._current_sweep_config
        ts_ms = cfg.get("turn_solver_ms", 50.0)
        _strategic = cfg.get("strategic_search", False)
        _mcts = cfg.get("mcts_enabled", False)
        _mcts_card_sims = cfg.get("mcts_card_sims", 0)

        # Mixed temperature: ~25% of games use higher temp for exploration
        explore_temp = self.temperature * EXPLORE_TEMP_MULTIPLIER
        async_results = [
            self._executor.apply_async(
                _play_one_game,
                (seed, self.ascension,
                 explore_temp if i % EXPLORE_GAME_RATIO == 0 else self.temperature,
                 self.total_games,
                 ts_ms,
                 _strategic,
                 _mcts,
                 _mcts_card_sims),
            )
            for i, seed in enumerate(seeds)
        ]
        return seeds, async_results

    def _collect_batch(
        self,
        seeds: List[str],
        async_results: List[Any],
        seed_pool,
        trainer,
    ) -> List[Dict[str, Any]]:
        """Collect results from a previously submitted batch. Blocks until done."""
        results: List[Dict[str, Any]] = []
        for ar, seed in zip(async_results, seeds):
            if self._shutdown_requested:
                break
            try:
                # MCTS deep search: boss fights can take 10+ min, full game up to 1h
                result = ar.get(timeout=3600)
            except Exception as e:
                logger.warning("Game %s failed (%s): %s", seed, type(e).__name__, e)
                result = {
                    "seed": seed, "won": False, "floor": 0, "hp": 0,
                    "decisions": 0, "duration_s": 0.0, "transitions": [],
                }

            self._episode_counter += 1
            ep_id = self._episode_counter
            for t in result.get("transitions", []):
                trainer.add_transition(
                    obs=t["obs"],
                    action_mask=t["action_mask"],
                    action=t["action"],
                    reward=t["reward"],
                    done=t["done"],
                    value=t["value"],
                    log_prob=t["log_prob"],
                    episode_id=ep_id,
                )
                buf_t = trainer.buffer[-1]
                buf_t.final_floor = t["final_floor"]
                buf_t.cleared_act1 = t["cleared_act1"]
                buf_t.cleared_act2 = t["cleared_act2"]
                buf_t.cleared_act3 = t["cleared_act3"]

            seed_pool.record_result(seed, {"won": result["won"], "floor": result["floor"]})
            results.append(result)

            # Add to replay buffer if good enough
            if hasattr(self, "_replay_buffer") and result.get("transitions"):
                self._replay_buffer.maybe_add(
                    result["floor"], result["transitions"], result["won"]
                )

        # Mix in replay transitions (25% of batch size)
        if hasattr(self, "_replay_buffer") and self._replay_buffer.size > 0:
            n_replay = max(1, int(len(results) * REPLAY_MIX_RATIO))
            replay_transitions = self._replay_buffer.sample_transitions(n_replay * 8)
            if replay_transitions:
                current_replay_ep = None
                for t in replay_transitions:
                    if current_replay_ep is None or t.get("done", False):
                        self._episode_counter += 1
                        current_replay_ep = self._episode_counter
                    trainer.add_transition(
                        obs=t["obs"],
                        action_mask=t["action_mask"],
                        action=t["action"],
                        reward=t["reward"],
                        done=t["done"],
                        value=t["value"],
                        log_prob=t["log_prob"],
                        episode_id=current_replay_ep,
                    )
                    buf_t = trainer.buffer[-1]
                    buf_t.final_floor = t["final_floor"]
                    buf_t.cleared_act1 = t["cleared_act1"]
                    buf_t.cleared_act2 = t["cleared_act2"]
                    buf_t.cleared_act3 = t["cleared_act3"]

        return results

    def _play_batch(
        self,
        model,
        encoder,
        seed_pool,
        trainer,
    ) -> List[Dict[str, Any]]:
        """Play a batch of games (blocking). Convenience wrapper around submit/collect."""
        seeds, async_results = self._submit_batch(seed_pool)
        return self._collect_batch(seeds, async_results, seed_pool, trainer)

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


def main():
    """CLI entry point for overnight training."""
    import argparse

    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s | %(levelname)s | %(message)s",
        datefmt="%H:%M:%S",
    )

    parser = argparse.ArgumentParser(description="Overnight training runner")
    parser.add_argument("--workers", type=int, default=TRAIN_WORKERS, help="Number of parallel workers")
    parser.add_argument("--games", type=int, default=50000, help="Maximum total games")
    parser.add_argument("--batch", type=int, default=TRAIN_GAMES_PER_BATCH, help="Games per batch")
    parser.add_argument("--batch-size", type=int, default=TRAIN_BATCH_SIZE, help="PPO mini-batch size")
    parser.add_argument("--ascension", type=int, default=0, help="Ascension level")
    parser.add_argument("--run-dir", type=str, default="logs/active", help="Output directory")
    parser.add_argument("--headless-after", type=int, default=30, help="Go headless after N minutes")
    parser.add_argument("--visual-at", type=str, default="07:30", help="Switch to visual at HH:MM")
    parser.add_argument("--temperature", type=float, default=TEMPERATURE, help="Exploration temperature (0=greedy)")
    parser.add_argument("--resume", type=str, default=None, help="Path to checkpoint .pt to resume from")
    parser.add_argument("--hidden-dim", type=int, default=MODEL_HIDDEN_DIM, help="Model hidden dimension")
    parser.add_argument("--num-blocks", type=int, default=MODEL_NUM_BLOCKS, help="Number of residual blocks")
    parser.add_argument("--max-batch-size", type=int, default=TRAIN_MAX_BATCH_INFERENCE, help="Max inference batch size")
    parser.add_argument("--weekend", action="store_true", help="Use weekend configs (D+E only)")
    parser.add_argument("--overnight", action="store_true", help="Full 5-config ablation sweep")
    parser.add_argument("--max-hours-per-config", type=float, default=None, help="Hours per sweep config (None=auto)")
    args = parser.parse_args()

    if args.overnight:
        sweep = OVERNIGHT_SWEEP_CONFIGS
    elif args.weekend:
        sweep = WEEKEND_SWEEP_CONFIGS
    else:
        sweep = DEFAULT_SWEEP_CONFIGS

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
        "resume_path": args.resume,
        "hidden_dim": args.hidden_dim,
        "num_blocks": args.num_blocks,
        "max_batch_size": args.max_batch_size,
        "sweep_configs": sweep,
        "max_hours_per_config": args.max_hours_per_config,
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
