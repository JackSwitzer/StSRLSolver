"""
VOD-specific state management wrapping core/state/run.py.
"""

from dataclasses import dataclass, field
from typing import Optional
from datetime import datetime
from enum import Enum

from core.state import RunState, create_watcher_run


class DecisionType(str, Enum):
    """Types of decisions that can be extracted from a VOD."""
    SEED = "seed"
    NEOW = "neow"
    PATH = "path"
    COMBAT_START = "combat_start"
    COMBAT_TURN = "combat_turn"
    COMBAT_END = "combat_end"
    CARD_REWARD = "card_reward"
    RELIC_REWARD = "relic_reward"
    POTION_REWARD = "potion_reward"
    SHOP = "shop"
    REST = "rest"
    BOSS_RELIC = "boss_relic"
    EVENT = "event"
    RESULT = "result"


@dataclass
class MapPosition:
    """Position on the map within an act."""
    act: int = 1
    x: int = -1  # Column (-1 = not on map yet)
    y: int = -1  # Row (floor within act)

    def is_at_start(self) -> bool:
        return self.x == -1 and self.y == -1


@dataclass
class CombatExtraction:
    """Tracks combat state during extraction."""
    floor: int
    enemy: str
    turn: int = 1
    hp_at_start: int = 0
    max_hp: int = 0
    cards_played: list[list[str]] = field(default_factory=list)  # Per-turn
    timestamp_start: Optional[str] = None
    timestamp_end: Optional[str] = None

    def add_turn(self, cards: list[str]) -> None:
        """Record cards played this turn."""
        self.cards_played.append(cards)
        self.turn += 1

    def to_dict(self) -> dict:
        return {
            "floor": self.floor,
            "enemy": self.enemy,
            "turn_count": self.turn - 1,
            "hp_at_start": self.hp_at_start,
            "max_hp": self.max_hp,
            "cards_by_turn": self.cards_played,
            "timestamp_start": self.timestamp_start,
            "timestamp_end": self.timestamp_end,
        }


@dataclass
class DecisionLog:
    """A single decision extracted from the VOD."""
    decision_type: DecisionType
    floor: int
    timestamp: Optional[str]
    data: dict
    confidence: float = 1.0
    pass_number: int = 1

    def unique_key(self) -> str:
        """Generate a unique key for voting comparison."""
        # Floor + type + sorted options hash for decisions with options
        base = f"{self.floor}:{self.decision_type.value}"
        if "options" in self.data:
            opts = sorted(self.data["options"]) if isinstance(self.data["options"], list) else [str(self.data["options"])]
            base += f":{hash(tuple(opts))}"
        return base

    def to_dict(self) -> dict:
        return {
            "type": self.decision_type.value,
            "floor": self.floor,
            "timestamp": self.timestamp,
            "data": self.data,
            "confidence": self.confidence,
            "pass": self.pass_number,
        }

    @classmethod
    def from_dict(cls, data: dict) -> "DecisionLog":
        return cls(
            decision_type=DecisionType(data["type"]),
            floor=data["floor"],
            timestamp=data.get("timestamp"),
            data=data["data"],
            confidence=data.get("confidence", 1.0),
            pass_number=data.get("pass", 1),
        )


