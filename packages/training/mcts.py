"""
Monte Carlo Tree Search for Slay the Spire.

Provides two MCTS implementations:
1. MCTS - Generic MCTS with neural network policy/value guidance (AlphaZero-style)
2. CombatMCTS - Specialized for StS combat using CombatEngine.copy()

Uses UCB1 with policy prior:
    UCB = Q(s,a) + c_puct * P(s,a) * sqrt(N(s)) / (1 + N(s,a))
"""

from __future__ import annotations

import math
import random
from dataclasses import dataclass, field
from typing import List, Dict, Any, Optional, Callable, Tuple, Union
import numpy as np


@dataclass
class MCTSNode:
    """Node in the MCTS search tree."""
    state: Any  # Game state representation (CombatEngine for CombatMCTS)
    parent: Optional[MCTSNode] = None
    action: Any = None  # Action that led to this state
    children: Dict[Any, MCTSNode] = field(default_factory=dict)

    # Statistics
    visits: int = 0
    value_sum: float = 0.0
    prior: float = 0.0  # Prior probability from policy network

    @property
    def value(self) -> float:
        """Mean value across visits."""
        if self.visits == 0:
            return 0.0
        return self.value_sum / self.visits

    @property
    def is_expanded(self) -> bool:
        return len(self.children) > 0


class MCTS:
    """
    Generic Monte Carlo Tree Search with neural network guidance.

    Uses UCB1 with policy prior (similar to AlphaZero):
    UCB = Q(s,a) + c_puct * P(s,a) * sqrt(N(s)) / (1 + N(s,a))
    """

    def __init__(
        self,
        policy_fn: Callable[[Any], Tuple[Dict[Any, float], float]],
        c_puct: float = 1.4,
        num_simulations: int = 100,
        dirichlet_alpha: float = 0.3,
        dirichlet_epsilon: float = 0.25,
    ):
        """
        Args:
            policy_fn: Function that takes state, returns (action_probs, value)
            c_puct: Exploration constant
            num_simulations: Number of MCTS simulations per move
            dirichlet_alpha: Dirichlet noise alpha for exploration
            dirichlet_epsilon: Weight of Dirichlet noise
        """
        self.policy_fn = policy_fn
        self.c_puct = c_puct
        self.num_simulations = num_simulations
        self.dirichlet_alpha = dirichlet_alpha
        self.dirichlet_epsilon = dirichlet_epsilon

    def search(
        self,
        root_state: Any,
        get_legal_actions: Callable[[Any], List[Any]],
        apply_action: Callable[[Any, Any], Any],
        is_terminal: Callable[[Any], bool],
        get_terminal_value: Callable[[Any], float],
        add_noise: bool = True,
    ) -> Dict[Any, float]:
        """
        Run MCTS from root state.

        Args:
            root_state: Current game state
            get_legal_actions: Function to get legal actions from state
            apply_action: Function to apply action and get next state
            is_terminal: Check if state is terminal (game over)
            get_terminal_value: Get value of terminal state (win=1, loss=0)
            add_noise: Whether to add Dirichlet noise at root

        Returns:
            Dictionary of action -> visit proportion
        """
        root = MCTSNode(state=root_state)

        # Expand root
        self._expand(root, get_legal_actions, add_noise=add_noise)

        # Run simulations
        for _ in range(self.num_simulations):
            node = root
            search_path = [node]

            # Selection: traverse to leaf
            while node.is_expanded and not is_terminal(node.state):
                action, node = self._select_child(node)
                search_path.append(node)

            # Expansion and evaluation
            if is_terminal(node.state):
                value = get_terminal_value(node.state)
            else:
                # Expand node
                self._expand(node, get_legal_actions, add_noise=False)
                # Get value from neural network
                _, value = self.policy_fn(node.state)

            # Backpropagation
            self._backpropagate(search_path, value)

        # Return action visit counts (normalized)
        total_visits = sum(child.visits for child in root.children.values())
        if total_visits == 0:
            # Uniform if no visits
            actions = list(root.children.keys())
            return {a: 1.0 / len(actions) for a in actions}

        return {
            action: child.visits / total_visits
            for action, child in root.children.items()
        }

    def _expand(
        self,
        node: MCTSNode,
        get_legal_actions: Callable[[Any], List[Any]],
        add_noise: bool = False,
    ):
        """Expand a node by adding children for all legal actions."""
        legal_actions = get_legal_actions(node.state)
        if not legal_actions:
            return

        # Get policy from neural network
        action_probs, _ = self.policy_fn(node.state)

        # Filter to legal actions and normalize
        priors = {}
        for action in legal_actions:
            priors[action] = action_probs.get(action, 1e-6)

        # Normalize
        total = sum(priors.values())
        if total > 0:
            priors = {a: p / total for a, p in priors.items()}
        else:
            # Uniform if all zeros
            priors = {a: 1.0 / len(legal_actions) for a in legal_actions}

        # Add Dirichlet noise at root for exploration
        if add_noise and self.dirichlet_epsilon > 0:
            noise = np.random.dirichlet([self.dirichlet_alpha] * len(legal_actions))
            for i, action in enumerate(legal_actions):
                priors[action] = (
                    (1 - self.dirichlet_epsilon) * priors[action]
                    + self.dirichlet_epsilon * noise[i]
                )

        # Create child nodes
        for action, prior in priors.items():
            child = MCTSNode(
                state=None,  # Lazy - computed on first visit
                parent=node,
                action=action,
                prior=prior,
            )
            node.children[action] = child

    def _select_child(self, node: MCTSNode) -> Tuple[Any, MCTSNode]:
        """Select best child using PUCT formula."""
        best_score = float('-inf')
        best_action = None
        best_child = None

        sqrt_parent_visits = math.sqrt(node.visits)

        for action, child in node.children.items():
            # PUCT formula
            if child.visits == 0:
                q_value = 0
            else:
                q_value = child.value

            u_value = (
                self.c_puct * child.prior * sqrt_parent_visits / (1 + child.visits)
            )
            score = q_value + u_value

            if score > best_score:
                best_score = score
                best_action = action
                best_child = child

        return best_action, best_child

    def _backpropagate(self, search_path: List[MCTSNode], value: float):
        """Backpropagate value through the search path."""
        for node in reversed(search_path):
            node.visits += 1
            node.value_sum += value

    def select_action(
        self,
        action_visits: Dict[Any, float],
        temperature: float = 1.0,
    ) -> Any:
        """
        Select action based on visit counts.

        Args:
            action_visits: Dictionary of action -> visit proportion
            temperature: Temperature for sampling (0 = greedy, 1 = proportional)

        Returns:
            Selected action
        """
        if temperature == 0:
            # Greedy selection
            return max(action_visits.keys(), key=lambda a: action_visits[a])

        # Sample proportionally with temperature
        actions = list(action_visits.keys())
        visits = np.array([action_visits[a] for a in actions])

        # Apply temperature
        if temperature != 1.0:
            visits = visits ** (1.0 / temperature)

        # Normalize and sample
        probs = visits / visits.sum()
        return np.random.choice(actions, p=probs)


