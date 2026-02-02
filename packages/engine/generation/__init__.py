"""
Generation module - Map, rewards, and content generation.

Contains:
- Map generation algorithm matching the game exactly
- Card, relic, potion, and gold reward generation
- Shop inventory generation
- Pity timer (blizzard) systems for rare cards and potions
- Treasure room (chest) prediction
- Boss prediction for all acts
"""

# Map Generation
from .map import (
    MapGenerator, MapGeneratorConfig, MapRoomNode, MapEdge, RoomType,
    generate_act4_map, get_map_seed_offset, map_to_string,
)

# Reward Generation
from .rewards import (
    # State tracking classes
    CardBlizzardState,
    PotionBlizzardState,
    RewardState,
    ShopInventory,
    # Card rewards
    generate_card_rewards,
    generate_colorless_card_rewards,
    # Relic rewards
    generate_relic_reward,
    generate_elite_relic_reward,
    generate_boss_relics,
    # Potion rewards
    check_potion_drop,
    generate_potion_reward,
    # Gold rewards
    generate_gold_reward,
    # Shop generation
    generate_shop_inventory,
    # Constants
    CARD_BLIZZ_START_OFFSET,
    CARD_BLIZZ_GROWTH,
    CARD_BLIZZ_MAX_OFFSET,
    CARD_RARITY_THRESHOLDS,
    CARD_UPGRADE_CHANCES,
)

# Treasure Room Prediction
from .treasure import (
    # Enums and data classes
    ChestType,
    TreasureReward,
    ChestPrediction,
    # Prediction functions
    predict_chest,
    predict_chest_relic,
    predict_full_chest,
    predict_treasure_sequence,
    # Utility
    get_treasure_counter_after_chest,
    # Constants
    CHEST_TYPE_THRESHOLDS,
    CHEST_RELIC_THRESHOLDS,
    CHEST_GOLD_CHANCE,
    CHEST_GOLD_BASE,
)

# Potion Drop Prediction
from .potions import (
    # Data classes
    PotionPrediction,
    # Prediction functions
    predict_potion_drop,
    predict_potion_from_seed,
    predict_multiple_potion_drops,
    # Pool utilities
    get_potion_pool_for_class,
    get_potion_by_id,
    get_rng_calls_for_potion_selection,
    # Pool constants (exact game order)
    WATCHER_POTION_POOL,
    IRONCLAD_POTION_POOL,
    SILENT_POTION_POOL,
    DEFECT_POTION_POOL,
)

# Shop Inventory Prediction
from .shop import (
    predict_shop_inventory,
    format_shop_inventory,
    ShopCard,
    ShopRelic,
    ShopPotion,
    PredictedShopInventory,
    ShopPredictionResult,
)

# Encounter Prediction (monsters, elites, bosses for all acts)
from .encounters import (
    # Boss prediction
    predict_all_bosses,
    predict_all_bosses_extended,
    # Encounter prediction
    predict_act_encounters,
    predict_all_acts,
    get_monsterrng_calls_for_act,
    # Generation functions
    generate_exordium_encounters,
    generate_city_encounters,
    generate_beyond_encounters,
    generate_ending_encounters,
    get_ending_encounters,
    # Boss lists
    EXORDIUM_BOSSES,
    CITY_BOSSES,
    BEYOND_BOSSES,
    ENDING_BOSSES,
    ENDING_ELITE,
    ENDING_BOSS,
    ACT_4_KEYS,
)

__all__ = [
    # Map Generation
    "MapGenerator", "MapGeneratorConfig", "MapRoomNode", "MapEdge", "RoomType",
    "generate_act4_map", "get_map_seed_offset", "map_to_string",
    # Reward State
    "CardBlizzardState", "PotionBlizzardState", "RewardState", "ShopInventory",
    # Card Rewards
    "generate_card_rewards", "generate_colorless_card_rewards",
    # Relic Rewards
    "generate_relic_reward", "generate_elite_relic_reward", "generate_boss_relics",
    # Potion Rewards
    "check_potion_drop", "generate_potion_reward",
    # Gold Rewards
    "generate_gold_reward",
    # Shop Generation
    "generate_shop_inventory",
    # Treasure Room
    "ChestType", "TreasureReward", "ChestPrediction",
    "predict_chest", "predict_chest_relic", "predict_full_chest",
    "predict_treasure_sequence", "get_treasure_counter_after_chest",
    "CHEST_TYPE_THRESHOLDS", "CHEST_RELIC_THRESHOLDS",
    "CHEST_GOLD_CHANCE", "CHEST_GOLD_BASE",
    # Boss Prediction (now part of encounters)
    "predict_all_bosses", "predict_all_bosses_extended",
    # Potion Prediction
    "PotionPrediction",
    "predict_potion_drop", "predict_potion_from_seed", "predict_multiple_potion_drops",
    "get_potion_pool_for_class", "get_potion_by_id", "get_rng_calls_for_potion_selection",
    "WATCHER_POTION_POOL", "IRONCLAD_POTION_POOL", "SILENT_POTION_POOL", "DEFECT_POTION_POOL",
    # Shop Prediction
    "predict_shop_inventory", "format_shop_inventory",
    "ShopCard", "ShopRelic", "ShopPotion",
    "PredictedShopInventory", "ShopPredictionResult",
    # Encounter Prediction
    "predict_act_encounters", "predict_all_acts", "get_monsterrng_calls_for_act",
    "generate_exordium_encounters", "generate_city_encounters", "generate_beyond_encounters",
    "generate_ending_encounters", "get_ending_encounters",
    "EXORDIUM_BOSSES", "CITY_BOSSES", "BEYOND_BOSSES",
    "ENDING_BOSSES", "ENDING_ELITE", "ENDING_BOSS", "ACT_4_KEYS",
    # Constants
    "CARD_BLIZZ_START_OFFSET", "CARD_BLIZZ_GROWTH", "CARD_BLIZZ_MAX_OFFSET",
    "CARD_RARITY_THRESHOLDS", "CARD_UPGRADE_CHANCES",
]
