"""
Protocol definitions for Java ↔ Python combat search communication.

Message Types:
- search_request: Java sends combat state + search params
- search_response: Python returns best line + alternatives
- verify_state: Post-action state verification
- verify_response: Verification result
"""

from __future__ import annotations

import json
import uuid
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, Union


# =============================================================================
# Request Types
# =============================================================================


@dataclass
class SearchRequest:
    """Search request from Java mod."""

    request_id: str
    seed: Dict[str, Any]  # {seed_long, seed_string}
    rng_state: Dict[str, int]  # RNG counters
    player: Dict[str, Any]  # Player state
    card_piles: Dict[str, List]  # hand, draw_pile, discard_pile, exhaust_pile
    enemies: List[Dict[str, Any]]  # Enemy states
    potions: List[Dict[str, Any]]  # Potion slots
    relics: List[Dict[str, Any]]  # Relic list
    search_params: Dict[str, Any] = field(default_factory=dict)

    @classmethod
    def from_json(cls, data: Dict[str, Any]) -> "SearchRequest":
        """Parse from JSON dict."""
        return cls(
            request_id=data.get("request_id", str(uuid.uuid4())),
            seed=data.get("seed", {}),
            rng_state=data.get("rng_state", {}),
            player=data.get("player", {}),
            card_piles=data.get("card_piles", {}),
            enemies=data.get("enemies", []),
            potions=data.get("potions", []),
            relics=data.get("relics", []),
            search_params=data.get("search_params", {}),
        )


@dataclass
class VerifyStateRequest:
    """State verification request from Java mod."""

    request_id: str
    action_taken: Dict[str, Any]  # The action that was executed
    predicted_state: Dict[str, Any]  # What Python predicted
    actual_state: Dict[str, Any]  # What Java observed

    @classmethod
    def from_json(cls, data: Dict[str, Any]) -> "VerifyStateRequest":
        """Parse from JSON dict."""
        return cls(
            request_id=data.get("request_id", str(uuid.uuid4())),
            action_taken=data.get("action_taken", {}),
            predicted_state=data.get("predicted_state", {}),
            actual_state=data.get("actual_state", {}),
        )


# =============================================================================
# Response Types
# =============================================================================


@dataclass
class ActionInfo:
    """Information about a single action."""

    action_type: str  # "card", "potion", "end_turn"
    card_id: Optional[str] = None
    card_idx: Optional[int] = None
    potion_id: Optional[str] = None
    potion_slot: Optional[int] = None
    target_idx: int = -1
    target_name: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to JSON-serializable dict."""
        result = {"type": self.action_type}
        if self.card_id:
            result["card_id"] = self.card_id
        if self.card_idx is not None:
            result["card_idx"] = self.card_idx
        if self.potion_id:
            result["potion_id"] = self.potion_id
        if self.potion_slot is not None:
            result["slot"] = self.potion_slot
        if self.target_idx >= 0:
            result["target"] = self.target_idx
        if self.target_name:
            result["target_name"] = self.target_name
        return result


@dataclass
class ExpectedOutcome:
    """Expected outcome of an action sequence."""

    hp_lost: int = 0
    damage_dealt: int = 0
    enemy_killed: bool = False
    player_dead: bool = False
    block_remaining: int = 0
    energy_remaining: int = 0

    def to_dict(self) -> Dict[str, Any]:
        """Convert to JSON-serializable dict."""
        return {
            "hp_lost": self.hp_lost,
            "damage_dealt": self.damage_dealt,
            "enemy_killed": self.enemy_killed,
            "player_dead": self.player_dead,
            "block_remaining": self.block_remaining,
            "energy_remaining": self.energy_remaining,
        }


@dataclass
class BestLine:
    """A recommended action sequence."""

    actions: List[ActionInfo]
    expected_outcome: ExpectedOutcome
    display_text: str  # Human-readable summary
    score: float = 0.0  # Search score
    visits: int = 0  # MCTS visit count

    def to_dict(self) -> Dict[str, Any]:
        """Convert to JSON-serializable dict."""
        return {
            "actions": [a.to_dict() for a in self.actions],
            "expected_outcome": self.expected_outcome.to_dict(),
            "display_text": self.display_text,
            "score": self.score,
            "visits": self.visits,
        }


@dataclass
class SearchResponse:
    """Search response to Java mod."""

    request_id: str
    best_line: Optional[BestLine] = None
    alternative_lines: List[BestLine] = field(default_factory=list)
    search_time_ms: float = 0.0
    nodes_explored: int = 0
    error: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to JSON-serializable dict."""
        result = {
            "type": "search_response",
            "request_id": self.request_id,
            "search_time_ms": self.search_time_ms,
            "nodes_explored": self.nodes_explored,
        }
        if self.best_line:
            result["best_line"] = self.best_line.to_dict()
        if self.alternative_lines:
            result["alternative_lines"] = [a.to_dict() for a in self.alternative_lines]
        if self.error:
            result["error"] = self.error
        return result

    def to_json(self) -> str:
        """Convert to JSON string."""
        return json.dumps(self.to_dict())


