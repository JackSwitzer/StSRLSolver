"""Seed pool for training: manages seed rotation with difficulty-aware sampling."""

from __future__ import annotations

from typing import Dict, List, Optional


class SeedPool:
    """Manages seed rotation with difficulty-aware sampling."""

    def __init__(self, initial_seeds: Optional[List[str]] = None, max_plays: int = 3):
        self.max_plays = max_plays
        self.play_counts: Dict[str, int] = {}
        self.results: Dict[str, List[Dict]] = {}  # seed -> list of game results
        self._next_idx = 0

        if initial_seeds:
            for s in initial_seeds:
                self.play_counts[s] = 0
        else:
            # Generate default seeds
            for i in range(200):
                s = f"Train_{i}"
                self.play_counts[s] = 0

    def get_seed(self) -> str:
        """Get next seed to play (round-robin with max plays)."""
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
