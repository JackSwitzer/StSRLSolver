"""Training loop for StS RL agent.

Implements REINFORCE with baseline (generalized advantage estimation) using
the StsEnv gymnasium wrapper and the numpy PolicyValueNet.

This is the simplest viable training loop.  Upgrade path:
1. Swap PolicyValueNet numpy backend for torch (same interface).
2. Replace REINFORCE with PPO (clipped surrogate).
3. Add StsVecEnv for parallel data collection.

Usage:
    # CLI
    uv run python -m packages.training.train --episodes 100 --envs 1

    # Python
    from packages.training.train import Trainer
    trainer = Trainer(num_envs=1, learning_rate=3e-4)
    trainer.train(total_episodes=100)
"""

from __future__ import annotations

import argparse
import time
from pathlib import Path
from typing import Any, Dict, List, Optional

import numpy as np

from .episode_logger import EpisodeLog, EpisodeStep
from .gym_env import StsEnv
from .policy_net import PolicyValueNet


class RolloutBuffer:
    """Stores trajectory data from a single episode for training."""

    def __init__(self) -> None:
        self.observations: List[np.ndarray] = []
        self.actions: List[int] = []
        self.rewards: List[float] = []
        self.values: List[float] = []
        self.action_masks: List[np.ndarray] = []
        self.dones: List[bool] = []

    def add(
        self,
        obs: np.ndarray,
        action: int,
        reward: float,
        value: float,
        action_mask: np.ndarray,
        done: bool,
    ) -> None:
        self.observations.append(obs.copy())
        self.actions.append(action)
        self.rewards.append(reward)
        self.values.append(value)
        self.action_masks.append(action_mask.copy())
        self.dones.append(done)

    def __len__(self) -> int:
        return len(self.rewards)

    def compute_returns_and_advantages(
        self,
        gamma: float = 0.99,
        gae_lambda: float = 0.95,
        last_value: float = 0.0,
    ) -> tuple[np.ndarray, np.ndarray]:
        """Compute GAE advantages and discounted returns.

        Args:
            gamma: Discount factor.
            gae_lambda: GAE lambda for bias-variance tradeoff.
            last_value: Bootstrap value for the last state (0 if terminal).

        Returns:
            (returns, advantages) each of shape (n_steps,).
        """
        n = len(self.rewards)
        advantages = np.zeros(n, dtype=np.float32)
        returns = np.zeros(n, dtype=np.float32)

        next_value = last_value
        gae = 0.0

        for t in reversed(range(n)):
            if self.dones[t]:
                next_value = 0.0
                gae = 0.0

            delta = self.rewards[t] + gamma * next_value - self.values[t]
            gae = delta + gamma * gae_lambda * gae
            advantages[t] = gae
            returns[t] = advantages[t] + self.values[t]
            next_value = self.values[t]

        return returns, advantages

    def to_arrays(self) -> Dict[str, np.ndarray]:
        """Convert buffer to numpy arrays for training."""
        return {
            "observations": np.array(self.observations, dtype=np.float32),
            "actions": np.array(self.actions, dtype=np.int64),
            "rewards": np.array(self.rewards, dtype=np.float32),
            "values": np.array(self.values, dtype=np.float32),
            "action_masks": np.array(self.action_masks, dtype=np.bool_),
        }


