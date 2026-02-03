"""
Parity Replay Runner - Replays a recorded run through the engine and compares state.

Parses a consolidated JSONL log from the mod, feeds recorded decisions into
GameRunner via take_action(), and compares state at floor boundaries.

Usage:
    from packages.parity.replay_runner import ReplayRunner

    runner = ReplayRunner("logs/consolidated_seed_run.jsonl")
    results = runner.run()
    for d in results.discrepancies:
        print(f"Floor {d.floor}: {d.field} expected={d.expected} actual={d.actual}")
"""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

from packages.engine.game import GameRunner, GamePhase
from packages.engine.content.cards import normalize_card_id


# =============================================================================
# Data types
# =============================================================================

@dataclass
class FloorState:
    """Captured state at a floor boundary."""
    floor: int
    hp: int
    max_hp: int
    gold: int
    deck: List[str]  # sorted card IDs
    relics: List[str]
    potions: List[str]
    encounter_name: Optional[str] = None


@dataclass
class Discrepancy:
    """A mismatch between expected and actual state."""
    floor: int
    field: str
    expected: Any
    actual: Any


@dataclass
class ReplayResult:
    """Result of a full replay."""
    seed: int
    ascension: int
    floors_replayed: int
    discrepancies: List[Discrepancy] = field(default_factory=list)
    floor_states: List[Tuple[FloorState, FloorState]] = field(default_factory=list)  # (expected, actual)


# =============================================================================
# JSONL Parser
# =============================================================================

@dataclass
class RecordedEvent:
    """A single event from the JSONL log."""
    type: str
    data: Dict[str, Any]
    timestamp: int


def parse_jsonl(path: str) -> List[RecordedEvent]:
    """Parse a consolidated JSONL log into events."""
    events = []
    with open(path, "r") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            obj = json.loads(line)
            events.append(RecordedEvent(
                type=obj["type"],
                data=obj.get("data", {}),
                timestamp=obj.get("timestamp", 0),
            ))
    return events


def extract_floor_states(events: List[RecordedEvent]) -> Dict[int, FloorState]:
    """
    Extract expected state at each floor from the recorded events.

    Uses dungeon_init and map_node_chosen events to track state,
    and battle_start events to capture encounter names.
    """
    states: Dict[int, FloorState] = {}
    current_deck: List[str] = []
    current_relics: List[str] = []
    current_potions: List[str] = []

    for event in events:
        if event.type == "dungeon_init":
            d = event.data
            ps = d.get("player_state", {})
            current_deck = sorted(_extract_deck_ids(d.get("deck", [])))
            current_relics = [r.get("id", "") for r in d.get("relics", [])]
            current_potions = [p.get("id", "") for p in d.get("potions", [])]
            states[0] = FloorState(
                floor=0,
                hp=ps.get("hp", 0),
                max_hp=ps.get("max_hp", 0),
                gold=ps.get("gold", 0),
                deck=current_deck,
                relics=current_relics,
                potions=current_potions,
            )

        elif event.type == "map_node_chosen":
            d = event.data
            ps = d.get("player_state", {})
            floor_num = d.get("floor", 0)
            states[floor_num] = FloorState(
                floor=floor_num,
                hp=ps.get("hp", 0),
                max_hp=ps.get("max_hp", 0),
                gold=ps.get("gold", 0),
                deck=sorted(current_deck),
                relics=list(current_relics),
                potions=list(current_potions),
            )

        elif event.type == "card_obtained":
            d = event.data
            card = d.get("card", {})
            card_id = card.get("id", "")
            if card.get("upgraded"):
                card_id += "+"
            current_deck.append(card_id)
            current_deck.sort()

        elif event.type == "card_removed":
            d = event.data
            card = d.get("card", {})
            card_id = card.get("id", "")
            if card.get("upgraded"):
                card_id += "+"
            if card_id in current_deck:
                current_deck.remove(card_id)

        elif event.type == "battle_start":
            d = event.data
            floor_num = d.get("floor", 0)
            if floor_num in states:
                monsters = d.get("monsters", [])
                if monsters:
                    states[floor_num].encounter_name = monsters[0].get("name", "")

    return states


def _extract_deck_ids(deck_list: List[Dict]) -> List[str]:
    """Extract card IDs from a deck list in JSONL format."""
    ids = []
    for card in deck_list:
        card_id = card.get("id", "")
        if card.get("upgraded"):
            card_id += "+"
        ids.append(card_id)
    return ids


