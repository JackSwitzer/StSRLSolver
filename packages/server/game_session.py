"""
Game session management -- wraps a GameRunner instance.

Each WebSocket connection gets its own GameSession.  The session tracks
step count and action history for debugging and replay.
"""

from __future__ import annotations

import uuid
from typing import Any, Dict, List

from packages.engine.game import GameRunner


class GameSession:
    """Manages a single game session backed by a GameRunner."""

    def __init__(
        self,
        seed: str = "TEST",
        ascension: int = 20,
        character: str = "Watcher",
        session_id: str | None = None,
    ):
        self.session_id = session_id or uuid.uuid4().hex[:12]
        self.runner = GameRunner(
            seed=seed,
            ascension=ascension,
            character=character,
            verbose=False,
        )
        self.step_count: int = 0
        self.action_history: List[Dict[str, Any]] = []

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def get_observation(self, profile: str = "human") -> Dict[str, Any]:
        """Return the current JSON-serializable observation."""
        return self.runner.get_observation(profile=profile)

    def get_actions(self) -> List[Dict[str, Any]]:
        """Return available actions as JSON-serializable dicts."""
        return self.runner.get_available_action_dicts()

    def take_action(self, action_dict: Dict[str, Any]) -> Dict[str, Any]:
        """Execute an action and return the result dict."""
        result = self.runner.take_action_dict(action_dict)
        self.step_count += 1
        self.action_history.append(action_dict)
        return result

    @property
    def game_over(self) -> bool:
        return self.runner.game_over

    @property
    def game_won(self) -> bool:
        return self.runner.game_won
