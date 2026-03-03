"""
WebSocket server for streaming Slay the Spire game state to visualization clients.

Usage:
    uv run python -m packages.server --port 8080
"""

__version__ = "0.1.0"

from .ws_server import GameServer
from .game_session import GameSession
from .protocol import (
    MessageType,
    make_game_created,
    make_actions,
    make_action_result,
    make_observation,
    make_step,
    make_game_over,
    make_error,
)
