"""Gymnasium environment wrapper for Slay the Spire.

Provides a standard Gymnasium-compatible interface for RL training against
the STS engine. Observations are flat float32 arrays from ObservationEncoder,
actions are integer indices into an ActionSpace, and the reward signal
combines HP efficiency with win/loss outcomes.

Usage:
    import gymnasium as gym
    from packages.training.gym_env import StsEnv

    env = StsEnv(seed="TEST123", ascension=20)
    obs, info = env.reset()
    while True:
        mask = info["action_mask"]
        action = pick_action_from_mask(mask)
        obs, reward, terminated, truncated, info = env.step(action)
        if terminated or truncated:
            break

Vectorized usage:
    from packages.training.gym_env import StsVecEnv

    venv = StsVecEnv(num_envs=8, ascension=20)
    obs_batch, info_batch = venv.reset()
    # obs_batch: (8, obs_dim), info_batch: list of 8 info dicts
"""

from __future__ import annotations

import multiprocessing as mp
from typing import Any, Dict, List, Optional, Tuple

import gymnasium as gym
import numpy as np
from gymnasium import spaces

from packages.engine.game import GamePhase, GameRunner
from packages.engine.rl_masks import ActionSpace
from packages.engine.rl_observations import ObservationEncoder


# Upper bound on discrete action indices. The ActionSpace grows lazily but
# Gymnasium's Discrete space requires a fixed upper bound at construction.
_MAX_ACTION_INDEX = 2048


