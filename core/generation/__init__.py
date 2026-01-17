"""
Generation module - Map, rewards, and content generation.

Contains:
- Map generation algorithm matching the game exactly
- Card, relic, potion, and gold reward generation
- Shop inventory generation
- Pity timer (blizzard) systems for rare cards and potions
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
    # Constants
    "CARD_BLIZZ_START_OFFSET", "CARD_BLIZZ_GROWTH", "CARD_BLIZZ_MAX_OFFSET",
    "CARD_RARITY_THRESHOLDS", "CARD_UPGRADE_CHANCES",
]