class Trainer:
    """Trains a policy on StS using REINFORCE with baseline.

    Uses a single StsEnv for simplicity.  For parallel collection,
    use StsVecEnv directly or run multiple Trainer instances.

    Attributes:
        policy: The PolicyValueNet being trained.
        env: The StsEnv instance.
        gamma: Discount factor.
        gae_lambda: GAE lambda.
        learning_rate: SGD step size.
        entropy_coeff: Entropy regularization coefficient.
    """

    def __init__(
        self,
        num_envs: int = 1,
        obs_dim: int = 1186,
        action_dim: int = 2048,
        hidden_dim: int = 256,
        num_layers: int = 3,
        learning_rate: float = 3e-4,
        gamma: float = 0.99,
        gae_lambda: float = 0.95,
        entropy_coeff: float = 0.01,
        value_coeff: float = 0.5,
        max_steps_per_episode: int = 3000,
        seed: Optional[str] = None,
        ascension: int = 20,
        checkpoint_dir: str = "checkpoints",
        log_dir: str = "episodes",
    ) -> None:
        self.num_envs = num_envs  # reserved for future vectorized use
        self.learning_rate = learning_rate
        self.gamma = gamma
        self.gae_lambda = gae_lambda
        self.entropy_coeff = entropy_coeff
        self.value_coeff = value_coeff
        self.max_steps = max_steps_per_episode
        self.checkpoint_dir = checkpoint_dir
        self.log_dir = log_dir

        # Single env for now
        self.env = StsEnv(
            seed=seed,
            ascension=ascension,
            max_steps=max_steps_per_episode,
        )

        self.policy = PolicyValueNet(
            obs_dim=obs_dim,
            action_dim=action_dim,
            hidden_dim=hidden_dim,
            num_layers=num_layers,
        )

        # Metrics tracking
        self._episode_rewards: List[float] = []
        self._episode_lengths: List[int] = []
        self._episode_wins: List[bool] = []
        self._episode_floors: List[int] = []

    def collect_episode(self) -> tuple[RolloutBuffer, EpisodeLog]:
        """Collect a single episode of experience.

        Returns:
            (buffer, episode_log) tuple.
        """
        buffer = RolloutBuffer()

        obs, info = self.env.reset()
        steps: List[EpisodeStep] = []
        total_reward = 0.0
        hp_history = [info.get("hp", 0)]
        last_floor = info.get("floor", 0)
        start_time = time.time()

        for step_i in range(self.max_steps):
            mask = info["action_mask"]
            if not mask.any():
                break

            # Get action and value from policy
            _, value = self.policy.forward(obs)
            action_index = self.policy.predict_action(obs, mask)

            # Resolve action ID
            action_id = "unknown"
            aid = self.env.action_space_manager.index_to_action_id(action_index)
            if aid is not None:
                action_id = aid

            # Step environment
            obs_next, reward, terminated, truncated, info = self.env.step(action_index)
            done = terminated or truncated

            # Store in buffer
            buffer.add(obs, action_index, reward, value, mask, done)

            # Log step
            steps.append(EpisodeStep(
                observation=[],  # Don't store obs in log (too large)
                action_index=action_index,
                action_id=action_id,
                reward=reward,
                phase=info.get("phase", "unknown"),
                floor=info.get("floor", last_floor),
            ))
            total_reward += reward

            # HP tracking
            current_floor = info.get("floor", last_floor)
            if current_floor != last_floor:
                hp_history.append(info.get("hp", 0))
                last_floor = current_floor

            obs = obs_next

            if done:
                hp_history.append(info.get("final_hp", info.get("hp", 0)))
                break

        won = info.get("game_won", False)
        floors_reached = info.get("final_floor", last_floor)
        duration = time.time() - start_time

        episode_log = EpisodeLog(
            seed=str(getattr(self.env, "seed_val", "unknown")),
            ascension=self.env.ascension,
            character=self.env.character,
            won=won,
            total_reward=total_reward,
            floors_reached=floors_reached,
            steps=steps,
            hp_history=hp_history,
            duration_seconds=round(duration, 3),
            timestamp=time.strftime("%Y-%m-%dT%H:%M:%S"),
        )

        return buffer, episode_log

    def collect_episodes(self, n_episodes: int = 1) -> tuple[List[RolloutBuffer], List[EpisodeLog]]:
        """Collect multiple episodes.

        Args:
            n_episodes: Number of episodes to collect.

        Returns:
            (buffers, episode_logs) tuple of lists.
        """
        buffers: List[RolloutBuffer] = []
        logs: List[EpisodeLog] = []

        for _ in range(n_episodes):
            buf, log = self.collect_episode()
            buffers.append(buf)
            logs.append(log)

        return buffers, logs

    def train_step(self, buffers: List[RolloutBuffer]) -> Dict[str, float]:
        """Update policy from collected buffers.

        Merges all buffers, computes advantages, and applies a single
        gradient step.

        Args:
            buffers: List of RolloutBuffer from collect_episodes.

        Returns:
            Dict with loss metrics.
        """
        # Merge all buffers
        all_obs = []
        all_actions = []
        all_returns = []
        all_advantages = []
        all_masks = []

        for buf in buffers:
            if len(buf) == 0:
                continue

            returns, advantages = buf.compute_returns_and_advantages(
                gamma=self.gamma,
                gae_lambda=self.gae_lambda,
            )

            arrays = buf.to_arrays()
            all_obs.append(arrays["observations"])
            all_actions.append(arrays["actions"])
            all_returns.append(returns)
            all_advantages.append(advantages)
            all_masks.append(arrays["action_masks"])

        if not all_obs:
            return {"policy_loss": 0.0, "value_loss": 0.0, "entropy": 0.0, "total_loss": 0.0}

        obs_batch = np.concatenate(all_obs)
        actions_batch = np.concatenate(all_actions)
        returns_batch = np.concatenate(all_returns)
        advantages_batch = np.concatenate(all_advantages)
        masks_batch = np.concatenate(all_masks)

        # Normalize advantages
        adv_std = advantages_batch.std()
        if adv_std > 1e-8:
            advantages_batch = (advantages_batch - advantages_batch.mean()) / adv_std

        # Apply gradient update
        metrics = self.policy.apply_gradients(
            obs_batch=obs_batch,
            actions=actions_batch,
            advantages=advantages_batch,
            returns=returns_batch,
            action_masks=masks_batch,
            learning_rate=self.learning_rate,
            entropy_coeff=self.entropy_coeff,
            value_coeff=self.value_coeff,
        )

        return metrics

    def train(
        self,
        total_episodes: int = 1000,
        episodes_per_batch: int = 1,
        log_every: int = 10,
        save_every: int = 100,
        save_episodes: bool = False,
    ) -> Dict[str, List[float]]:
        """Main training loop.

        Args:
            total_episodes: Total number of episodes to train on.
            episodes_per_batch: Episodes to collect before each gradient step.
            log_every: Print metrics every N episodes.
            save_every: Save checkpoint every N episodes.
            save_episodes: Whether to save episode JSONL logs.

        Returns:
            Dict of metric histories.
        """
        history: Dict[str, List[float]] = {
            "reward": [],
            "length": [],
            "win_rate": [],
            "floor": [],
            "policy_loss": [],
            "value_loss": [],
            "entropy": [],
        }

        episode_count = 0
        batch_num = 0

        print(f"Starting training: {total_episodes} episodes, "
              f"batch_size={episodes_per_batch}, lr={self.learning_rate}")
        print(f"Policy: {self.policy.param_count} parameters")
        print("-" * 60)

        while episode_count < total_episodes:
            n = min(episodes_per_batch, total_episodes - episode_count)
            buffers, logs = self.collect_episodes(n)

            # Train on collected data
            metrics = self.train_step(buffers)

            # Record metrics
            for log in logs:
                self._episode_rewards.append(log.total_reward)
                self._episode_lengths.append(len(log.steps))
                self._episode_wins.append(log.won)
                self._episode_floors.append(log.floors_reached)

                history["reward"].append(log.total_reward)
                history["length"].append(len(log.steps))
                history["win_rate"].append(float(log.won))
                history["floor"].append(log.floors_reached)

                if save_episodes:
                    log.save_compact(f"{self.log_dir}/episodes.jsonl")

            history["policy_loss"].append(metrics["policy_loss"])
            history["value_loss"].append(metrics["value_loss"])
            history["entropy"].append(metrics["entropy"])

            episode_count += n
            batch_num += 1

            # Logging
            if batch_num % log_every == 0 or episode_count >= total_episodes:
                recent = min(log_every * episodes_per_batch, len(self._episode_rewards))
                avg_reward = np.mean(self._episode_rewards[-recent:])
                avg_length = np.mean(self._episode_lengths[-recent:])
                win_rate = np.mean(self._episode_wins[-recent:])
                avg_floor = np.mean(self._episode_floors[-recent:])

                print(
                    f"Episode {episode_count}/{total_episodes} | "
                    f"Reward: {avg_reward:.3f} | "
                    f"Length: {avg_length:.0f} | "
                    f"Win: {win_rate:.1%} | "
                    f"Floor: {avg_floor:.1f} | "
                    f"PLoss: {metrics['policy_loss']:.4f} | "
                    f"VLoss: {metrics['value_loss']:.4f} | "
                    f"Ent: {metrics['entropy']:.4f}"
                )

            # Checkpointing
            if episode_count % save_every == 0 or episode_count >= total_episodes:
                ckpt_path = f"{self.checkpoint_dir}/policy_{episode_count}.npz"
                self.policy.save(ckpt_path)

        print("-" * 60)
        print(f"Training complete. {episode_count} episodes.")
        final_win_rate = np.mean(self._episode_wins[-100:]) if self._episode_wins else 0.0
        print(f"Final win rate (last 100): {final_win_rate:.1%}")

        return history


