"""
Slay the Spire Core Simulator

A faithful Python recreation of the game's mechanics based on decompiled source.
This is the "ground truth" module - all mechanics must match the original game exactly.

Submodules:
- content: Cards, relics, potions, enemies, powers, stances
- state: RNG system, game state tracking
- calc: Damage and block calculations
- generation: Map generation

Legacy modules (still in core root):
- combat: Combat simulation
- damage: Damage calculation utilities

Usage:
    from core import GameState, simulate_combat, predict_enemy_move

    # Create game state from seed
    state = GameState(seed="ABC123XYZ", ascension=20, character="Watcher")

    # Predict enemy's next move
    move = predict_enemy_move(state, enemy_id="JawWorm")

    # Simulate combat outcome
    result = simulate_combat(state, deck, enemies)
"""

__version__ = "0.1.0"

# RNG System (from state submodule)
from .state.rng import XorShift128, Random, GameRNG, seed_to_long, long_to_seed

# Damage Calculation (still in core root)
from .damage import (
    DamageType, Power, Relic, CombatState,
    calculate_card_damage, calculate_block, calculate_incoming_damage,
    wrath_damage, divinity_damage
)

# Stance System (from content submodule)
from .content.stances import StanceID, StanceEffect, StanceManager, STANCES

# Cards (from content submodule)
from .content.cards import (
    Card, CardType, CardRarity, CardTarget, CardColor,
    WATCHER_CARDS, get_card, get_starting_deck
)

# Enemies (from content submodule)
from .content.enemies import (
    Enemy, EnemyState, MoveInfo, Intent, EnemyType,
    JawWorm, Cultist, GremlinNob, Lagavulin, SlimeBoss,
    create_enemy, ENEMY_CLASSES
)

# Combat Simulation (still in core root)
from .combat import Combat, CombatPhase, PlayerState, CombatLog

# Powers System (from content submodule)
from .content.powers import (
    Power as PowerInstance, PowerType, PowerManager, DamageType as PowerDamageType,
    create_power, create_strength, create_dexterity, create_weak, create_vulnerable,
    create_frail, create_poison, create_artifact, create_intangible,
    create_vigor, create_mantra, POWER_DATA,
    WEAK_MULTIPLIER, VULNERABLE_MULTIPLIER, FRAIL_MULTIPLIER,
)

# Map Generation (from generation submodule)
from .generation.map import (
    MapGenerator, MapGeneratorConfig, MapRoomNode, MapEdge, RoomType,
    generate_act4_map, get_map_seed_offset, map_to_string
)

# Potions (from content submodule)
from .content.potions import (
    Potion, PotionRarity, PotionTargetType, PlayerClass,
    ALL_POTIONS, COMMON_POTIONS, UNCOMMON_POTIONS, RARE_POTIONS,
    UNIVERSAL_POTIONS, IRONCLAD_POTIONS, SILENT_POTIONS, DEFECT_POTIONS, WATCHER_POTIONS,
    get_potion_pool, get_potion_by_id, calculate_potion_slots, calculate_drop_chance,
    POTION_COMMON_CHANCE, POTION_UNCOMMON_CHANCE, POTION_RARE_CHANCE,
    BASE_POTION_DROP_CHANCE, BLIZZARD_MOD_STEP,
)

# Relics (from content submodule)
from .content.relics import (
    Relic, RelicTier, RelicEffect,
    ALL_RELICS, STARTER_RELICS, COMMON_RELICS, UNCOMMON_RELICS,
    RARE_RELICS, BOSS_RELICS, SHOP_RELICS, SPECIAL_RELICS,
    get_relic, get_relics_by_tier, get_relics_for_class, get_starter_relic,
    PlayerClass as RelicPlayerClass,
)

# Game Runner
from .game import (
    GameRunner, GamePhase,
    PathAction, NeowAction, CombatAction, RewardAction,
    EventAction, ShopAction, RestAction, TreasureAction, BossRewardAction,
    DecisionLogEntry, GameAction,
)

__all__ = [
    # Version
    "__version__",
    # RNG
    "XorShift128", "Random", "GameRNG", "seed_to_long", "long_to_seed",
    # Damage
    "DamageType", "Power", "Relic", "CombatState",
    "calculate_card_damage", "calculate_block", "calculate_incoming_damage",
    "wrath_damage", "divinity_damage",
    # Stances
    "StanceID", "StanceEffect", "StanceManager", "STANCES",
    # Cards
    "Card", "CardType", "CardRarity", "CardTarget", "CardColor",
    "WATCHER_CARDS", "get_card", "get_starting_deck",
    # Enemies
    "Enemy", "EnemyState", "MoveInfo", "Intent", "EnemyType",
    "JawWorm", "Cultist", "GremlinNob", "Lagavulin", "SlimeBoss",
    "create_enemy", "ENEMY_CLASSES",
    # Combat
    "Combat", "CombatPhase", "PlayerState", "CombatLog",
    # Map
    "MapGenerator", "MapGeneratorConfig", "MapRoomNode", "MapEdge", "RoomType",
    "generate_act4_map", "get_map_seed_offset", "map_to_string",
    # Powers
    "PowerInstance", "PowerType", "PowerManager", "PowerDamageType",
    "create_power", "create_strength", "create_dexterity", "create_weak", "create_vulnerable",
    "create_frail", "create_poison", "create_artifact", "create_intangible",
    "create_vigor", "create_mantra", "POWER_DATA",
    "WEAK_MULTIPLIER", "VULNERABLE_MULTIPLIER", "FRAIL_MULTIPLIER",
    # Potions
    "Potion", "PotionRarity", "PotionTargetType", "PlayerClass",
    "ALL_POTIONS", "COMMON_POTIONS", "UNCOMMON_POTIONS", "RARE_POTIONS",
    "UNIVERSAL_POTIONS", "IRONCLAD_POTIONS", "SILENT_POTIONS", "DEFECT_POTIONS", "WATCHER_POTIONS",
    "get_potion_pool", "get_potion_by_id", "calculate_potion_slots", "calculate_drop_chance",
    "POTION_COMMON_CHANCE", "POTION_UNCOMMON_CHANCE", "POTION_RARE_CHANCE",
    "BASE_POTION_DROP_CHANCE", "BLIZZARD_MOD_STEP",
    # Relics
    "Relic", "RelicTier", "RelicEffect",
    "ALL_RELICS", "STARTER_RELICS", "COMMON_RELICS", "UNCOMMON_RELICS",
    "RARE_RELICS", "BOSS_RELICS", "SHOP_RELICS", "SPECIAL_RELICS",
    "get_relic", "get_relics_by_tier", "get_relics_for_class", "get_starter_relic",
    "RelicPlayerClass",
    # Game Runner
    "GameRunner", "GamePhase",
    "PathAction", "NeowAction", "CombatAction", "RewardAction",
    "EventAction", "ShopAction", "RestAction", "TreasureAction", "BossRewardAction",
    "DecisionLogEntry", "GameAction",
]
