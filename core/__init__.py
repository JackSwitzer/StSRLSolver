"""
Slay the Spire Core Simulator

A faithful Python recreation of the game's mechanics based on decompiled source.
This is the "ground truth" module - all mechanics must match the original game exactly.

Submodules:
- content: Cards, relics, potions, enemies, powers, stances
- state: RNG system, game state tracking
- calc: Damage and block calculations, combat simulation (CombatSimulator)
- generation: Map generation

Legacy modules (deprecated, will be removed):
- combat: Use core.calc.combat_sim.CombatSimulator instead

Usage:
    from core import CombatSimulator, Random
    from core.content.enemies import JawWorm

    # Setup combat simulator
    sim = CombatSimulator()
    deck = ["Strike_P", "Strike_P", "Defend_P", "Defend_P", "Eruption", "Vigilance"]
    enemies = [JawWorm(Random(12345), ascension=0)]

    # Initialize combat state
    state = sim.setup_combat(deck, enemies, player_hp=80, player_max_hp=80)

    # Get legal actions and execute
    actions = sim.get_legal_actions(state)
    new_state = sim.execute_action(state, actions[0])

    # Or simulate full combat with a policy
    result = sim.simulate_full_combat(state, sim.greedy_policy)
"""

__version__ = "0.1.0"

# RNG System (from state submodule)
from .state.rng import XorShift128, Random, GameRNG, seed_to_long, long_to_seed

# Damage Calculation (from calc submodule)
from .calc.damage import (
    calculate_damage,
    calculate_block,
    calculate_incoming_damage,
    apply_hp_loss,
    wrath_damage,
    divinity_damage,
    # Constants
    WEAK_MULT,
    VULN_MULT,
    FRAIL_MULT,
    WRATH_MULT,
    DIVINITY_MULT,
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

# Combat Simulation - Use CombatSimulator from calc submodule (preferred)
from .calc.combat_sim import CombatSimulator, ActionType, Action, CombatResult

# Legacy Combat (deprecated - will be removed in future version)
# CombatLog is preserved here as the pattern is useful for EV tracking
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

# Encounter Prediction (from generation submodule)
from .generation.encounters import (
    predict_all_acts, predict_all_bosses, predict_all_bosses_extended,
    predict_act_encounters, get_monsterrng_calls_for_act,
    generate_exordium_encounters, generate_city_encounters,
    generate_beyond_encounters, generate_ending_encounters,
    EXORDIUM_BOSSES, CITY_BOSSES, BEYOND_BOSSES,
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
    "calculate_damage", "calculate_block", "calculate_incoming_damage",
    "apply_hp_loss", "wrath_damage", "divinity_damage",
    "WEAK_MULT", "VULN_MULT", "FRAIL_MULT", "WRATH_MULT", "DIVINITY_MULT",
    # Stances
    "StanceID", "StanceEffect", "StanceManager", "STANCES",
    # Cards
    "Card", "CardType", "CardRarity", "CardTarget", "CardColor",
    "WATCHER_CARDS", "get_card", "get_starting_deck",
    # Enemies
    "Enemy", "EnemyState", "MoveInfo", "Intent", "EnemyType",
    "JawWorm", "Cultist", "GremlinNob", "Lagavulin", "SlimeBoss",
    "create_enemy", "ENEMY_CLASSES",
    # Combat Simulation (preferred)
    "CombatSimulator", "ActionType", "Action", "CombatResult",
    # Combat (legacy, deprecated)
    "Combat", "CombatPhase", "PlayerState", "CombatLog",
    # Map
    "MapGenerator", "MapGeneratorConfig", "MapRoomNode", "MapEdge", "RoomType",
    "generate_act4_map", "get_map_seed_offset", "map_to_string",
    # Encounters
    "predict_all_acts", "predict_all_bosses", "predict_all_bosses_extended",
    "predict_act_encounters", "get_monsterrng_calls_for_act",
    "generate_exordium_encounters", "generate_city_encounters",
    "generate_beyond_encounters", "generate_ending_encounters",
    "EXORDIUM_BOSSES", "CITY_BOSSES", "BEYOND_BOSSES",
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
