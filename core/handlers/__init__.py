"""
Handlers for Slay the Spire RL.

Combat Handlers:
- CombatRunner: Runs combat encounters
- CombatResult: Result of combat
- create_encounter: Create encounter from enemy list

Room Handlers:
- EventHandler: Event selection and outcome application
- ShopHandler: Shop generation and purchases
- RestHandler: Rest site actions (rest, upgrade, dig, lift, recall, toke)
- TreasureHandler: Chest opening and key mechanics
- RewardHandler: Combat reward collection
"""

from .combat import CombatRunner, CombatResult, create_encounter
from .rooms import (
    # Handlers
    EventHandler,
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

__all__ = [
    # Combat
    "CombatRunner",
    "CombatResult",
    "create_encounter",

    # Room Handlers
    "EventHandler",
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
