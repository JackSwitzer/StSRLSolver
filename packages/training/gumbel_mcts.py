"""
Gumbel MuZero: guaranteed policy improvement with low simulation budgets.

Instead of UCB/PUCT selection, uses Gumbel-Top-k sampling + sequential halving:
1. Sample k candidate actions using Gumbel noise added to policy logits
2. Sequential halving: allocate simulations, eliminate bottom half, repeat
3. Final action has guaranteed policy improvement over raw network

Key advantage: works reliably with as few as 16 simulations (standard MCTS
fails below 64). This means 4x more games/hour for the same compute.

Reference: "Policy Improvement by Planning with Gumbel" (Danihelka et al., 2022)
"""

from __future__ import annotations

from dataclasses import dataclass
import math
from typing import Any, Callable, Dict, List, Optional, Tuple

import numpy as np


@dataclass(frozen=True)
class RootActionStats:
    """Summary statistics for a root action from the last search."""

    action: Any
    visits: int
    probability: float
    q_value: float
    prior: float


@dataclass(frozen=True)
class SearchSummary:
    """Root-level summary from the most recent Gumbel search."""

    num_simulations: int
    root_value: float
    actions: Tuple[RootActionStats, ...]


class GumbelMCTS:
    """Gumbel MuZero search with sequential halving.

    Uses CombatEngine.copy() as the forward model (same as CombatMCTS).
    Compatible with the same policy_fn interface.
    """

    def __init__(
        self,
        policy_fn: Optional[Callable] = None,
        num_simulations: int = 16,
        c_visit: float = 50.0,
        c_scale: float = 1.0,
        max_candidates: int = 16,
    ):
        """
        Args:
            policy_fn: Optional function CombatEngine -> (action_priors, value).
            num_simulations: Total simulation budget.
            c_visit: Visit count scaling for completed Q-values.
            c_scale: Scale factor for completed Q-values.
            max_candidates: Maximum initial candidates (k) for Gumbel sampling.
        """
        self.policy_fn = policy_fn
        self.num_simulations = num_simulations
        self.c_visit = c_visit
        self.c_scale = c_scale
        self.max_candidates = max_candidates
        self.last_search_summary: Optional[SearchSummary] = None

    def search(self, engine: Any) -> Dict[Any, float]:
        """Run Gumbel MuZero search from current combat state.

        Args:
            engine: CombatEngine instance (will be copied, not mutated).

        Returns:
            Mapping of Action -> visit proportion for the root.
        """
        root_state = engine.copy()
        legal_actions = root_state.get_legal_actions()

        if not legal_actions:
            self.last_search_summary = None
            return {}
        if len(legal_actions) == 1:
            self.last_search_summary = SearchSummary(
                num_simulations=1,
                root_value=1.0,
                actions=(
                    RootActionStats(
                        action=legal_actions[0],
                        visits=1,
                        probability=1.0,
                        q_value=1.0,
                        prior=1.0,
                    ),
                ),
            )
            return {legal_actions[0]: 1.0}

        # Get prior policy and root value
        if self.policy_fn is not None:
            action_priors, root_value = self.policy_fn(root_state)
        else:
            action_priors = {}
            root_value = 0.0

        # Normalize priors
        priors = {}
        for a in legal_actions:
            priors[a] = action_priors.get(a, 1e-6)
        total = sum(priors.values())
        if total > 0:
            priors = {a: p / total for a, p in priors.items()}
        else:
            priors = {a: 1.0 / len(legal_actions) for a in legal_actions}

        # Step 1: Gumbel-Top-k sampling to select candidates
        k = min(self.max_candidates, len(legal_actions), self.num_simulations)
        candidates = self._gumbel_top_k(legal_actions, priors, k)

        # Step 2: Sequential halving to allocate simulations
        visit_counts: Dict[Any, int] = {a: 0 for a in candidates}
        q_values: Dict[Any, float] = {a: 0.0 for a in candidates}
        q_sums: Dict[Any, float] = {a: 0.0 for a in candidates}

        remaining = list(candidates)
        budget = self.num_simulations
        num_phases = max(1, int(math.log2(len(remaining))))

        for phase in range(num_phases):
            if len(remaining) <= 1:
                break

            # Allocate budget evenly across remaining candidates
            sims_per_action = max(1, budget // (len(remaining) * (num_phases - phase)))

            for action in remaining:
                for _ in range(sims_per_action):
                    # Simulate: copy state, execute action, evaluate
                    child_state = root_state.copy()
                    child_state.execute_action(action)

                    if child_state.is_combat_over():
                        value = self._terminal_value(child_state)
                    elif self.policy_fn is not None:
                        _, value = self.policy_fn(child_state)
                    else:
                        value = self._heuristic_value(child_state)

                    visit_counts[action] += 1
                    q_sums[action] += value
                    q_values[action] = q_sums[action] / visit_counts[action]

                    budget -= 1
                    if budget <= 0:
                        break
                if budget <= 0:
                    break

            if budget <= 0:
                break

            # Halving: keep top half by completed Q-value
            scored = [(self._completed_q(q_values[a], visit_counts[a], priors[a]), a)
                      for a in remaining]
            scored.sort(reverse=True)
            half = max(1, len(scored) // 2)
            remaining = [a for _, a in scored[:half]]

        # Use any remaining budget on the top candidates
        while budget > 0 and remaining:
            for action in remaining:
                if budget <= 0:
                    break
                child_state = root_state.copy()
                child_state.execute_action(action)

                if child_state.is_combat_over():
                    value = self._terminal_value(child_state)
                elif self.policy_fn is not None:
                    _, value = self.policy_fn(child_state)
                else:
                    value = self._heuristic_value(child_state)

                visit_counts[action] += 1
                q_sums[action] += value
                q_values[action] = q_sums[action] / visit_counts[action]
                budget -= 1

        # Build visit distribution
        total_visits = sum(visit_counts.values())
        if total_visits == 0:
            probs = {a: 1.0 / len(legal_actions) for a in legal_actions}
        else:
            probs = {
            action: count / total_visits
            for action, count in visit_counts.items()
            if count > 0
        }

        self.last_search_summary = SearchSummary(
            num_simulations=self.num_simulations,
            root_value=float(root_value),
            actions=tuple(
                RootActionStats(
                    action=action,
                    visits=visit_counts.get(action, 0),
                    probability=float(probs.get(action, 0.0)),
                    q_value=float(q_values.get(action, 0.0)),
                    prior=float(priors.get(action, 0.0)),
                )
                for action in sorted(probs, key=probs.get, reverse=True)
            ),
        )
        return probs

    def select_action(
        self,
        action_probs: Dict[Any, float],
        temperature: float = 0.0,
    ) -> Any:
        """Select action from visit distribution."""
        if not action_probs:
            raise ValueError("No actions to select from")

        if temperature == 0:
            return max(action_probs, key=action_probs.get)

        actions = list(action_probs.keys())
        weights = np.array([action_probs[a] for a in actions])
        if temperature != 1.0:
            weights = weights ** (1.0 / temperature)
        probs = weights / weights.sum()
        return np.random.choice(actions, p=probs)

    def _gumbel_top_k(
        self,
        actions: List[Any],
        priors: Dict[Any, float],
        k: int,
    ) -> List[Any]:
        """Select top-k actions using Gumbel noise for exploration."""
        # Add Gumbel(0,1) noise to log-priors, take top-k
        log_priors = np.array([math.log(max(priors[a], 1e-10)) for a in actions])
        gumbel_noise = np.random.gumbel(size=len(actions))
        perturbed = log_priors + gumbel_noise

        # Top-k indices
        top_k_idx = np.argpartition(perturbed, -k)[-k:]
        top_k_idx = top_k_idx[np.argsort(perturbed[top_k_idx])[::-1]]

        return [actions[i] for i in top_k_idx]

    def _completed_q(self, q: float, visits: int, prior: float) -> float:
        """Completed Q-value for sequential halving scoring."""
        if visits == 0:
            return -float('inf')
        # Mix Q-value with prior bonus (decreases with visits)
        prior_bonus = self.c_scale * prior / (1 + visits / self.c_visit)
        return q + prior_bonus

    def _terminal_value(self, engine: Any) -> float:
        """Value of a terminal combat state."""
        if engine.is_victory():
            hp = engine.state.player.hp
            max_hp = max(engine.state.player.max_hp, 1)
            return 0.7 + 0.3 * (hp / max_hp)
        return 0.0

    def _heuristic_value(self, engine: Any) -> float:
        """Heuristic evaluation when no policy_fn is available."""
        if engine.is_combat_over():
            return self._terminal_value(engine)

        state = engine.state
        player = state.player
        hp_ratio = player.hp / max(player.max_hp, 1)

        live_enemies = [e for e in state.enemies if e.hp > 0]
        total_enemy_hp = sum(e.hp for e in live_enemies)
        total_enemy_max = sum(max(1, e.max_hp) for e in state.enemies)

        if total_enemy_hp <= 0:
            return 0.85 + 0.15 * hp_ratio

        enemy_progress = 1.0 - (total_enemy_hp / max(total_enemy_max, 1))

        return max(0.0, min(1.0, 0.35 * hp_ratio + 0.30 * enemy_progress + 0.15))

    def export_last_root_stats(self, selected_action: Any = None) -> Optional[Dict[str, Any]]:
        """Return the most recent root summary as a JSON-friendly payload."""
        if self.last_search_summary is None:
            return None

        return {
            "sims": self.last_search_summary.num_simulations,
            "root_value": self.last_search_summary.root_value,
            "actions": [
                {
                    "id": str(action.action),
                    "visits": action.visits,
                    "pct": round(action.probability * 100, 1),
                    "q": round(action.q_value, 4),
                    "selected": action.action == selected_action,
                }
                for action in self.last_search_summary.actions
            ],
        }
