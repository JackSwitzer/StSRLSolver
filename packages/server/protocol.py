"""
Message protocol definitions for the WebSocket game server.

Client -> Server message types:
    new_game      - Start a new game session
    get_actions   - Request available actions
    take_action   - Execute an action
    get_observation - Request current game observation
    auto_play     - Auto-play N steps with delay

Server -> Client message types:
    game_created  - New game started, includes initial observation
    actions       - Available actions list
    action_result - Result of executing an action
    observation   - Current game observation
    step          - Single auto-play step update
    game_over     - Terminal game state
    error         - Error response
"""

from __future__ import annotations

from enum import Enum
from typing import Any, Dict, List, Optional


class MessageType(str, Enum):
    """All valid message types in the protocol."""

    # Client -> Server
    NEW_GAME = "new_game"
    GET_ACTIONS = "get_actions"
    TAKE_ACTION = "take_action"
    GET_OBSERVATION = "get_observation"
    AUTO_PLAY = "auto_play"

    # Server -> Client
    GAME_CREATED = "game_created"
    ACTIONS = "actions"
    ACTION_RESULT = "action_result"
    OBSERVATION = "observation"
    STEP = "step"
    GAME_OVER = "game_over"
    ERROR = "error"


# ---------------------------------------------------------------------------
# Server -> Client message constructors
# ---------------------------------------------------------------------------


def make_game_created(session_id: str, observation: Dict[str, Any]) -> Dict[str, Any]:
    """Build a game_created response."""
    return {
        "type": MessageType.GAME_CREATED.value,
        "session_id": session_id,
        "observation": observation,
    }


def make_actions(actions: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Build an actions response."""
    return {
        "type": MessageType.ACTIONS.value,
        "actions": actions,
    }


def make_action_result(
    result: Dict[str, Any],
    observation: Dict[str, Any],
    game_over: bool = False,
    game_won: bool = False,
) -> Dict[str, Any]:
    """Build an action_result response."""
    msg: Dict[str, Any] = {
        "type": MessageType.ACTION_RESULT.value,
        "result": result,
        "observation": observation,
    }
    if game_over:
        msg["game_over"] = True
        msg["won"] = game_won
    return msg


def make_observation(observation: Dict[str, Any]) -> Dict[str, Any]:
    """Build an observation response."""
    return {
        "type": MessageType.OBSERVATION.value,
        "observation": observation,
    }


def make_step(
    step: int,
    observation: Dict[str, Any],
    action_taken: Dict[str, Any],
    game_over: bool = False,
    game_won: bool = False,
) -> Dict[str, Any]:
    """Build a step response (sent during auto_play)."""
    msg: Dict[str, Any] = {
        "type": MessageType.STEP.value,
        "step": step,
        "observation": observation,
        "action_taken": action_taken,
    }
    if game_over:
        msg["game_over"] = True
        msg["won"] = game_won
    return msg


def make_game_over(won: bool, observation: Dict[str, Any]) -> Dict[str, Any]:
    """Build a game_over response."""
    return {
        "type": MessageType.GAME_OVER.value,
        "won": won,
        "observation": observation,
    }


def make_error(message: str, request_type: Optional[str] = None) -> Dict[str, Any]:
    """Build an error response."""
    msg: Dict[str, Any] = {
        "type": MessageType.ERROR.value,
        "error": message,
    }
    if request_type is not None:
        msg["request_type"] = request_type
    return msg