@dataclass
class VerifyResponse:
    """Verification response."""

    request_id: str
    matches: bool
    mismatches: List[Dict[str, Any]] = field(default_factory=list)
    diagnosis: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert to JSON-serializable dict."""
        return {
            "type": "verify_response",
            "request_id": self.request_id,
            "matches": self.matches,
            "mismatches": self.mismatches,
            "diagnosis": self.diagnosis,
        }

    def to_json(self) -> str:
        """Convert to JSON string."""
        return json.dumps(self.to_dict())


# =============================================================================
# Message Parsing
# =============================================================================


def parse_message(data: Union[str, bytes, Dict]) -> Union[SearchRequest, VerifyStateRequest, None]:
    """
    Parse an incoming message from Java.

    Args:
        data: JSON string, bytes, or dict

    Returns:
        Parsed request object or None if invalid
    """
    if isinstance(data, bytes):
        data = data.decode("utf-8")

    if isinstance(data, str):
        try:
            data = json.loads(data)
        except json.JSONDecodeError:
            return None

    msg_type = data.get("type", "")

    if msg_type == "search_request":
        return SearchRequest.from_json(data)
    elif msg_type == "verify_state":
        return VerifyStateRequest.from_json(data)
    else:
        # Try to infer from fields
        if "player" in data and "enemies" in data:
            return SearchRequest.from_json(data)

    return None


def create_response(
    request_id: str,
    best_action=None,
    action_scores: Dict[str, float] = None,
    search_time_ms: float = 0.0,
    nodes_explored: int = 0,
    combat_state=None,
    hand: List[str] = None,
    enemies: List = None,
    error: str = None,
) -> SearchResponse:
    """
    Create a search response from MCTS results.

    Args:
        request_id: Original request ID
        best_action: Best action from MCTS
        action_scores: Score for each action
        search_time_ms: Search duration
        nodes_explored: Nodes explored
        combat_state: Current combat state (for context)
        hand: Current hand (card IDs)
        enemies: Current enemies (for target names)
        error: Error message if search failed

    Returns:
        SearchResponse object
    """
    response = SearchResponse(
        request_id=request_id,
        search_time_ms=search_time_ms,
        nodes_explored=nodes_explored,
        error=error,
    )

    if error or best_action is None:
        return response

    # Convert best action to ActionInfo
    from ..state.combat import PlayCard, UsePotion, EndTurn

    actions = []
    if isinstance(best_action, PlayCard):
        card_id = hand[best_action.card_idx] if hand and best_action.card_idx < len(hand) else "Unknown"
        target_name = None
        if best_action.target_idx >= 0 and enemies and best_action.target_idx < len(enemies):
            target_name = enemies[best_action.target_idx].id
        actions.append(ActionInfo(
            action_type="card",
            card_id=card_id,
            card_idx=best_action.card_idx,
            target_idx=best_action.target_idx,
            target_name=target_name,
        ))
    elif isinstance(best_action, UsePotion):
        target_name = None
        if best_action.target_idx >= 0 and enemies and best_action.target_idx < len(enemies):
            target_name = enemies[best_action.target_idx].id
        actions.append(ActionInfo(
            action_type="potion",
            potion_slot=best_action.potion_idx,
            target_idx=best_action.target_idx,
            target_name=target_name,
        ))
    elif isinstance(best_action, EndTurn):
        actions.append(ActionInfo(action_type="end_turn"))

    # Create display text
    if actions:
        action = actions[0]
        if action.action_type == "card":
            display = f"Play {action.card_id}"
            if action.target_name:
                display += f" -> {action.target_name}"
        elif action.action_type == "potion":
            display = f"Use potion slot {action.potion_slot}"
            if action.target_name:
                display += f" -> {action.target_name}"
        else:
            display = "End Turn"
    else:
        display = "No action"

    # Get score for best action
    score = 0.0
    if action_scores:
        action_key = repr(best_action)
        score = action_scores.get(action_key, 0.0)

    response.best_line = BestLine(
        actions=actions,
        expected_outcome=ExpectedOutcome(),  # Will be filled by simulation
        display_text=display,
        score=score,
    )

    # Add alternative lines from action_scores
    if action_scores:
        sorted_actions = sorted(
            action_scores.items(),
            key=lambda x: x[1],
            reverse=True,
        )
        # Skip the best one, take top 3 alternatives
        for action_repr, alt_score in sorted_actions[1:4]:
            alt_line = BestLine(
                actions=[],  # Would need to reconstruct actions from repr
                expected_outcome=ExpectedOutcome(),
                display_text=f"Alternative: {action_repr}",
                score=alt_score,
            )
            response.alternative_lines.append(alt_line)

    return response
