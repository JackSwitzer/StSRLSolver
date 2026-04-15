"""MCTS with UCB1 for strategic decisions.

At each strategic decision point, runs N simulations:
1. Copy the game state
2. Take an action in the copy
3. Play forward with a rollout policy (first legal action or heuristic)
4. Evaluate terminal state with value head
5. Backpropagate value to update node statistics
6. Select action with most visits (robust action selection)
"""

import logging
import math
import time
from dataclasses import dataclass
from typing import Any, Dict, List, Optional, TYPE_CHECKING

import numpy as np

if TYPE_CHECKING:
    from packages.engine.game import GameRunner

logger = logging.getLogger(__name__)


from .training_config import MCTS_BUDGETS, MCTS_UCB_C

# Maximum rollout steps (prevent infinite loops)
MAX_ROLLOUT_STEPS = 100

# Re-export for tests
UCB_C = MCTS_UCB_C


@dataclass
class MCTSNode:
    """Node in the MCTS tree (one per action at the root)."""
    action_idx: int
    visits: int = 0
    total_value: float = 0.0

    @property
    def mean_value(self) -> float:
        if self.visits == 0:
            return 0.0
        return self.total_value / self.visits

    def ucb1(self, parent_visits: int) -> float:
        """UCB1 score: exploitation + exploration."""
        if self.visits == 0:
            return float('inf')  # Unvisited nodes get priority
        exploit = self.mean_value
        explore = UCB_C * math.sqrt(math.log(parent_visits) / self.visits)
        return exploit + explore