class StsEnv(gym.Env):
    """Slay the Spire environment for RL training.

    Observation: flat float32 array from ObservationEncoder.
    Action: integer index into ActionSpace.
    Reward: HP efficiency per decision, scaled by win/loss outcome.

    The action mask is exposed in ``info["action_mask"]`` so RL algorithms
    (PPO, MaskablePPO, etc.) can mask invalid actions.
    """

    metadata = {"render_modes": ["human", "ansi"]}

    def __init__(
        self,
        seed: Optional[str] = None,
        ascension: int = 20,
        character: str = "Watcher",
        render_mode: Optional[str] = None,
        max_steps: int = 3000,
    ) -> None:
        super().__init__()
        self.seed_val = seed
        self.ascension = ascension
        self.character = character
        self.render_mode = render_mode
        self.max_steps = max_steps

        # Shared encoder and action space across all episodes.
        self.encoder = ObservationEncoder()
        self.action_space_manager = ActionSpace()

        # Gymnasium spaces.
        self.observation_space = spaces.Box(
            low=-np.inf,
            high=np.inf,
            shape=(self.encoder.size,),
            dtype=np.float32,
        )
        self.action_space = spaces.Discrete(_MAX_ACTION_INDEX)

        # Episode state (set on reset).
        self.runner: Optional[GameRunner] = None
        self._current_actions: List[Dict[str, Any]] = []
        self._step_count: int = 0
        self._hp_before: int = 0
        self._max_hp: int = 1

    # ------------------------------------------------------------------
    # Gymnasium core API
    # ------------------------------------------------------------------

    def reset(
        self,
        *,
        seed: Optional[int] = None,
        options: Optional[Dict[str, Any]] = None,
    ) -> Tuple[np.ndarray, Dict[str, Any]]:
        """Reset the environment and start a new episode.

        Args:
            seed: Optional integer seed (converted to string for the engine).
            options: Unused, reserved for future extension.

        Returns:
            (observation, info) tuple.
        """
        super().reset(seed=seed)

        if seed is not None:
            self.seed_val = str(seed)
        elif self.seed_val is None:
            self.seed_val = str(self.np_random.integers(0, 2**31))

        self.runner = GameRunner(
            seed=self.seed_val,
            ascension=self.ascension,
            character=self.character,
            verbose=False,
        )
        self._step_count = 0
        self._hp_before = self.runner.run_state.current_hp
        self._max_hp = max(self.runner.run_state.max_hp, 1)

        obs = self._get_obs()
        info = self._get_info()
        return obs, info

    def step(
        self, action_index: int
    ) -> Tuple[np.ndarray, float, bool, bool, Dict[str, Any]]:
        """Execute an action by index.

        Two-step selections (card selection, stance selection) are surfaced
        as normal actions -- the ActionSpace/mask already enumerates the
        follow-up selection options. The agent picks one index and we
        forward it to the engine.

        Args:
            action_index: Integer index into the action space.

        Returns:
            Standard Gymnasium 5-tuple (obs, reward, terminated, truncated, info).
        """
        assert self.runner is not None, "Call reset() before step()"
        self._step_count += 1

        # Snapshot HP before the action for reward computation.
        hp_before = self.runner.run_state.current_hp
        max_hp = max(self.runner.run_state.max_hp, 1)

        # Map integer index to an engine action dict.
        action_dict = self.action_space_manager.index_to_action(
            action_index, self._current_actions
        )

        if action_dict is None:
            # Invalid action index -- penalise and re-expose the same state.
            obs = self._get_obs()
            info = self._get_info()
            return obs, -0.01, False, False, info

        # Execute the action.
        result = self.runner.take_action_dict(action_dict)

        # If the engine returned "selection required", the pending_selection
        # is now set. The *next* call to _get_info will reflect the follow-up
        # selection actions in the mask, so the agent handles it naturally as
        # a new step.

        # Compute reward.
        reward = self._compute_reward(hp_before, max_hp, result)

        # Terminal conditions.
        terminated = self.runner.game_over
        truncated = self._step_count >= self.max_steps and not terminated

        obs = self._get_obs()
        info = self._get_info()

        # Attach terminal metadata.
        if terminated:
            info["game_won"] = self.runner.game_won
            info["game_lost"] = self.runner.game_lost
            info["final_floor"] = self.runner.run_state.floor
            info["final_hp"] = self.runner.run_state.current_hp

        return obs, reward, terminated, truncated, info

    def render(self) -> Optional[str]:
        """Render the environment (ansi mode returns a string)."""
        if self.runner is None:
            return None
        if self.render_mode == "ansi":
            return self._render_ansi()
        if self.render_mode == "human":
            text = self._render_ansi()
            print(text)
            return None
        return None

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    def _get_obs(self) -> np.ndarray:
        """Get current observation as numpy array."""
        assert self.runner is not None
        obs_dict = self.runner.get_observation()
        return self.encoder.observation_to_array(obs_dict)

    def _get_info(self) -> Dict[str, Any]:
        """Build info dict with action mask and metadata."""
        assert self.runner is not None
        self._current_actions = self.runner.get_available_action_dicts()

        # Register all actions and build mask.
        self.action_space_manager.register_actions(self._current_actions)
        mask = self._build_padded_mask()

        phase = self.runner.phase
        return {
            "action_mask": mask,
            "available_actions": self._current_actions,
            "num_actions": len(self._current_actions),
            "phase": phase.name if hasattr(phase, "name") else str(phase),
            "floor": self.runner.run_state.floor,
            "act": self.runner.run_state.act,
            "hp": self.runner.run_state.current_hp,
            "max_hp": self.runner.run_state.max_hp,
        }

    def _build_padded_mask(self) -> np.ndarray:
        """Build a boolean mask padded to _MAX_ACTION_INDEX."""
        raw = self.action_space_manager.actions_to_mask(self._current_actions)
        mask = np.zeros(_MAX_ACTION_INDEX, dtype=np.bool_)
        n = min(len(raw), _MAX_ACTION_INDEX)
        mask[:n] = raw[:n]
        return mask

    def _compute_reward(
        self,
        hp_before: int,
        max_hp: int,
        result: Dict[str, Any],
    ) -> float:
        """Compute the reward signal for a single step.

        Reward shaping:
        - Per-step cost: -0.001 (encourages shorter games)
        - HP change: (hp_after - hp_before) / max_hp (combat efficiency)
        - Win bonus: +1.0
        - Loss penalty: -0.5
        """
        reward = -0.001  # step cost

        hp_after = self.runner.run_state.current_hp
        hp_delta = (hp_after - hp_before) / max(max_hp, 1)
        reward += hp_delta

        if self.runner.game_over:
            if self.runner.game_won:
                reward += 1.0
            elif self.runner.game_lost:
                reward -= 0.5

        return reward

    def _render_ansi(self) -> str:
        """Produce a compact text representation of the current state."""
        if self.runner is None:
            return "<no game>"
        rs = self.runner.run_state
        phase = self.runner.phase
        lines = [
            f"[Step {self._step_count}] Phase: {phase.name}  "
            f"Floor: {rs.floor}  Act: {rs.act}",
            f"HP: {rs.current_hp}/{rs.max_hp}  Gold: {rs.gold}  "
            f"Deck: {len(rs.deck)} cards",
        ]
        if self.runner.current_combat is not None:
            cs = self.runner.current_combat.state
            enemies = ", ".join(
                f"{e.id}({e.hp}/{e.max_hp})" for e in cs.enemies
            )
            lines.append(
                f"Combat: energy={cs.energy} hand={len(cs.hand)} "
                f"enemies=[{enemies}]"
            )
        lines.append(f"Actions: {len(self._current_actions)} available")
        return "\n".join(lines)


