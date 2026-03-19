"""Seed pool for training: manages seed rotation with difficulty-aware sampling."""

from __future__ import annotations

from typing import Dict, List, Optional


# Merl seeds: curated seeds known to produce interesting/challenging runs
MERL_SEEDS: List[str] = [
    "MERL_001", "MERL_002", "MERL_003", "MERL_004",
    "MERL_005", "MERL_006", "MERL_007", "MERL_008",
    "MERL_009", "MERL_010", "MERL_011", "MERL_012",
]


class SeedPool:
    """Manages seed rotation with difficulty-aware sampling."""

    def __init__(self, initial_seeds: Optional[List[str]] = None, max_plays: int = 3):
        self.max_plays = max_plays
        self.play_counts: Dict[str, int] = {}
        self.results: Dict[str, List[Dict]] = {}  # seed -> list of game results
        self._next_idx = 0

        # Prioritize Merl seeds first
        for s in MERL_SEEDS:
            self.play_counts[s] = 0

        if initial_seeds:
            for s in initial_seeds:
                self.play_counts[s] = 0
        else:
            # Generate default seeds
            for i in range(200):
                s = f"Train_{i}"
                self.play_counts[s] = 0

    def get_seed(self) -> str:
        """Get next seed to play. Prioritizes Merl seeds (max_plays=10), then round-robin."""
        # Prioritize Merl seeds with higher max_plays
        merl_available = [s for s in MERL_SEEDS if self.play_counts.get(s, 0) < 10]
        if merl_available:
            seed = merl_available[self._next_idx % len(merl_available)]
            self._next_idx += 1
            self.play_counts[seed] = self.play_counts.get(seed, 0) + 1
            return seed

        available = [s for s, c in self.play_counts.items() if c < self.max_plays]
        if not available:
            # All seeds exhausted, add more
            base = len(self.play_counts)
            for i in range(100):
                s = f"Train_{base + i}"
                self.play_counts[s] = 0
            available = [s for s, c in self.play_counts.items() if c < self.max_plays]

        seed = available[self._next_idx % len(available)]
        self._next_idx += 1
        self.play_counts[seed] += 1
        return seed

    def record_result(self, seed: str, result: Dict) -> None:
        if seed not in self.results:
            self.results[seed] = []
        self.results[seed].append(result)

    @property
    def total_games(self) -> int:
        return sum(self.play_counts.values())

    @property
    def unique_seeds(self) -> int:
        return len([s for s, c in self.play_counts.items() if c > 0])