class StrategicMCTS:
    """MCTS engine for strategic decisions.

    Uses GameRunner.copy() to simulate forward from decision points.
    Value estimates come from either the neural value head or
    a heuristic based on game state (HP, floor, deck quality).
    """

    def __init__(self, encoder=None, client=None):
        """
        Args:
            encoder: RunStateEncoder for generating observations
            client: InferenceClient for value head evaluation (optional)
        """
        self.encoder = encoder
        self.client = client
        self._stats = {"total_sims": 0, "total_ms": 0.0}

    def search(
        self,
        runner: 'GameRunner',
        actions: list,
        phase_type: str,
        budget: Optional[int] = None,
        combat_only: bool = False,
    ) -> tuple:
        """Run MCTS and return (best_action_idx, visit_count_policy).

        Args:
            runner: Current game state (will NOT be modified)
            actions: List of available actions
            phase_type: Decision type for budget lookup
            budget: Override simulation count (default from MCTS_BUDGETS)
            combat_only: If True, rollouts stop when combat ends

        Returns:
            (action_idx, policy) where policy is normalized visit counts
        """
        n_actions = len(actions)
        if n_actions <= 1:
            return 0, np.array([1.0])

        n_sims = budget or MCTS_BUDGETS.get(phase_type, 10)

        # Initialize root children
        nodes = [MCTSNode(action_idx=i) for i in range(n_actions)]
        total_visits = 0

        t0 = time.monotonic()

        for sim in range(n_sims):
            # Selection: pick action with highest UCB1
            # (On first n_actions iterations, each action gets one visit)
            if total_visits < n_actions:
                action_idx = sim % n_actions
            else:
                action_idx = max(range(n_actions), key=lambda i: nodes[i].ucb1(total_visits))

            # Simulation: copy game, take action, rollout, evaluate
            try:
                game_copy = runner.copy()
                game_copy.take_action(actions[action_idx])
                value = self._rollout_and_evaluate(game_copy, phase_type, combat_only=combat_only)
            except Exception as e:
                logger.warning("MCTS sim %d failed: %s", sim, e)
                value = 0.0

            # Backpropagation
            nodes[action_idx].visits += 1
            nodes[action_idx].total_value += value
            total_visits += 1

        elapsed_ms = (time.monotonic() - t0) * 1000
        self._stats["total_sims"] += total_visits
        self._stats["total_ms"] += elapsed_ms

        # Select action with most visits (robust selection)
        visits = np.array([n.visits for n in nodes], dtype=np.float32)
        best_idx = int(np.argmax(visits))

        # Policy = normalized visit counts
        policy = visits / visits.sum() if visits.sum() > 0 else np.ones(n_actions) / n_actions

        if logger.isEnabledFor(logging.DEBUG):
            logger.debug(
                "MCTS: %d sims in %.0fms, chose action %d (visits=%s, values=%s)",
                total_visits, elapsed_ms, best_idx,
                [n.visits for n in nodes],
                [f"{n.mean_value:.3f}" for n in nodes],
            )

        return best_idx, policy

    def _rollout_and_evaluate(self, runner: 'GameRunner', phase_type: str, combat_only: bool = False) -> float:
        """Play forward from current state and return value estimate.

        Strategy:
        1. If value head available: play a few steps, then evaluate with NN
        2. Otherwise: play to completion (or MAX_ROLLOUT_STEPS) with heuristic

        When combat_only=True, rollout stops when combat ends (phase changes
        from COMBAT), preventing combat MCTS from simulating entire rest-of-game.
        """
        from packages.engine.game import GamePhase

        # Quick rollout: play forward a few strategic decisions
        rollout_steps = 0
        max_steps = MAX_ROLLOUT_STEPS

        while not runner.game_over and rollout_steps < max_steps:
            try:
                acts = runner.get_available_actions()
            except Exception:
                break
            if not acts:
                break

            phase = runner.phase

            # Combat-only mode: stop when combat ends
            if combat_only and phase != GamePhase.COMBAT:
                break

            if phase == GamePhase.COMBAT:
                # In combat: random action for rollout diversity
                runner.take_action(acts[np.random.randint(len(acts))])
                rollout_steps += 1
                continue

            if len(acts) == 1:
                runner.take_action(acts[0])
                rollout_steps += 1
                continue

            # Strategic decision during rollout
            # If we have a value head, evaluate here and return
            if self.client is not None and self.encoder is not None:
                value = self._value_head_eval(runner, acts, phase)
                if value is not None:
                    return value

            # Otherwise: take a random action and continue rollout
            idx = np.random.randint(len(acts))
            runner.take_action(acts[idx])
            rollout_steps += 1

        # Terminal evaluation: heuristic based on game state
        return self._heuristic_value(runner)

    def _value_head_eval(self, runner: 'GameRunner', actions: list, phase) -> Optional[float]:
        """Evaluate position using neural value head."""
        from packages.engine.game import GamePhase

        _PHASE_MAP = {
            GamePhase.MAP_NAVIGATION: "path",
            GamePhase.COMBAT_REWARDS: "card_pick",
            GamePhase.BOSS_REWARDS: "card_pick",
            GamePhase.REST: "rest",
            GamePhase.SHOP: "shop",
            GamePhase.EVENT: "event",
        }
        phase_type = _PHASE_MAP.get(phase, "other")

        try:
            obs = self.encoder.encode(
                runner.run_state, phase_type=phase_type,
                boss_name=getattr(runner, "_boss_name", ""),
                room_type=getattr(runner, "current_room_type", ""),
                actions=actions, runner=runner,
            )
            resp = self.client.infer_strategic(obs, len(actions))
            if resp and resp.get("ok"):
                return float(resp["value"])
        except Exception as e:
            logger.warning("MCTS value head eval failed: %s", e)

        return None

    def _heuristic_value(self, runner: 'GameRunner') -> float:
        """Heuristic game state evaluation when value head unavailable.

        Returns value in [0, 1] range:
        - Floor progress: floor/55
        - HP bonus: current_hp/max_hp * 0.2
        - Win = 1.0, Death early = scaled by progress
        """
        rs = runner.run_state
        floor = getattr(rs, "floor", 0)
        progress = floor / 55.0

        if runner.game_won:
            return 1.0
        if runner.game_over:
            return progress * 0.5  # Partial credit for progress

        hp_ratio = getattr(rs, "current_hp", 0) / max(getattr(rs, "max_hp", 80), 1)
        return min(1.0, progress + hp_ratio * 0.2)

    @property
    def stats(self) -> Dict[str, Any]:
        """Return cumulative MCTS statistics."""
        return dict(self._stats)