# =============================================================================
# CombatMCTS - Engine-integrated MCTS for StS combat
# =============================================================================

class CombatMCTS:
    """
    MCTS specialized for Slay the Spire combat using CombatEngine.copy().

    Unlike the generic MCTS which takes callable interfaces, CombatMCTS
    operates directly on CombatEngine instances. Each tree node holds a
    full engine copy so that legal actions, state transitions, and
    terminal checks are all handled through the engine API.

    Value function can be provided externally (e.g. from a neural network)
    or defaults to a heuristic rollout evaluator.
    """

    def __init__(
        self,
        policy_fn: Optional[Callable] = None,
        num_simulations: int = 128,
        c_puct: float = 1.4,
        max_rollout_turns: int = 5,
    ):
        """
        Args:
            policy_fn: Optional function CombatEngine -> (action_priors, value).
                       action_priors maps Action -> float prior probability.
                       If None, uniform priors and heuristic rollout are used.
            num_simulations: Number of MCTS simulations per search call.
            c_puct: Exploration constant for PUCT formula.
            max_rollout_turns: Maximum turns for heuristic rollout evaluation.
        """
        self.policy_fn = policy_fn
        self.num_simulations = num_simulations
        self.c_puct = c_puct
        self.max_rollout_turns = max_rollout_turns

    def search(self, engine: Any) -> Dict[Any, float]:
        """
        Run MCTS from current combat state.

        Args:
            engine: CombatEngine instance (will be copied, not mutated).

        Returns:
            Mapping of Action -> visit proportion for each legal action
            at the root.
        """
        root = MCTSNode(state=engine.copy())

        # Expand root
        self._expand(root)

        if not root.children:
            return {}

        for _ in range(self.num_simulations):
            # Selection: walk tree to a leaf
            node = root
            search_path = [node]

            while node.is_expanded and not node.state.is_combat_over():
                node = self._select(node)
                search_path.append(node)

            # Evaluate leaf
            if node.state.is_combat_over():
                value = self._terminal_value(node.state)
            else:
                self._expand(node)
                value = self._evaluate(node)

            # Backpropagation
            for n in reversed(search_path):
                n.visits += 1
                n.value_sum += value

        return self._get_action_probabilities(root)

    # -----------------------------------------------------------------
    # Internal helpers
    # -----------------------------------------------------------------

    def _expand(self, node: MCTSNode) -> None:
        """Expand node by creating children for all legal actions."""
        actions = node.state.get_legal_actions()
        if not actions:
            return

        # Get priors
        if self.policy_fn is not None:
            action_priors, _ = self.policy_fn(node.state)
        else:
            action_priors = {}

        # Build prior map, fall back to uniform
        priors: Dict[Any, float] = {}
        for action in actions:
            priors[action] = action_priors.get(action, 1e-6)

        total = sum(priors.values())
        if total > 0:
            priors = {a: p / total for a, p in priors.items()}
        else:
            priors = {a: 1.0 / len(actions) for a in actions}

        for action in actions:
            child_engine = node.state.copy()
            child_engine.execute_action(action)
            child = MCTSNode(
                state=child_engine,
                parent=node,
                action=action,
                prior=priors.get(action, 1.0 / len(actions)),
            )
            node.children[action] = child

    def _select(self, node: MCTSNode) -> MCTSNode:
        """Select best child using PUCT formula."""
        best_score = float('-inf')
        best_child = None

        sqrt_parent = math.sqrt(node.visits) if node.visits > 0 else 1.0

        for child in node.children.values():
            q = child.value if child.visits > 0 else 0.0
            u = self.c_puct * child.prior * sqrt_parent / (1 + child.visits)
            score = q + u
            if score > best_score:
                best_score = score
                best_child = child

        return best_child

    def _evaluate(self, node: MCTSNode) -> float:
        """
        Evaluate a leaf node.

        If a policy_fn was provided that returns a value estimate, use that.
        Otherwise fall back to a heuristic rollout.
        """
        if self.policy_fn is not None:
            _, value = self.policy_fn(node.state)
            return value
        return self._rollout_value(node.state)

    def _rollout_value(self, engine: Any) -> float:
        """
        Quick heuristic evaluation of a combat state.

        Returns value in [0, 1] where 1 = player wins with full HP,
        0 = player dead.
        """
        if engine.is_combat_over():
            return self._terminal_value(engine)

        state = engine.state
        player_hp = state.player.hp
        player_max_hp = state.player.max_hp

        # Sum total enemy HP remaining
        total_enemy_hp = sum(max(0, e.hp) for e in state.enemies if e.hp > 0)
        total_enemy_max = sum(max(1, e.max_hp) for e in state.enemies)

        if total_enemy_hp <= 0:
            # All enemies dead, high value
            return 0.8 + 0.2 * (player_hp / max(player_max_hp, 1))

        # Ratio of enemy HP destroyed as progress indicator
        hp_ratio = player_hp / max(player_max_hp, 1)
        enemy_progress = 1.0 - (total_enemy_hp / max(total_enemy_max, 1))

        # Combine: surviving with high HP while killing enemies is good
        value = 0.3 * hp_ratio + 0.5 * enemy_progress

        # Bonus for current block
        block = state.player.block
        if block > 0:
            value += 0.05 * min(block / 20.0, 1.0)

        # Penalty for being in Wrath with enemies alive
        if state.stance == "Wrath" and total_enemy_hp > 0:
            value -= 0.1

        return max(0.0, min(1.0, value))

    def _terminal_value(self, engine: Any) -> float:
        """Value of a terminal combat state."""
        if engine.is_victory():
            hp = engine.state.player.hp
            max_hp = engine.state.player.max_hp
            return 0.8 + 0.2 * (hp / max(max_hp, 1))
        return 0.0  # Defeat

    def _get_action_probabilities(self, root: MCTSNode) -> Dict[Any, float]:
        """Convert root child visit counts to a probability distribution."""
        total = sum(c.visits for c in root.children.values())
        if total == 0:
            n = len(root.children)
            return {a: 1.0 / n for a in root.children}
        return {
            action: child.visits / total
            for action, child in root.children.items()
        }

    def select_action(
        self,
        action_probs: Dict[Any, float],
        temperature: float = 0.0,
    ) -> Any:
        """
        Select an action from the visit distribution.

        Args:
            action_probs: action -> visit proportion mapping from search().
            temperature: 0 = greedy (best action), >0 = stochastic sampling.

        Returns:
            Selected Action object.
        """
        if not action_probs:
            raise ValueError("No actions to select from")

        if temperature == 0:
            return max(action_probs, key=action_probs.get)

        actions = list(action_probs.keys())
        weights = np.array([action_probs[a] for a in actions])
        if temperature != 1.0:
            weights = weights ** (1.0 / temperature)
        probs = weights / weights.sum()
        idx = np.random.choice(len(actions), p=probs)
        return actions[idx]
