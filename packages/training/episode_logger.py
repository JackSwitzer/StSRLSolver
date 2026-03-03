"""Log episode trajectories for training and analysis.

Collects full observation/action/reward trajectories from StsEnv rollouts
and saves them as JSONL files for offline training, behavioral cloning,
and post-hoc analysis.

Usage:
    from packages.training.episode_logger import EpisodeLog

    log = EpisodeLog.from_env_rollout(env, policy)
    log.save("episodes/run_001.jsonl")

    # Load back
    logs = EpisodeLog.load("episodes/run_001.jsonl")
"""

from __future__ import annotations

import json
import time
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any, Callable, Dict, List, Optional, Protocol

import numpy as np


class PolicyProtocol(Protocol):
    """Protocol for policy objects used by EpisodeLog.from_env_rollout."""

    def predict_action(
        self,
        obs: np.ndarray,
        action_mask: np.ndarray,
    ) -> int: ...


@dataclass
class EpisodeStep:
    """A single step within an episode."""

    observation: List[float]
    action_index: int
    action_id: str
    reward: float
    phase: str
    floor: int

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> EpisodeStep:
        return cls(**d)


@dataclass
class EpisodeLog:
    """Full episode trajectory with metadata."""

    seed: str
    ascension: int
    character: str
    won: bool
    total_reward: float
    floors_reached: int
    steps: List[EpisodeStep] = field(default_factory=list)
    hp_history: List[int] = field(default_factory=list)
    duration_seconds: float = 0.0
    timestamp: str = ""

    def to_dict(self) -> Dict[str, Any]:
        d = {
            "seed": self.seed,
            "ascension": self.ascension,
            "character": self.character,
            "won": self.won,
            "total_reward": self.total_reward,
            "floors_reached": self.floors_reached,
            "hp_history": self.hp_history,
            "duration_seconds": self.duration_seconds,
            "timestamp": self.timestamp,
            "num_steps": len(self.steps),
        }
        return d

    def save(self, path: str) -> None:
        """Save episode as JSONL (metadata line + one line per step).

        Args:
            path: File path for output. Parent directories are created.
        """
        p = Path(path)
        p.parent.mkdir(parents=True, exist_ok=True)

        with open(p, "a") as f:
            # First line: episode metadata
            f.write(json.dumps(self.to_dict()) + "\n")
            # Subsequent lines: individual steps
            for step in self.steps:
                f.write(json.dumps(step.to_dict()) + "\n")

    def save_compact(self, path: str) -> None:
        """Save episode as a single JSONL line (no per-step obs).

        Useful for aggregate statistics without storing full trajectories.
        """
        p = Path(path)
        p.parent.mkdir(parents=True, exist_ok=True)

        with open(p, "a") as f:
            f.write(json.dumps(self.to_dict()) + "\n")

    @classmethod
    def load(cls, path: str) -> List[EpisodeLog]:
        """Load episodes from a JSONL file.

        Each episode starts with a metadata line (has 'seed' key) followed
        by step lines (have 'action_index' key).

        Returns:
            List of EpisodeLog objects.
        """
        p = Path(path)
        if not p.exists():
            return []

        episodes: List[EpisodeLog] = []
        current_episode: Optional[EpisodeLog] = None
        current_steps: List[EpisodeStep] = []

        with open(p) as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                d = json.loads(line)

                if "seed" in d and "action_index" not in d:
                    # This is an episode metadata line
                    if current_episode is not None:
                        current_episode.steps = current_steps
                        episodes.append(current_episode)

                    current_episode = cls(
                        seed=d["seed"],
                        ascension=d["ascension"],
                        character=d["character"],
                        won=d["won"],
                        total_reward=d["total_reward"],
                        floors_reached=d["floors_reached"],
                        hp_history=d.get("hp_history", []),
                        duration_seconds=d.get("duration_seconds", 0.0),
                        timestamp=d.get("timestamp", ""),
                    )
                    current_steps = []

                elif "action_index" in d:
                    # This is a step line
                    current_steps.append(EpisodeStep.from_dict(d))

        if current_episode is not None:
            current_episode.steps = current_steps
            episodes.append(current_episode)

        return episodes

    @classmethod
    def from_env_rollout(
        cls,
        env: Any,
        policy: PolicyProtocol,
        max_steps: int = 3000,
        seed: Optional[str] = None,
        record_obs: bool = True,
    ) -> EpisodeLog:
        """Collect a full episode by running a policy on an environment.

        Args:
            env: StsEnv instance.
            policy: Object with predict_action(obs, mask) -> int method.
            max_steps: Maximum steps before truncation.
            seed: Optional seed for reset. If None, uses env default.
            record_obs: Whether to record full observation vectors in steps.
                Set to False for compact logging.

        Returns:
            Completed EpisodeLog.
        """
        start_time = time.time()

        reset_kwargs: Dict[str, Any] = {}
        if seed is not None:
            reset_kwargs["seed"] = int(seed) if seed.isdigit() else hash(seed) % (2**31)

        obs, info = env.reset(**reset_kwargs)

        ep_seed = getattr(env, "seed_val", seed or "unknown")
        ep_ascension = getattr(env, "ascension", 0)
        ep_character = getattr(env, "character", "Watcher")

        steps: List[EpisodeStep] = []
        hp_history: List[int] = [info.get("hp", 0)]
        total_reward = 0.0
        last_floor = info.get("floor", 0)

        for step_i in range(max_steps):
            mask = info["action_mask"]

            if not mask.any():
                break

            action_index = policy.predict_action(obs, mask)

            # Resolve action ID for logging
            action_id = "unknown"
            if hasattr(env, "action_space_manager"):
                aid = env.action_space_manager.index_to_action_id(action_index)
                if aid is not None:
                    action_id = aid

            obs_next, reward, terminated, truncated, info = env.step(action_index)

            step = EpisodeStep(
                observation=obs.tolist() if record_obs else [],
                action_index=action_index,
                action_id=action_id,
                reward=reward,
                phase=info.get("phase", "unknown"),
                floor=info.get("floor", last_floor),
            )
            steps.append(step)
            total_reward += reward

            # Track HP at floor transitions
            current_floor = info.get("floor", last_floor)
            if current_floor != last_floor:
                hp_history.append(info.get("hp", 0))
                last_floor = current_floor

            obs = obs_next

            if terminated or truncated:
                hp_history.append(info.get("final_hp", info.get("hp", 0)))
                break

        won = info.get("game_won", False) if "game_won" in info else False
        floors_reached = info.get("final_floor", last_floor)
        duration = time.time() - start_time

        return cls(
            seed=str(ep_seed),
            ascension=ep_ascension,
            character=ep_character,
            won=won,
            total_reward=total_reward,
            floors_reached=floors_reached,
            steps=steps,
            hp_history=hp_history,
            duration_seconds=round(duration, 3),
            timestamp=time.strftime("%Y-%m-%dT%H:%M:%S"),
        )


def collect_episodes(
    env: Any,
    policy: PolicyProtocol,
    n_episodes: int = 10,
    max_steps: int = 3000,
    record_obs: bool = True,
) -> List[EpisodeLog]:
    """Collect multiple episodes sequentially.

    Args:
        env: StsEnv instance (reused across episodes).
        policy: Policy with predict_action method.
        n_episodes: Number of episodes to collect.
        max_steps: Max steps per episode.
        record_obs: Whether to store full observation arrays.

    Returns:
        List of EpisodeLog objects.
    """
    episodes: List[EpisodeLog] = []
    for _ in range(n_episodes):
        ep = EpisodeLog.from_env_rollout(
            env, policy, max_steps=max_steps, record_obs=record_obs,
        )
        episodes.append(ep)
    return episodes