def main() -> None:
    """CLI entry point."""
    parser = argparse.ArgumentParser(description="Train StS RL agent")
    parser.add_argument("--episodes", type=int, default=100, help="Total episodes")
    parser.add_argument("--batch-size", type=int, default=1, help="Episodes per gradient step")
    parser.add_argument("--lr", type=float, default=3e-4, help="Learning rate")
    parser.add_argument("--gamma", type=float, default=0.99, help="Discount factor")
    parser.add_argument("--entropy", type=float, default=0.01, help="Entropy coefficient")
    parser.add_argument("--hidden", type=int, default=256, help="Hidden layer dim")
    parser.add_argument("--layers", type=int, default=3, help="Number of hidden layers")
    parser.add_argument("--max-steps", type=int, default=3000, help="Max steps per episode")
    parser.add_argument("--ascension", type=int, default=20, help="Ascension level")
    parser.add_argument("--seed", type=str, default=None, help="Fixed seed (None = random)")
    parser.add_argument("--log-every", type=int, default=10, help="Log interval")
    parser.add_argument("--save-every", type=int, default=100, help="Checkpoint interval")
    parser.add_argument("--save-episodes", action="store_true", help="Save episode logs")
    parser.add_argument("--checkpoint-dir", type=str, default="checkpoints")
    parser.add_argument("--log-dir", type=str, default="episodes")
    args = parser.parse_args()

    trainer = Trainer(
        learning_rate=args.lr,
        gamma=args.gamma,
        entropy_coeff=args.entropy,
        hidden_dim=args.hidden,
        num_layers=args.layers,
        max_steps_per_episode=args.max_steps,
        ascension=args.ascension,
        seed=args.seed,
        checkpoint_dir=args.checkpoint_dir,
        log_dir=args.log_dir,
    )

    trainer.train(
        total_episodes=args.episodes,
        episodes_per_batch=args.batch_size,
        log_every=args.log_every,
        save_every=args.save_every,
        save_episodes=args.save_episodes,
    )


if __name__ == "__main__":
    main()
