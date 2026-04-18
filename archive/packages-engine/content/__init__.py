"""
Content module - All game content data definitions.

Contains cards, relics, potions, enemies, powers, stances, and events.
"""

# Cards
from .cards import (
    Card, CardType, CardRarity, CardTarget, CardColor, CardEffect,
    WATCHER_CARDS, COLORLESS_CARDS, CURSE_CARDS, STATUS_CARDS, ALL_CARDS,
    get_card, get_starting_deck,
)

# Relics
from .relics import (
    Relic, RelicTier, RelicEffect,
    ALL_RELICS, STARTER_RELICS, COMMON_RELICS, UNCOMMON_RELICS,
    RARE_RELICS, BOSS_RELICS, SHOP_RELICS, SPECIAL_RELICS,
    get_relic, get_relics_by_tier, get_relics_for_class, get_starter_relic,
    PlayerClass as RelicPlayerClass,
)

# Potions
from .potions import (
    Potion, PotionRarity, PotionTargetType, PlayerClass,
    ALL_POTIONS, COMMON_POTIONS, UNCOMMON_POTIONS, RARE_POTIONS,
    UNIVERSAL_POTIONS, IRONCLAD_POTIONS, SILENT_POTIONS, DEFECT_POTIONS, WATCHER_POTIONS,
    get_potion_pool, get_potion_by_id, calculate_potion_slots, calculate_drop_chance,
    POTION_COMMON_CHANCE, POTION_UNCOMMON_CHANCE, POTION_RARE_CHANCE,
    BASE_POTION_DROP_CHANCE, BLIZZARD_MOD_STEP,
)

# Powers
from .powers import (
    Power, PowerType, PowerManager, DamageType,
    create_power, create_strength, create_dexterity, create_weak, create_vulnerable,
    create_frail, create_poison, create_artifact, create_intangible,
    create_vigor, create_mantra, POWER_DATA,
    WEAK_MULTIPLIER, VULNERABLE_MULTIPLIER, FRAIL_MULTIPLIER,
)

# Stances
from .stances import StanceID, StanceEffect, StanceManager, STANCES

# Enemies
from .enemies import (
    Enemy, EnemyState, MoveInfo, Intent, EnemyType,
    JawWorm, Cultist, GremlinNob, Lagavulin, SlimeBoss,
    create_enemy, ENEMY_CLASSES,
)

# Enemy Data (pure data access - HP ranges, damage values, move IDs)
from .enemies import (
    ENEMY_DATA,
    get_hp_range as enemy_get_hp_range,
    get_damage_value as enemy_get_damage_value,
    get_damage_values as enemy_get_damage_values,
    get_enemy_type as enemy_get_type_from_data,
    get_move_name as enemy_get_move_name,
)

# Events
from .events import (
    Event, EventChoice, Outcome, OutcomeType, Act, NeowBonus,
    ALL_EVENTS, EXORDIUM_EVENTS, CITY_EVENTS, BEYOND_EVENTS, SHRINE_EVENTS,
    get_event, get_events_for_act, calculate_outcome_value,
)

__all__ = [
    # Cards
    "Card", "CardType", "CardRarity", "CardTarget", "CardColor", "CardEffect",
    "WATCHER_CARDS", "COLORLESS_CARDS", "CURSE_CARDS", "STATUS_CARDS", "ALL_CARDS",
    "get_card", "get_starting_deck",
    # Relics
    "Relic", "RelicTier", "RelicEffect",
    "ALL_RELICS", "STARTER_RELICS", "COMMON_RELICS", "UNCOMMON_RELICS",
    "RARE_RELICS", "BOSS_RELICS", "SHOP_RELICS", "SPECIAL_RELICS",
    "get_relic", "get_relics_by_tier", "get_relics_for_class", "get_starter_relic",
    "RelicPlayerClass",
    # Potions
    "Potion", "PotionRarity", "PotionTargetType", "PlayerClass",
    "ALL_POTIONS", "COMMON_POTIONS", "UNCOMMON_POTIONS", "RARE_POTIONS",
    "UNIVERSAL_POTIONS", "IRONCLAD_POTIONS", "SILENT_POTIONS", "DEFECT_POTIONS", "WATCHER_POTIONS",
    "get_potion_pool", "get_potion_by_id", "calculate_potion_slots", "calculate_drop_chance",
    "POTION_COMMON_CHANCE", "POTION_UNCOMMON_CHANCE", "POTION_RARE_CHANCE",
    "BASE_POTION_DROP_CHANCE", "BLIZZARD_MOD_STEP",
    # Powers
    "Power", "PowerType", "PowerManager", "DamageType",
    "create_power", "create_strength", "create_dexterity", "create_weak", "create_vulnerable",
    "create_frail", "create_poison", "create_artifact", "create_intangible",
    "create_vigor", "create_mantra", "POWER_DATA",
    "WEAK_MULTIPLIER", "VULNERABLE_MULTIPLIER", "FRAIL_MULTIPLIER",
    # Stances
    "StanceID", "StanceEffect", "StanceManager", "STANCES",
    # Enemies
    "Enemy", "EnemyState", "MoveInfo", "Intent", "EnemyType",
    "JawWorm", "Cultist", "GremlinNob", "Lagavulin", "SlimeBoss",
    "create_enemy", "ENEMY_CLASSES",
    # Enemy Data
    "ENEMY_DATA",
    "enemy_get_hp_range", "enemy_get_damage_value", "enemy_get_damage_values",
    "enemy_get_type_from_data", "enemy_get_move_name",
    # Events
    "Event", "EventChoice", "Outcome", "OutcomeType", "Act", "NeowBonus",
    "ALL_EVENTS", "EXORDIUM_EVENTS", "CITY_EVENTS", "BEYOND_EVENTS", "SHRINE_EVENTS",
    "get_event", "get_events_for_act", "calculate_outcome_value",
]
