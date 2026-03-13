"""Stochastic Population-Based Training (PBT) parameter optimizer.

Maintains a population of parameter vectors and evolves them based on
performance feedback. Bottom 20% of the population copies from top 20%
with random perturbation after each evaluation cycle.

Usage:
    optimizer = StochasticParamOptimizer(
        param_names=["damage_penalty", "elite_reward", "entropy_coeff"],
        initial_values={"damage_penalty": -0.03, "elite_reward": 0.50, "entropy_coeff": 0.05},
        bounds={"damage_penalty": (-0.10, 0.0), "elite_reward": (0.1, 1.5), "entropy_coeff": (0.01, 0.15)},
        population_size=8,
    )
    params = optimizer.suggest()
    # ... run training with params, measure avg_floor ...
    optimizer.report(params, score=avg_floor)
"""

from __future__ import annotations

import json
import logging
import time
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

import numpy as np

logger = logging.getLogger(__name__)


class StochasticParamOptimizer:
    """PBT-style optimizer for reward/training hyperparameters.

    Each member of the population is a dict of param_name -> value.
    After enough reports, truncation selection replaces the bottom 20%
    with perturbed copies of the top 20%.
    """

    def __init__(
        self,
        param_names: List[str],
        initial_values: Dict[str, float],
        bounds: Dict[str, Tuple[float, float]],
        population_size: int = 8,
        perturbation_scale: float = 0.1,
        truncation_fraction: float = 0.2,
        log_path: Optional[Path] = None,
    ):
        self.param_names = list(param_names)
        self.bounds = {k: bounds[k] for k in self.param_names}
        self.population_size = population_size
        self.perturbation_scale = perturbation_scale
        self.truncation_fraction = truncation_fraction
        self.log_path = log_path

        # Initialize population: member 0 is the initial values, rest are perturbed
        self._population: List[Dict[str, float]] = []
        base = {k: initial_values[k] for k in self.param_names}
        self._population.append(dict(base))
        for _ in range(population_size - 1):
            self._population.append(self._perturb(base))

        # Scores: one per population member, None if not yet evaluated
        self._scores: List[Optional[float]] = [None] * population_size

        # Round-robin index for suggest()
        self._next_idx = 0

        # History of all reported results
        self._history: List[Dict[str, Any]] = []

        # Best ever seen
        self._best_score: float = float("-inf")
        self._best_params: Dict[str, float] = dict(base)

        # Generation counter (increments after each full truncation cycle)
        self._generation = 0

    def suggest(self) -> Dict[str, float]:
        """Return the next parameter set to evaluate (round-robin through population)."""
        params = dict(self._population[self._next_idx])
        params["__population_idx"] = self._next_idx
        self._next_idx = (self._next_idx + 1) % self.population_size
        return params

    def report(self, params: Dict[str, float], score: float) -> None:
        """Report the performance of a parameter set.

        When all members have been evaluated, runs truncation selection.
        """
        idx = int(params.get("__population_idx", -1))
        if idx < 0 or idx >= self.population_size:
            logger.warning("report() called with unknown population index %s", idx)
            return

        self._scores[idx] = score

        # Update best ever
        if score > self._best_score:
            self._best_score = score
            self._best_params = {k: params[k] for k in self.param_names}

        # Log to history
        entry = {
            "timestamp": time.time(),
            "generation": self._generation,
            "population_idx": idx,
            "score": score,
            "params": {k: params[k] for k in self.param_names},
        }
        self._history.append(entry)
        if self.log_path:
            try:
                with open(self.log_path, "a") as f:
                    f.write(json.dumps(entry) + "\n")
            except OSError:
                pass

        # Check if all members have been evaluated
        if all(s is not None for s in self._scores):
            self._truncation_step()

    def get_best(self) -> Dict[str, float]:
        """Return the best parameter set found so far."""
        return dict(self._best_params)

    def save(self, path: Path) -> None:
        """Save optimizer state to JSON file."""
        state = {
            "param_names": self.param_names,
            "bounds": self.bounds,
            "population_size": self.population_size,
            "perturbation_scale": self.perturbation_scale,
            "truncation_fraction": self.truncation_fraction,
            "population": self._population,
            "scores": self._scores,
            "next_idx": self._next_idx,
            "best_score": self._best_score,
            "best_params": self._best_params,
            "generation": self._generation,
        }
        path.parent.mkdir(parents=True, exist_ok=True)
        with open(path, "w") as f:
            json.dump(state, f, indent=2)
        logger.info("Saved optimizer state to %s (gen %d)", path, self._generation)

    @classmethod
    def load(cls, path: Path) -> "StochasticParamOptimizer":
        """Load optimizer state from JSON file."""
        with open(path) as f:
            state = json.load(f)

        opt = cls(
            param_names=state["param_names"],
            initial_values=state["best_params"],
            bounds={k: tuple(v) for k, v in state["bounds"].items()},
            population_size=state["population_size"],
            perturbation_scale=state["perturbation_scale"],
            truncation_fraction=state["truncation_fraction"],
        )
        opt._population = state["population"]
        opt._scores = state["scores"]
        opt._next_idx = state["next_idx"]
        opt._best_score = state["best_score"]
        opt._best_params = state["best_params"]
        opt._generation = state["generation"]
        logger.info("Loaded optimizer state from %s (gen %d)", path, opt._generation)
        return opt

    # ------------------------------------------------------------------
    # Internal
    # ------------------------------------------------------------------

    def _perturb(self, params: Dict[str, float]) -> Dict[str, float]:
        """Create a perturbed copy of a parameter set.

        Multiplicative perturbation: new = old * exp(N(0, scale)).
        For params near zero (|val| < 1e-6), uses additive: new = old + N(0, scale * bound_range).
        Result is clipped to bounds.
        """
        result = {}
        for k in self.param_names:
            lo, hi = self.bounds[k]
            val = params[k]
            bound_range = hi - lo

            if abs(val) < 1e-6:
                # Additive perturbation for near-zero params
                noise = np.random.normal(0, self.perturbation_scale * bound_range)
                new_val = val + noise
            else:
                # Multiplicative perturbation
                factor = np.exp(np.random.normal(0, self.perturbation_scale))
                new_val = val * factor

            result[k] = float(np.clip(new_val, lo, hi))
        return result

    def _truncation_step(self) -> None:
        """Replace bottom performers with perturbed copies of top performers."""
        n = self.population_size
        n_replace = max(1, int(n * self.truncation_fraction))

        # Rank by score (ascending — worst first)
        ranked = sorted(range(n), key=lambda i: self._scores[i])
        bottom_indices = ranked[:n_replace]
        top_indices = ranked[-n_replace:]

        replacements = []
        for bot_idx in bottom_indices:
            # Pick a random top performer to copy from
            src_idx = top_indices[int(np.random.randint(len(top_indices)))]
            old_score = self._scores[bot_idx]
            new_params = self._perturb(self._population[src_idx])
            self._population[bot_idx] = new_params
            replacements.append(
                f"  [{bot_idx}] (score={old_score:.2f}) <- [{src_idx}] "
                f"(score={self._scores[src_idx]:.2f}) + perturbation"
            )

        self._generation += 1

        # Reset all scores for next round
        self._scores = [None] * n

        logger.info(
            "PBT generation %d complete. Replacements:\n%s",
            self._generation,
            "\n".join(replacements),
        )
