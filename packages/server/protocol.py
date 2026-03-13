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
    CONQUERER_RUN = "conquerer_run"
    TRAINING_START = "training_start"
    TRAINING_STOP = "training_stop"
    TRAINING_RESUME = "training_resume"
    TRAINING_FOCUS = "training_focus"
    COMMAND = "command"

    # Server -> Client
    GAME_CREATED = "game_created"
    ACTIONS = "actions"
    ACTION_RESULT = "action_result"
    OBSERVATION = "observation"
    STEP = "step"
    GAME_OVER = "game_over"
    ERROR = "error"
    CONQUERER_PATH_RESULT = "conquerer_path_result"
    CONQUERER_COMPLETE = "conquerer_complete"
    GRID_UPDATE = "grid_update"
    AGENT_STEP = "agent_step"
    MCTS_RESULT = "mcts_result"
    PLANNER_RESULT = "planner_result"
    AGENT_EPISODE = "agent_episode"
    TRAINING_STATS = "training_stats"
    SYSTEM_STATS = "system_stats"
    METRICS_HISTORY = "metrics_history"


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


def make_conquerer_path_result(path_result: Dict[str, Any], active_paths: int) -> Dict[str, Any]:
    """Build a conquerer_path_result response (sent as each path completes)."""
    return {
        "type": MessageType.CONQUERER_PATH_RESULT.value,
        "path": path_result,
        "active_paths": active_paths,
    }


def make_conquerer_complete(result: Dict[str, Any]) -> Dict[str, Any]:
    """Build a conquerer_complete response with full ConquererResult."""
    return {
        "type": MessageType.CONQUERER_COMPLETE.value,
        **result,
    }


def make_mcts_result(
    agent_id: int,
    sims: int,
    elapsed_ms: float,
    root_value: float,
    actions: List[Dict[str, Any]],
    policy_version: Optional[int] = None,
) -> Dict[str, Any]:
    """Build an MCTS root-summary payload for the dashboard."""
    msg: Dict[str, Any] = {
        "type": MessageType.MCTS_RESULT.value,
        "agent_id": agent_id,
        "sims": sims,
        "elapsed_ms": round(elapsed_ms, 1),
        "root_value": root_value,
        "actions": actions,
    }
    if policy_version is not None:
        msg["policy_version"] = policy_version
    return msg


def make_planner_result(
    agent_id: int,
    lines_considered: int,
    strategy: str,
    turns_to_kill: int,
    expected_hp_loss: float,
    confidence: float,
    cards_played: List[str],
    elapsed_ms: Optional[float] = None,
    policy_version: Optional[int] = None,
) -> Dict[str, Any]:
    """Build a planner_result payload used by the combat detail view."""
    msg: Dict[str, Any] = {
        "type": MessageType.PLANNER_RESULT.value,
        "agent_id": agent_id,
        "lines_considered": lines_considered,
        "strategy": strategy,
        "turns_to_kill": turns_to_kill,
        "expected_hp_loss": expected_hp_loss,
        "confidence": confidence,
        "cards_played": cards_played,
    }
    if elapsed_ms is not None:
        msg["elapsed_ms"] = round(elapsed_ms, 1)
    if policy_version is not None:
        msg["policy_version"] = policy_version
    return msg