@dataclass
class VODRunState:
    """
    Wraps core RunState with VOD extraction-specific tracking.

    This is the primary state object used during VOD extraction.
    It maintains the game state (via RunState) plus metadata about
    the extraction process itself.
    """
    run: RunState
    video_id: str
    video_url: str = ""
    current_timestamp: str = "00:00"

    # Active combat tracking
    active_combat: Optional[CombatExtraction] = None

    # Decision log (all extracted decisions)
    decisions: list[DecisionLog] = field(default_factory=list)

    # Confidence scores by decision type
    confidence_scores: dict[str, list[float]] = field(default_factory=dict)

    # Path tracking
    map_position: MapPosition = field(default_factory=MapPosition)
    path_history: list[dict] = field(default_factory=list)  # {floor, options, chosen}

    # Extraction metadata
    extraction_started_at: str = field(default_factory=lambda: datetime.now().isoformat())
    extraction_passes: int = 0
    current_pass: int = 1

    # Seed tracking
    seed_detected: bool = False
    seed_verified: bool = False

    @classmethod
    def create(
        cls,
        video_id: str,
        video_url: str = "",
        seed: Optional[str] = None,
        ascension: int = 20,
        character: str = "Watcher",
    ) -> "VODRunState":
        """Create a new VODRunState for extraction."""
        if seed:
            run = create_watcher_run(seed=seed, ascension=ascension)
        else:
            # Create with placeholder seed - will be set when detected
            run = create_watcher_run(seed="UNKNOWN", ascension=ascension)

        return cls(
            run=run,
            video_id=video_id,
            video_url=video_url,
        )

    # --- State accessors (delegate to RunState) ---

    @property
    def floor(self) -> int:
        return self.run.floor

    @property
    def act(self) -> int:
        return self.run.act

    @property
    def hp(self) -> int:
        return self.run.current_hp

    @property
    def max_hp(self) -> int:
        return self.run.max_hp

    @property
    def gold(self) -> int:
        return self.run.gold

    @property
    def deck(self) -> list:
        return self.run.deck

    @property
    def relics(self) -> list:
        return self.run.relics

    @property
    def potions(self) -> list[str]:
        return self.run.get_potions()

    @property
    def seed(self) -> str:
        return self.run.seed_string

    # --- Decision logging ---

    def log_decision(
        self,
        decision_type: DecisionType,
        data: dict,
        floor: Optional[int] = None,
        timestamp: Optional[str] = None,
        confidence: float = 1.0,
    ) -> DecisionLog:
        """Log a decision extracted from the VOD."""
        decision = DecisionLog(
            decision_type=decision_type,
            floor=floor if floor is not None else self.run.floor,
            timestamp=timestamp or self.current_timestamp,
            data=data,
            confidence=confidence,
            pass_number=self.current_pass,
        )
        self.decisions.append(decision)

        # Track confidence by type
        type_key = decision_type.value
        if type_key not in self.confidence_scores:
            self.confidence_scores[type_key] = []
        self.confidence_scores[type_key].append(confidence)

        return decision

    def get_decisions_by_type(self, decision_type: DecisionType) -> list[DecisionLog]:
        """Get all decisions of a specific type."""
        return [d for d in self.decisions if d.decision_type == decision_type]

    def get_decisions_for_floor(self, floor: int) -> list[DecisionLog]:
        """Get all decisions for a specific floor."""
        return [d for d in self.decisions if d.floor == floor]

    # --- Combat tracking ---

    def start_combat(
        self,
        floor: int,
        enemy: str,
        timestamp: Optional[str] = None,
    ) -> CombatExtraction:
        """Start tracking a new combat."""
        self.active_combat = CombatExtraction(
            floor=floor,
            enemy=enemy,
            hp_at_start=self.run.current_hp,
            max_hp=self.run.max_hp,
            timestamp_start=timestamp or self.current_timestamp,
        )
        return self.active_combat

    def end_combat(
        self,
        hp_after: int,
        timestamp: Optional[str] = None,
    ) -> Optional[CombatExtraction]:
        """End current combat tracking."""
        if self.active_combat is None:
            return None

        self.active_combat.timestamp_end = timestamp or self.current_timestamp
        combat = self.active_combat
        self.active_combat = None

        # Update RunState HP
        self.run.current_hp = hp_after

        return combat

    # --- Path tracking ---

    def record_path(
        self,
        floor: int,
        chosen: str,
        options: Optional[list[str]] = None,
    ) -> None:
        """Record a path choice on the map."""
        self.path_history.append({
            "floor": floor,
            "chosen": chosen,
            "options": options or [],
        })

    # --- Serialization ---

    def to_dict(self) -> dict:
        """Serialize full state for checkpointing."""
        return {
            "video_id": self.video_id,
            "video_url": self.video_url,
            "current_timestamp": self.current_timestamp,
            "run": self.run.to_dict(),
            "active_combat": self.active_combat.to_dict() if self.active_combat else None,
            "decisions": [d.to_dict() for d in self.decisions],
            "confidence_scores": self.confidence_scores,
            "map_position": {
                "act": self.map_position.act,
                "x": self.map_position.x,
                "y": self.map_position.y,
            },
            "path_history": self.path_history,
            "extraction_started_at": self.extraction_started_at,
            "extraction_passes": self.extraction_passes,
            "current_pass": self.current_pass,
            "seed_detected": self.seed_detected,
            "seed_verified": self.seed_verified,
        }

    @classmethod
    def from_dict(cls, data: dict) -> "VODRunState":
        """Deserialize from checkpoint."""
        state = cls(
            run=RunState.from_dict(data["run"]),
            video_id=data["video_id"],
            video_url=data.get("video_url", ""),
            current_timestamp=data.get("current_timestamp", "00:00"),
        )

        if data.get("active_combat"):
            ac = data["active_combat"]
            state.active_combat = CombatExtraction(
                floor=ac["floor"],
                enemy=ac["enemy"],
                turn=ac.get("turn_count", 0) + 1,
                hp_at_start=ac["hp_at_start"],
                max_hp=ac["max_hp"],
                cards_played=ac.get("cards_by_turn", []),
                timestamp_start=ac.get("timestamp_start"),
                timestamp_end=ac.get("timestamp_end"),
            )

        state.decisions = [DecisionLog.from_dict(d) for d in data.get("decisions", [])]
        state.confidence_scores = data.get("confidence_scores", {})

        if "map_position" in data:
            mp = data["map_position"]
            state.map_position = MapPosition(act=mp["act"], x=mp["x"], y=mp["y"])

        state.path_history = data.get("path_history", [])
        state.extraction_started_at = data.get("extraction_started_at", datetime.now().isoformat())
        state.extraction_passes = data.get("extraction_passes", 0)
        state.current_pass = data.get("current_pass", 1)
        state.seed_detected = data.get("seed_detected", False)
        state.seed_verified = data.get("seed_verified", False)

        return state

    def snapshot(self) -> dict:
        """Create a minimal snapshot for prompt context."""
        return {
            "floor": self.run.floor,
            "act": self.run.act,
            "hp": self.run.current_hp,
            "max_hp": self.run.max_hp,
            "gold": self.run.gold,
            "deck_size": len(self.run.deck),
            "deck": self.run.get_deck_card_ids(),
            "relics": self.run.get_relic_ids(),
            "potions": self.run.get_potions(),
            "seed": self.run.seed_string if self.seed_detected else "UNKNOWN",
        }

    def copy(self) -> "VODRunState":
        """Create a deep copy for simulation."""
        return VODRunState.from_dict(self.to_dict())

    def __repr__(self) -> str:
        return (
            f"VODRunState(video={self.video_id}, floor={self.run.floor}, "
            f"hp={self.run.current_hp}/{self.run.max_hp}, "
            f"decisions={len(self.decisions)})"
        )
