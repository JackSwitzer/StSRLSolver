"""
Slay the Spire Engine

A faithful Python recreation of the game's mechanics based on decompiled Java source.
All mechanics must match the original game exactly (verified via parity tests).

Core subsystems:
- state: RNG (XorShift128), run state, combat state
- content: Cards, enemies, relics, potions, powers, stances, events
- generation: Map, encounters, rewards, shops, potions, treasure
- handlers: Room-phase logic (combat, events, shops, rest, rewards)
- calc: Damage/block formulas, combat tree-search simulator
- effects: Card effect execution system

Usage:
    from packages.engine import GameRunner, GamePhase

    runner = GameRunner(seed="SEED123", ascension=20)
    while not runner.game_over:
        actions = runner.get_available_actions()
        runner.take_action(actions[0])

    from packages.engine import CombatSimulator, Random
    sim = CombatSimulator()
    state = sim.setup_combat(deck, enemies, player_hp=80, player_max_hp=80)
    actions = sim.get_legal_actions(state)
    new_state = sim.execute_action(state, actions[0])
"""

__version__ = "0.2.0"

# RNG System
from .state.rng import XorShift128, Random, GameRNG, seed_to_long, long_to_seed

# Damage Calculation
from .calc.damage import (
    calculate_damage,
    calculate_block,
    calculate_incoming_damage,
    apply_hp_loss,
    wrath_damage,
    divinity_damage,
    WEAK_MULT,
    VULN_MULT,
    FRAIL_MULT,
    WRATH_MULT,
    DIVINITY_MULT,
)

# Stance System
from .content.stances import StanceID, StanceEffect, StanceManager, STANCES

# Cards
from .content.cards import (
    Card, CardType, CardRarity, CardTarget, CardColor,
    WATCHER_CARDS, get_card, get_starting_deck
)

# Enemies
from .content.enemies import (
    Enemy, EnemyState, MoveInfo, Intent, EnemyType,
    JawWorm, Cultist, GremlinNob, Lagavulin, SlimeBoss,
    create_enemy, ENEMY_CLASSES
)

# Combat Simulation
from .calc.combat_sim import CombatSimulator, ActionType, Action, CombatResult

# Powers System
from .content.powers import (
    Power as PowerInstance, PowerType, PowerManager, DamageType as PowerDamageType,
    create_power, create_strength, create_dexterity, create_weak, create_vulnerable,
    create_frail, create_poison, create_artifact, create_intangible,
    create_vigor, create_mantra, POWER_DATA,
    WEAK_MULTIPLIER, VULNERABLE_MULTIPLIER, FRAIL_MULTIPLIER,
)

# Map Generation
from .generation.map import (
    MapGenerator, MapGeneratorConfig, MapRoomNode, MapEdge, RoomType,
    generate_act4_map, get_map_seed_offset, map_to_string
)

# Encounter Prediction
from .generation.encounters import (
    predict_all_acts, predict_all_bosses, predict_all_bosses_extended,
    predict_act_encounters, get_monsterrng_calls_for_act,
    generate_exordium_encounters, generate_city_encounters,
    generate_beyond_encounters, generate_ending_encounters,
    EXORDIUM_BOSSES, CITY_BOSSES, BEYOND_BOSSES,
)

# Potions
from .content.potions import (
    Potion, PotionRarity, PotionTargetType, PlayerClass,
    ALL_POTIONS, COMMON_POTIONS, UNCOMMON_POTIONS, RARE_POTIONS,
    UNIVERSAL_POTIONS, IRONCLAD_POTIONS, SILENT_POTIONS, DEFECT_POTIONS, WATCHER_POTIONS,
    get_potion_pool, get_potion_by_id, calculate_potion_slots, calculate_drop_chance,
    POTION_COMMON_CHANCE, POTION_UNCOMMON_CHANCE, POTION_RARE_CHANCE,
    BASE_POTION_DROP_CHANCE, BLIZZARD_MOD_STEP,
)

# Relics
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
    RunResult, run_headless, run_parallel,
)

# State objects (for observation/serialization)
from .state.combat import (
    CombatState, EntityState, EnemyCombatState,
    PlayCard, UsePotion, EndTurn,
    create_player, create_enemy as create_combat_enemy, create_combat,
)
from .state.run import RunState, CardInstance, RelicInstance, PotionSlot, MapPosition

# Combat Engine (direct access)
from .combat_engine import CombatEngine

# Agent API (JSON-serializable action/observation interface)
# GameRunner implements JSON methods directly; agent_api is a compatibility shim.
from . import agent_api
from .agent_api import ActionDict, ActionResult, ObservationDict
