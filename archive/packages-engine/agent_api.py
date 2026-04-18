"""
Agent API - JSON-serializable action and observation interfaces for RL agents.

This module provides type definitions for the agent-facing API surface.
GameRunner implements the actual action/observation generation directly;
this module only exports the shared types and constants.

Key types:
- ActionDict: JSON-serializable action with id, type, label, params, phase
- ActionResult: Result of executing an action
- ObservationDict: Complete observable game state

Usage:
    runner = GameRunner(seed="TEST", ascension=20)

    # Get current observation
    obs = runner.get_observation()

    # Get available actions as dicts
    actions = runner.get_available_action_dicts()

    # Execute action dict
    result = runner.take_action_dict(actions[0])
"""

from __future__ import annotations

from typing import List, Dict, Any, Optional, TypedDict


# =============================================================================
# Type Definitions
# =============================================================================

class ActionDict(TypedDict, total=False):
    """JSON-serializable action dict."""
    id: str  # Stable identifier for the action
    type: str  # Action type enum string
    label: str  # Human-readable summary
    params: Dict[str, Any]  # Required parameters
    requires: List[str]  # Optional hints for missing params
    phase: str  # Current phase


class ActionResult(TypedDict, total=False):
    """Result of executing an action."""
    success: bool
    error: Optional[str]
    # Action-specific result fields
    data: Dict[str, Any]


class ObservationDict(TypedDict, total=False):
    """Complete observable game state.

    Profiles:
        "human" (default): Training-ready observation with all fields an RL
            agent needs.  draw_pile/discard_pile/exhaust_pile in combat are
            included as counts only (draw_pile_count, discard_pile_count,
            exhaust_pile_count).
        "debug": Superset of human.  Adds full pile contents, RNG counters,
            and other diagnostic fields under a top-level ``debug`` key.

    HP field naming convention:
        - ``run.current_hp`` / ``run.max_hp``: persistent run-level HP.
        - ``combat.player.hp`` / ``combat.player.max_hp``: combat-local
          player HP (mirrors CombatState.player.hp).
        - ``combat.enemies[].hp``: per-enemy combat HP.
        The RL encoder (``rl_observations.py``) reads ``run.current_hp``
        for the run scalar and ``combat.enemies[].hp`` for enemy features.
    """
    observation_schema_version: str
    action_schema_version: str
    profile: str
    phase: str
    run: Dict[str, Any]
    map: Dict[str, Any]
    combat: Optional[Dict[str, Any]]
    event: Optional[Dict[str, Any]]
    reward: Optional[Dict[str, Any]]
    shop: Optional[Dict[str, Any]]
    rest: Optional[Dict[str, Any]]
    treasure: Optional[Dict[str, Any]]
    debug: Optional[Dict[str, Any]]


# =============================================================================
# Phase Names (for observation)
# =============================================================================

PHASE_NAMES = {
    "NEOW": "neow",
    "MAP_NAVIGATION": "map",
    "COMBAT": "combat",
    "COMBAT_REWARDS": "reward",
    "EVENT": "event",
    "SHOP": "shop",
    "REST": "rest",
    "TREASURE": "treasure",
    "BOSS_REWARDS": "boss_reward",
    "RUN_COMPLETE": "run_complete",
}


# =============================================================================
# Action ID Generation
# =============================================================================

def generate_action_id(action_type: str, *args) -> str:
    """
    Generate a deterministic action ID from type and parameters.

    IDs are stable for identical state + phase.
    """
    parts = [action_type]
    for arg in args:
        if arg is not None and arg != -1:
            parts.append(str(arg))
    return "_".join(parts)