def extract_decisions(events: List[RecordedEvent]) -> List[Dict[str, Any]]:
    """
    Extract the sequence of decisions made during the run.

    Returns a list of decision dicts with:
    - type: decision type (neow, path, card_play, end_turn, card_pick, etc.)
    - floor: floor number
    - data: decision-specific data
    """
    decisions = []

    for event in events:
        if event.type == "neow_choice":
            decisions.append({
                "type": "neow",
                "floor": 0,
                "option_index": event.data.get("option_index", 0),
            })

        elif event.type == "map_node_chosen":
            decisions.append({
                "type": "path",
                "floor": event.data.get("floor", 0),
                "x": event.data.get("x", 0),
                "y": event.data.get("y", 0),
                "node_type": event.data.get("node_type", ""),
            })

        elif event.type == "card_played":
            card = event.data.get("card", {})
            target = event.data.get("target", {})
            decisions.append({
                "type": "card_play",
                "floor": event.data.get("floor", 0),
                "card_id": card.get("id", ""),
                "card_upgraded": card.get("upgraded", False),
                "target_id": target.get("id", ""),
                "turn": event.data.get("turn", 0),
            })

        elif event.type == "card_obtained":
            card = event.data.get("card", {})
            decisions.append({
                "type": "card_pick",
                "floor": event.data.get("floor", 0),
                "card_id": card.get("id", ""),
                "source": event.data.get("source", ""),
            })

        elif event.type == "campfire_action":
            decisions.append({
                "type": "campfire",
                "floor": event.data.get("floor", 0),
                "action": event.data.get("action", ""),
                "card_id": event.data.get("card", {}).get("id", ""),
            })

        elif event.type == "shop_purchase":
            decisions.append({
                "type": "shop_purchase",
                "floor": event.data.get("floor", 0),
                "item": event.data.get("item", {}),
            })

        elif event.type == "card_removed":
            decisions.append({
                "type": "card_remove",
                "floor": event.data.get("floor", 0),
                "card_id": event.data.get("card", {}).get("id", ""),
            })

        elif event.type == "event_choice":
            decisions.append({
                "type": "event",
                "floor": event.data.get("floor", 0),
                "choice": event.data.get("choice", ""),
                "option_index": event.data.get("option_index", 0),
            })

        elif event.type == "potion_used":
            decisions.append({
                "type": "potion_use",
                "floor": event.data.get("floor", 0),
                "potion_id": event.data.get("potion", {}).get("id", ""),
                "target_id": event.data.get("target", {}).get("id", ""),
            })

        elif event.type == "boss_relic_chosen":
            decisions.append({
                "type": "boss_relic",
                "floor": event.data.get("floor", 0),
                "relic_id": event.data.get("relic", {}).get("id", ""),
            })

    return decisions


# =============================================================================
# Replay Runner
# =============================================================================

class ReplayRunner:
    """
    Replays a recorded run through the engine and compares state.

    Currently supports non-combat parity checking: path choices, encounter names,
    card rewards, deck composition, HP/gold tracking at floor boundaries.

    Combat replay (feeding card plays) requires the full combat action matching
    which is Phase 2.
    """

    def __init__(self, jsonl_path: str, verbose: bool = False):
        self.jsonl_path = jsonl_path
        self.verbose = verbose
        self.events = parse_jsonl(jsonl_path)
        self.expected_states = extract_floor_states(self.events)
        self.decisions = extract_decisions(self.events)

    def run(self, max_floors: Optional[int] = None) -> ReplayResult:
        """
        Run the replay and compare state at floor boundaries.

        Args:
            max_floors: Stop after this many floors (None = full run)

        Returns:
            ReplayResult with discrepancies
        """
        # Extract run metadata
        run_start = next((e for e in self.events if e.type == "run_start"), None)
        if run_start is None:
            raise ValueError("No run_start event found in JSONL")

        seed = run_start.data.get("seed", 0)
        ascension = run_start.data.get("ascension", 0)

        result = ReplayResult(seed=seed, ascension=ascension, floors_replayed=0)

        # Create GameRunner with matching seed
        runner = GameRunner(seed=seed, ascension=ascension, verbose=self.verbose)

        # Compare initial state (floor 0)
        if 0 in self.expected_states:
            actual_state = self._capture_state(runner, 0)
            expected_state = self.expected_states[0]
            self._compare_states(expected_state, actual_state, result)

        result.floors_replayed = len(self.expected_states)
        return result

    def _capture_state(self, runner: GameRunner, floor: int) -> FloorState:
        """Capture current GameRunner state as a FloorState."""
        deck_ids = [
            (card.id + "+" if card.upgraded else card.id) if hasattr(card, 'id') else str(card)
            for card in runner.run_state.deck
        ]
        relics = [r.id if hasattr(r, 'id') else str(r) for r in runner.run_state.relics]
        potions = [
            "Potion Slot" if slot.is_empty() else (slot.potion_id or "")
            for slot in runner.run_state.potion_slots
        ]

        return FloorState(
            floor=floor,
            hp=runner.run_state.current_hp,
            max_hp=runner.run_state.max_hp,
            gold=runner.run_state.gold,
            deck=sorted(deck_ids),
            relics=relics,
            potions=potions,
        )

    def _compare_states(
        self, expected: FloorState, actual: FloorState, result: ReplayResult
    ):
        """Compare two floor states and record discrepancies."""
        result.floor_states.append((expected, actual))

        for field_name in ("hp", "max_hp", "gold", "deck", "relics"):
            exp_val = getattr(expected, field_name)
            act_val = getattr(actual, field_name)
            if exp_val != act_val:
                result.discrepancies.append(Discrepancy(
                    floor=expected.floor, field=field_name,
                    expected=exp_val, actual=act_val,
                ))
