"""Milestone detector for training monitoring.

Watches training metrics and emits structured milestone events to
logs/active/milestones.jsonl. Integrates with training_runner.py's
main loop -- call check() after each collect phase.

Milestones:
  - peak_floor: new highest avg_floor_100 or peak_floor
  - first_win: first game with won=True
  - win_rate: 1%, 5%, 10%, 25%, 50% thresholds crossed
  - game_count: 1k, 5k, 10k milestones
  - health_entropy: entropy < 0.02
  - health_value_loss: value_loss > 5.0 or 3x spike
  - throughput: games_per_min drops 50%
  - disk: free < 10GB (warn), < 5GB (critical), < 3GB (emergency pause)
"""

from __future__ import annotations

import json
import logging
import os
import shutil
import signal
import time
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, Optional, Set

logger = logging.getLogger(__name__)

# Win rate thresholds to detect (percent)
_WIN_RATE_THRESHOLDS = [1, 5, 10, 25, 50]

# Game count milestones
_GAME_COUNT_MILESTONES = [1000, 5000, 10000, 25000, 50000, 100000]

# Disk thresholds in GB
_DISK_WARN_GB = 10
_DISK_CRITICAL_GB = 5
_DISK_EMERGENCY_GB = 3


class MilestoneDetector:
    """Detects training milestones and writes them to milestones.jsonl."""

    def __init__(self, run_dir: Path, pid_file: Optional[str] = None):
        self.run_dir = run_dir
        self.milestones_path = run_dir / "milestones.jsonl"
        self.pid_file = pid_file or ".run/training.pid"

        # Tracking state
        self._best_avg_floor: float = 0.0
        self._best_peak_floor: int = 0
        self._first_win_seen: bool = False
        self._win_rate_crossed: Set[int] = set()
        self._game_milestones_crossed: Set[int] = set()
        self._prev_games_per_min: float = 0.0
        self._prev_value_loss: float = 0.0
        self._disk_warned: Set[str] = set()  # "warn", "critical", "emergency"

    def check(
        self,
        total_games: int,
        total_wins: int,
        avg_floor_100: float,
        peak_floor: int,
        win_rate_100: float,
        games_per_min: float,
        entropy: Optional[float] = None,
        value_loss: Optional[float] = None,
        **kwargs: Any,
    ) -> None:
        """Run all milestone checks. Call after each collect phase."""
        self._check_peak_floor(avg_floor_100, peak_floor, total_games)
        self._check_first_win(total_wins, total_games)
        self._check_win_rate(win_rate_100, total_games)
        self._check_game_count(total_games)
        self._check_health(entropy, value_loss, total_games)
        self._check_throughput(games_per_min, total_games)
        self._check_disk(total_games)

    def _check_peak_floor(
        self, avg_floor: float, peak_floor: int, total_games: int
    ) -> None:
        if avg_floor > self._best_avg_floor:
            self._best_avg_floor = avg_floor
            self._emit(
                "peak_floor",
                value=avg_floor,
                games=total_games,
                severity="info",
                detail="new best avg_floor_100",
            )
        if peak_floor > self._best_peak_floor:
            self._best_peak_floor = peak_floor
            self._emit(
                "peak_floor",
                value=peak_floor,
                games=total_games,
                severity="info",
                detail="new peak_floor",
            )

    def _check_first_win(self, total_wins: int, total_games: int) -> None:
        if not self._first_win_seen and total_wins > 0:
            self._first_win_seen = True
            self._emit(
                "first_win",
                value=total_wins,
                games=total_games,
                severity="critical",
                detail=f"First win after {total_games} games",
            )

    def _check_win_rate(self, win_rate_100: float, total_games: int) -> None:
        for threshold in _WIN_RATE_THRESHOLDS:
            if threshold not in self._win_rate_crossed and win_rate_100 >= threshold:
                self._win_rate_crossed.add(threshold)
                self._emit(
                    "win_rate",
                    value=win_rate_100,
                    games=total_games,
                    severity="info" if threshold < 25 else "critical",
                    detail=f"Crossed {threshold}% win rate",
                )

    def _check_game_count(self, total_games: int) -> None:
        for milestone in _GAME_COUNT_MILESTONES:
            if milestone not in self._game_milestones_crossed and total_games >= milestone:
                self._game_milestones_crossed.add(milestone)
                self._emit(
                    "game_count",
                    value=total_games,
                    games=total_games,
                    severity="info",
                    detail=f"Reached {milestone} games",
                )

    def _check_health(
        self,
        entropy: Optional[float],
        value_loss: Optional[float],
        total_games: int,
    ) -> None:
        if entropy is not None and entropy < 0.02:
            self._emit(
                "health_entropy",
                value=entropy,
                games=total_games,
                severity="warn",
                detail=f"Entropy collapsed: {entropy:.4f} < 0.02",
            )
        if value_loss is not None:
            if value_loss > 5.0:
                self._emit(
                    "health_value_loss",
                    value=value_loss,
                    games=total_games,
                    severity="warn",
                    detail=f"Value loss high: {value_loss:.3f} > 5.0",
                )
            # Detect 3x spike
            if (
                self._prev_value_loss > 0
                and value_loss > self._prev_value_loss * 3
            ):
                self._emit(
                    "health_value_loss",
                    value=value_loss,
                    games=total_games,
                    severity="warn",
                    detail=f"Value loss 3x spike: {value_loss:.3f} (prev {self._prev_value_loss:.3f})",
                )
            self._prev_value_loss = value_loss

    def _check_throughput(self, games_per_min: float, total_games: int) -> None:
        if (
            self._prev_games_per_min > 0
            and games_per_min > 0
            and games_per_min < self._prev_games_per_min * 0.5
        ):
            self._emit(
                "throughput",
                value=games_per_min,
                games=total_games,
                severity="warn",
                detail=f"Throughput dropped 50%+: {games_per_min:.1f} g/min (was {self._prev_games_per_min:.1f})",
            )
        if games_per_min > 0:
            self._prev_games_per_min = games_per_min

    def _check_disk(self, total_games: int) -> None:
        """Check free disk space. Emergency pause if < 3GB."""
        try:
            usage = shutil.disk_usage("/")
            free_gb = usage.free / (1024**3)
        except Exception:
            return

        if free_gb < _DISK_EMERGENCY_GB and "emergency" not in self._disk_warned:
            self._disk_warned.add("emergency")
            self._emit(
                "disk",
                value=round(free_gb, 1),
                games=total_games,
                severity="critical",
                detail=f"EMERGENCY: {free_gb:.1f}GB free < {_DISK_EMERGENCY_GB}GB -- pausing training",
            )
            self._emergency_pause()
        elif free_gb < _DISK_CRITICAL_GB and "critical" not in self._disk_warned:
            self._disk_warned.add("critical")
            self._emit(
                "disk",
                value=round(free_gb, 1),
                games=total_games,
                severity="critical",
                detail=f"Disk critical: {free_gb:.1f}GB free < {_DISK_CRITICAL_GB}GB",
            )
        elif free_gb < _DISK_WARN_GB and "warn" not in self._disk_warned:
            self._disk_warned.add("warn")
            self._emit(
                "disk",
                value=round(free_gb, 1),
                games=total_games,
                severity="warn",
                detail=f"Disk low: {free_gb:.1f}GB free < {_DISK_WARN_GB}GB",
            )

    def _emergency_pause(self) -> None:
        """Kill the training process if disk is critically low."""
        try:
            pid_path = Path(self.pid_file)
            if pid_path.exists():
                pid = int(pid_path.read_text().strip())
                logger.critical("Emergency disk pause: killing PID %d", pid)
                os.kill(pid, signal.SIGTERM)
            else:
                # We're in-process -- set shutdown flag via SIGTERM to self
                logger.critical("Emergency disk pause: sending SIGTERM to self")
                os.kill(os.getpid(), signal.SIGTERM)
        except Exception as e:
            logger.error("Emergency pause failed: %s", e)

    def _emit(self, type_: str, **fields: Any) -> None:
        """Append a milestone event to milestones.jsonl."""
        event = {
            "type": type_,
            "ts": datetime.now().isoformat(),
            **fields,
        }
        try:
            with open(self.milestones_path, "a") as f:
                f.write(json.dumps(event) + "\n")
        except Exception as e:
            logger.warning("Failed to write milestone: %s", e)

        severity = fields.get("severity", "info")
        detail = fields.get("detail", "")
        if severity == "critical":
            logger.warning("MILESTONE [%s] %s: %s", severity, type_, detail)
        else:
            logger.info("MILESTONE [%s] %s: %s", severity, type_, detail)
