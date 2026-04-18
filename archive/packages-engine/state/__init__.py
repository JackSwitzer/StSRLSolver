"""
State module - Game state management and RNG.

Contains:
- RNG system (XorShift128, seed management)
- Run state tracking (deck, relics, HP, map position, etc.)
- Combat state tracking for tree search
"""

# RNG System
from .rng import XorShift128, Random, GameRNG, seed_to_long, long_to_seed

# Run State Tracking
from .run import (
    RunState,
    CardInstance,
    RelicInstance,
    PotionSlot,
    MapPosition,
    create_watcher_run,
    create_ironclad_run,
    create_silent_run,
    create_defect_run,
    create_run_from_save,
    BASE_STARTING_GOLD,
    WATCHER_STARTING_DECK,
    WATCHER_STARTING_RELIC,
    WATCHER_BASE_HP,
    IRONCLAD_STARTING_DECK,
    IRONCLAD_STARTING_RELIC,
    IRONCLAD_BASE_HP,
    SILENT_STARTING_DECK,
    SILENT_STARTING_RELIC,
    SILENT_BASE_HP,
    DEFECT_STARTING_DECK,
    DEFECT_STARTING_RELIC,
    DEFECT_BASE_HP,
)

# Combat State (for tree search/simulation)
from .combat import (
    CombatState,
    EntityState,
    EnemyCombatState,
    PlayCard,
    UsePotion,
    EndTurn,
    create_player,
    create_enemy as create_combat_enemy,
    create_combat,
)

__all__ = [
    # RNG
    "XorShift128", "Random", "GameRNG", "seed_to_long", "long_to_seed",
    # Run State
    "RunState",
    "CardInstance",
    "RelicInstance",
    "PotionSlot",
    "MapPosition",
    "create_watcher_run",
    "create_ironclad_run",
    "create_silent_run",
    "create_defect_run",
    "create_run_from_save",
    "BASE_STARTING_GOLD",
    "WATCHER_STARTING_DECK",
    "WATCHER_STARTING_RELIC",
    "WATCHER_BASE_HP",
    "IRONCLAD_STARTING_DECK",
    "IRONCLAD_STARTING_RELIC",
    "IRONCLAD_BASE_HP",
    "SILENT_STARTING_DECK",
    "SILENT_STARTING_RELIC",
    "SILENT_BASE_HP",
    "DEFECT_STARTING_DECK",
    "DEFECT_STARTING_RELIC",
    "DEFECT_BASE_HP",
    # Combat State
    "CombatState",
    "EntityState",
    "EnemyCombatState",
    "PlayCard",
    "UsePotion",
    "EndTurn",
    "create_player",
    "create_combat_enemy",
    "create_combat",
]