# ======================================================================
# Vectorized environment
# ======================================================================


def _worker_fn(
    conn: mp.connection.Connection,
    env_kwargs: Dict[str, Any],
) -> None:
    """Worker process for StsVecEnv.

    Protocol:
    - ("reset", None) -> (obs, info)
    - ("step", action_index) -> (obs, reward, terminated, truncated, info)
    - ("close", None) -> exits
    """
    env = StsEnv(**env_kwargs)
    try:
        while True:
            cmd, data = conn.recv()
            if cmd == "reset":
                result = env.reset()
                conn.send(result)
            elif cmd == "step":
                result = env.step(data)
                conn.send(result)
            elif cmd == "get_info":
                conn.send(env._get_info())
            elif cmd == "close":
                break
            else:
                raise ValueError(f"Unknown command: {cmd}")
    finally:
        conn.close()


class StsVecEnv:
    """Vectorized STS environment running N games in parallel via multiprocessing.

    Each sub-environment runs in its own process. Communication uses
    ``multiprocessing.Pipe`` for minimal overhead.

    Usage:
        venv = StsVecEnv(num_envs=8, ascension=20)
        obs_batch, info_batch = venv.reset()
        for _ in range(1000):
            actions = pick_actions(obs_batch, info_batch)
            obs_batch, rewards, terminateds, truncateds, info_batch = venv.step(actions)
        venv.close()
    """

    def __init__(self, num_envs: int = 8, **env_kwargs: Any) -> None:
        self.num_envs = num_envs
        self.env_kwargs = env_kwargs
        self._closed = False

        # For each worker: (parent_conn, child_conn), process
        self._parent_conns: List[mp.connection.Connection] = []
        self._child_conns: List[mp.connection.Connection] = []
        self._processes: List[mp.Process] = []

        for i in range(num_envs):
            parent_conn, child_conn = mp.Pipe()
            # Give each env a unique seed offset if no seed specified.
            kw = dict(env_kwargs)
            if "seed" not in kw or kw["seed"] is None:
                kw["seed"] = str(i * 1000 + 42)
            p = mp.Process(
                target=_worker_fn,
                args=(child_conn, kw),
                daemon=True,
            )
            p.start()
            child_conn.close()  # Parent doesn't use child end.
            self._parent_conns.append(parent_conn)
            self._processes.append(p)

    def reset(self) -> Tuple[np.ndarray, List[Dict[str, Any]]]:
        """Reset all environments.

        Returns:
            (obs_batch, info_batch) where obs_batch has shape (num_envs, obs_dim).
        """
        for conn in self._parent_conns:
            conn.send(("reset", None))

        obs_list = []
        info_list = []
        for conn in self._parent_conns:
            obs, info = conn.recv()
            obs_list.append(obs)
            info_list.append(info)

        return np.stack(obs_list), info_list

    def step(
        self, actions: np.ndarray
    ) -> Tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray, List[Dict[str, Any]]]:
        """Step all environments in parallel.

        Args:
            actions: Array of shape (num_envs,) with action indices.

        Returns:
            (obs_batch, rewards, terminateds, truncateds, info_batch).
        """
        assert len(actions) == self.num_envs

        for conn, action in zip(self._parent_conns, actions):
            conn.send(("step", int(action)))

        obs_list = []
        reward_list = []
        terminated_list = []
        truncated_list = []
        info_list = []

        for i, conn in enumerate(self._parent_conns):
            obs, reward, terminated, truncated, info = conn.recv()
            # Auto-reset terminated/truncated envs.
            if terminated or truncated:
                conn.send(("reset", None))
                obs, reset_info = conn.recv()
                info["terminal_observation"] = info.get("terminal_observation", obs)
                info.update({
                    k: v for k, v in reset_info.items()
                    if k in ("action_mask", "available_actions", "num_actions")
                })
            obs_list.append(obs)
            reward_list.append(reward)
            terminated_list.append(terminated)
            truncated_list.append(truncated)
            info_list.append(info)

        return (
            np.stack(obs_list),
            np.array(reward_list, dtype=np.float32),
            np.array(terminated_list, dtype=np.bool_),
            np.array(truncated_list, dtype=np.bool_),
            info_list,
        )

    def close(self) -> None:
        """Shut down all worker processes."""
        if self._closed:
            return
        self._closed = True
        for conn in self._parent_conns:
            try:
                conn.send(("close", None))
            except (BrokenPipeError, OSError):
                pass
            conn.close()
        for p in self._processes:
            p.join(timeout=5)
            if p.is_alive():
                p.terminate()

    def __del__(self) -> None:
        self.close()
