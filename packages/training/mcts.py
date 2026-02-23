"""
Monte Carlo Tree Search for Slay the Spire.

Uses a neural network policy/value head to guide search.
Designed for self-play improvement on top of BC foundation.
"""

import math
import random
from dataclasses import dataclass, field
from typing import List, Dict, Any, Optional, Callable, Tuple
import numpy as np

@dataclass
class MCTSNode:
    """Node in the MCTS search tree."""
    state: Any  # Game state representation
    parent: Optional['MCTSNode'] = None
    action: Any = None  # Action that led to this state
    children: Dict[Any, 'MCTSNode'] = field(default_factory=dict)

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
    Monte Carlo Tree Search with neural network guidance.

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
            # Flip value for opponent (if applicable)
            # value = 1 - value  # Uncomment for adversarial games

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


class MCTSAgent:
    """
    Agent that uses MCTS with neural network guidance.

    Designed to work with our STS environment.
    """

    def __init__(
        self,
        policy_value_network,  # CardPickerBC or similar
        num_simulations: int = 100,
        temperature: float = 1.0,
    ):
        self.network = policy_value_network
        self.num_simulations = num_simulations
        self.temperature = temperature

        self.mcts = MCTS(
            policy_fn=self._get_policy_value,
            num_simulations=num_simulations,
        )

    def _get_policy_value(self, state) -> Tuple[Dict[Any, float], float]:
        """Get policy and value from neural network."""
        import torch

        # Convert state to tensor
        if isinstance(state, np.ndarray):
            state_tensor = torch.FloatTensor(state)
        else:
            state_tensor = state

        # Get network output
        self.network.eval()
        with torch.no_grad():
            logits = self.network(state_tensor.unsqueeze(0))[0]
            probs = torch.softmax(logits, dim=-1).numpy()

        # Convert to action dict
        action_probs = {i: float(probs[i]) for i in range(len(probs))}

        # Estimate value as max probability (simple heuristic)
        # TODO: Add separate value head to network
        value = float(probs.max())

        return action_probs, value

    def get_action(self, state, legal_actions: List[int]) -> int:
        """Get best action using MCTS."""
        # Simple case: use network directly without full MCTS
        # (Full MCTS requires game simulation which we don't have yet)
        import torch

        if isinstance(state, np.ndarray):
            state_tensor = torch.FloatTensor(state)
        else:
            state_tensor = state

        self.network.eval()
        with torch.no_grad():
            logits = self.network(state_tensor.unsqueeze(0))[0]

            # Mask illegal actions
            mask = torch.zeros_like(logits)
            mask[legal_actions] = 1
            logits = logits.masked_fill(mask == 0, float('-inf'))

            if self.temperature == 0:
                return logits.argmax().item()

            # Sample with temperature
            probs = torch.softmax(logits / self.temperature, dim=-1)
            return torch.multinomial(probs, 1).item()


if __name__ == "__main__":
    # Simple test
    print("MCTS module loaded successfully")

    # Test with random policy
    def random_policy(state):
        actions = {i: 1.0/10 for i in range(10)}
        value = 0.5
        return actions, value

    mcts = MCTS(
        policy_fn=random_policy,
        num_simulations=50,
    )

    # Test search
    visits = mcts.search(
        root_state="test",
        get_legal_actions=lambda s: list(range(10)),
        apply_action=lambda s, a: s,
        is_terminal=lambda s: False,
        get_terminal_value=lambda s: 0.5,
    )

    print(f"Visit distribution: {visits}")
    print(f"Best action: {mcts.select_action(visits, temperature=0)}")
