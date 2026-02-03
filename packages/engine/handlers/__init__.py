"""
Handlers for Slay the Spire RL.

Combat Handlers:
- CombatRunner: Runs combat encounters
- CombatResult: Result of combat
- create_encounter: Create encounter from enemy list

Room Handlers:
- EventHandler (new): Complete event system with selection, choices, and outcomes
- ShopHandler: Shop generation and purchases
- RestHandler: Rest site actions (rest, upgrade, dig, lift, recall, toke)
- TreasureHandler: Chest opening and key mechanics
- RewardHandler: Combat reward collection
"""

from .combat import CombatRunner, CombatResult, create_enemies_from_encounter, ENCOUNTER_TABLE
from .rooms import (
    # Handlers (legacy - EventHandler moved to event_handler.py)
    EventHandler as LegacyEventHandler,
    ShopHandler,
    RestHandler,
    TreasureHandler,
    RewardHandler,

    # Result dataclasses
    EventResult,
    ShopResult,
    RestResult,
    TreasureResult,
    CombatRewards,
    RewardResult,
)

# New complete event system
from .event_handler import (
    EventHandler,
    EventState,
    EventPhase,
    EventChoiceResult,
    EventChoice,
    EventDefinition,
    ACT1_EVENTS,
    ACT2_EVENTS,
    ACT3_EVENTS,
    SHRINE_EVENTS,
    SPECIAL_ONE_TIME_EVENTS,
)

__all__ = [
    # Combat
    "CombatRunner",
    "CombatResult",
    "create_enemies_from_encounter",

    # Event Handler (new complete system)
    "EventHandler",
    "EventState",
    "EventPhase",
    "EventChoiceResult",
    "EventChoice",
    "EventDefinition",
    "ACT1_EVENTS",
    "ACT2_EVENTS",
    "ACT3_EVENTS",
    "SHRINE_EVENTS",
    "SPECIAL_ONE_TIME_EVENTS",

    # Other Room Handlers
    "ShopHandler",
    "RestHandler",
    "TreasureHandler",
    "RewardHandler",

    # Result types
    "EventResult",
    "ShopResult",
    "RestResult",
    "TreasureResult",
    "CombatRewards",
    "RewardResult",
]
