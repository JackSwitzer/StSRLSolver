"""Seed pool for training: manages seed rotation with difficulty-aware sampling."""

from __future__ import annotations

from typing import Dict, List, Optional

# High-value seeds from Merl's A20 Watcher runs -- prioritized for training
MERL_SEEDS: List[str] = [
    "1LM7V5N5RBS9T", "44A3PNRBRGEZ", "47WBD2053A2I", "2LS2I1KBMN52D",
    "4V6WMX8507Y02", "1AAG54G24TRIN", "2QMLIWRVYYM9", "28PB07ZASVLP3",
    "2ATIY7STKY9BN", "QIPQIKIZ3YMG", "3AYDIYKAR3IP2", "4R9V1NTNCR0W7",
]

# Max plays for Merl seeds (more replays -- these are proven winnable)
_MERL_MAX_PLAYS = 10


class SeedPool:
    """Manages seed rotation with difficulty-aware sampling.

    Merl seeds (known-winnable A20 Watcher seeds) are prioritized with
    higher max_plays (10 vs 3 for normal seeds) so the model gets more
    exposure to seeds that are known to be solvable.
    """

    def __init__(self, initial_seeds: Optional[List[str]] = None, max_plays: int = 3):
        self.max_plays = max_plays
        self.play_counts: Dict[str, int] = {}
        self._max_plays_per_seed: Dict[str, int] = {}
        self.results: Dict[str, List[Dict]] = {}  # seed -> list of game results
        self._next_idx = 0

        # Always add Merl seeds first (prioritized)
        for s in MERL_SEEDS:
            self.play_counts[s] = 0
            self._max_plays_per_seed[s] = _MERL_MAX_PLAYS

        if initial_seeds:
            for s in initial_seeds:
                if s not in self.play_counts:
                    self.play_counts[s] = 0
                    self._max_plays_per_seed[s] = max_plays
        else:
            # Generate default seeds
            for i in range(200):
                s = f"Train_{i}"
                if s not in self.play_counts:
                    self.play_counts[s] = 0
                    self._max_plays_per_seed[s] = max_plays

    def _seed_max_plays(self, seed: str) -> int:
        """Return max plays for a seed (Merl seeds get more)."""
        return self._max_plays_per_seed.get(seed, self.max_plays)

    def get_seed(self) -> str:
        """Get next seed to play. Merl seeds interleaved first, then round-robin."""
        # Prioritize Merl seeds: pick the one with fewest plays (round-robin)
        merl_available = [
            s for s in MERL_SEEDS
            if self.play_counts.get(s, 0) < self._seed_max_plays(s)
        ]
        if merl_available:
            # Pick the Merl seed with fewest plays to spread evenly
            seed = min(merl_available, key=lambda s: self.play_counts.get(s, 0))
            self.play_counts[seed] += 1
            return seed

        # Fall back to normal round-robin
        available = [
            s for s, c in self.play_counts.items()
            if c < self._seed_max_plays(s)
        ]
        if not available:
            # All seeds exhausted, add more
            base = len(self.play_counts)
            for i in range(100):
                s = f"Train_{base + i}"
                self.play_counts[s] = 0
                self._max_plays_per_seed[s] = self.max_plays
            available = [
                s for s, c in self.play_counts.items()
                if c < self._seed_max_plays(s)
            ]

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
