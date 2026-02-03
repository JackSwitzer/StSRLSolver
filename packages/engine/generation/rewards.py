"""
Slay the Spire - Reward Generation System

Implements all reward mechanics from the decompiled game source:
- Card rewards with rarity rolls and pity timer (cardBlizzRandomizer)
- Relic rewards with tier-based probability distribution
- Boss relic selection (3 choices)
- Potion drops with blizzard modifier
- Gold rewards by room type
- Shop inventory generation

All RNG uses the game's XorShift128 algorithm for seed reproducibility.
"""

from dataclasses import dataclass, field
from typing import List, Optional, Dict, Set, Tuple, Any
from enum import Enum
import sys
import os

# Setup import path - use importlib to avoid __init__.py circular imports
import importlib.util

_core_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

def _load_module(name: str, filepath: str):
    """Load a module directly from file path."""
    spec = importlib.util.spec_from_file_location(name, filepath)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

# Load modules directly to avoid __init__.py imports
_rng_module = _load_module("rng", os.path.join(_core_dir, "state", "rng.py"))
Random = _rng_module.Random
GameRNG = _rng_module.GameRNG
seed_to_long = _rng_module.seed_to_long

_cards_module = _load_module("cards", os.path.join(_core_dir, "content", "cards.py"))
Card = _cards_module.Card
CardRarity = _cards_module.CardRarity
CardColor = _cards_module.CardColor
CardType = _cards_module.CardType
IRONCLAD_CARDS = _cards_module.IRONCLAD_CARDS
SILENT_CARDS = _cards_module.SILENT_CARDS
DEFECT_CARDS = _cards_module.DEFECT_CARDS
WATCHER_CARDS = _cards_module.WATCHER_CARDS
COLORLESS_CARDS = _cards_module.COLORLESS_CARDS
CURSE_CARDS = _cards_module.CURSE_CARDS
get_card = _cards_module.get_card

_relics_module = _load_module("relics", os.path.join(_core_dir, "content", "relics.py"))
Relic = _relics_module.Relic
RelicTier = _relics_module.RelicTier
PlayerClass = _relics_module.PlayerClass
ALL_RELICS = _relics_module.ALL_RELICS
COMMON_RELICS = _relics_module.COMMON_RELICS
UNCOMMON_RELICS = _relics_module.UNCOMMON_RELICS
RARE_RELICS = _relics_module.RARE_RELICS
BOSS_RELICS = _relics_module.BOSS_RELICS
SHOP_RELICS = _relics_module.SHOP_RELICS
SPECIAL_RELICS = _relics_module.SPECIAL_RELICS
get_relic = _relics_module.get_relic
get_relics_by_tier = _relics_module.get_relics_by_tier

_potions_module = _load_module("potions", os.path.join(_core_dir, "content", "potions.py"))
Potion = _potions_module.Potion
PotionRarity = _potions_module.PotionRarity
PotionPlayerClass = _potions_module.PlayerClass
ALL_POTIONS = _potions_module.ALL_POTIONS
COMMON_POTIONS = _potions_module.COMMON_POTIONS
UNCOMMON_POTIONS = _potions_module.UNCOMMON_POTIONS
RARE_POTIONS = _potions_module.RARE_POTIONS
get_potion_pool = _potions_module.get_potion_pool
get_potion_by_id = _potions_module.get_potion_by_id
POTION_COMMON_CHANCE = _potions_module.POTION_COMMON_CHANCE
POTION_UNCOMMON_CHANCE = _potions_module.POTION_UNCOMMON_CHANCE
POTION_RARE_CHANCE = _potions_module.POTION_RARE_CHANCE
BASE_POTION_DROP_CHANCE = _potions_module.BASE_POTION_DROP_CHANCE
BLIZZARD_MOD_STEP = _potions_module.BLIZZARD_MOD_STEP

_hashmap_module = _load_module("java_hashmap", os.path.join(_core_dir, "utils", "java_hashmap.py"))
get_java_iteration_order = _hashmap_module.get_java_iteration_order


# ============================================================================
# CONSTANTS - Extracted from decompiled source
# ============================================================================

# Card Blizzard (pity timer) constants from AbstractDungeon.java
CARD_BLIZZ_START_OFFSET = 5  # Initial offset (+5 to roll)
CARD_BLIZZ_GROWTH = 1        # Decrease per common card
CARD_BLIZZ_MAX_OFFSET = -40  # Minimum offset (maximum penalty)

# Card rarity thresholds by room type
# Normal room (AbstractRoom base): rare=3, uncommon=37
# Elite room (MonsterRoomElite): rare=10, uncommon=40
# Shop room: rare=3, uncommon=37 (uses AbstractDungeon.rollRarity())
CARD_RARITY_THRESHOLDS = {
    "normal": {"rare": 3, "uncommon": 37},
    "elite": {"rare": 10, "uncommon": 40},
    "shop": {"rare": 3, "uncommon": 37},  # Same as normal/fallback
    "fallback": {"rare": 3, "uncommon": 37},  # getCardRarityFallback
}

# Card upgrade chances by act (from dungeon classes)
# Format: {act: {ascension_12_plus: chance, default: chance}}
CARD_UPGRADE_CHANCES = {
    1: 0.0,                        # Exordium: 0%
    2: {"default": 0.25, "a12": 0.125},  # TheCity: 25% or 12.5% at A12+
    3: {"default": 0.50, "a12": 0.25},   # TheBeyond: 50% or 25% at A12+
    4: {"default": 0.50, "a12": 0.25},   # TheEnding: 50% or 25% at A12+
}

# Elite relic tier thresholds (MonsterRoomElite.returnRandomRelicTier)
# roll 0-99: <50 = COMMON, >82 = RARE, else UNCOMMON
ELITE_RELIC_THRESHOLDS = {"common": 50, "rare": 82}

# Normal room relic tier thresholds (MonsterRoom.returnRandomRelicTier)
# roll 0-99: <50 = COMMON, >85 = RARE, else UNCOMMON
NORMAL_RELIC_THRESHOLDS = {"common": 50, "rare": 85}

# Shop relic tier thresholds (ShopScreen.rollRelicTier)
# roll 0-99: <48 = COMMON, <82 = UNCOMMON, else RARE
SHOP_RELIC_THRESHOLDS = {"common": 48, "uncommon": 82}

# Random relic tier thresholds (AbstractDungeon.returnRandomRelicTier)
# Acts 1-3: commonRelicChance=50, uncommonRelicChance=33
# Act 4: commonRelicChance=0, uncommonRelicChance=100
RANDOM_RELIC_CHANCES = {
    "default": {"common": 50, "uncommon": 33},  # Acts 1-3
    "act4": {"common": 0, "uncommon": 100},     # TheEnding
}

# Gold reward amounts by room type
GOLD_REWARDS = {
    "boss": {"base": 100, "variance": 5, "a13_multiplier": 0.75},
    "elite": {"min": 25, "max": 35, "a13_fixed": 30},
    "normal": {"min": 10, "max": 20, "a13_fixed": 15},
}


# ============================================================================
# DATA CLASSES
# ============================================================================

@dataclass
class CardBlizzardState:
    """
    Tracks the card rarity pity timer state.

    The blizzard system increases rare card chances when you get commons:
    - Each common card: offset decreases by CARD_BLIZZ_GROWTH (1)
    - Getting a rare: offset resets to CARD_BLIZZ_START_OFFSET (5)
    - Offset adds to the roll, so negative = higher rare chance
    - Minimum offset is CARD_BLIZZ_MAX_OFFSET (-40)
    """
    offset: int = CARD_BLIZZ_START_OFFSET

    def on_common(self):
        """Called when a common card is rolled."""
        self.offset -= CARD_BLIZZ_GROWTH
        if self.offset < CARD_BLIZZ_MAX_OFFSET:
            self.offset = CARD_BLIZZ_MAX_OFFSET

    def on_rare(self):
        """Called when a rare card is rolled."""
        self.offset = CARD_BLIZZ_START_OFFSET

    def on_uncommon(self):
        """Called when an uncommon card is rolled - no change."""
        pass


@dataclass
class PotionBlizzardState:
    """
    Tracks the potion drop pity timer state.

    The blizzard system adjusts potion drop chances:
    - Each potion dropped: modifier decreases by 10%
    - Each combat without potion: modifier increases by 10%
    """
    modifier: int = 0

    def on_drop(self):
        """Called when a potion drops."""
        self.modifier -= BLIZZARD_MOD_STEP

    def on_no_drop(self):
        """Called when no potion drops."""
        self.modifier += BLIZZARD_MOD_STEP


@dataclass
class RewardState:
    """
    Complete state for reward generation across a run.

    Tracks:
    - Card rarity pity timer
    - Potion drop pity timer
    - Owned relics (to prevent duplicates)
    - Seen cards (optional, for some mechanics)
    """
    card_blizzard: CardBlizzardState = field(default_factory=CardBlizzardState)
    potion_blizzard: PotionBlizzardState = field(default_factory=PotionBlizzardState)
    owned_relics: Set[str] = field(default_factory=set)

    def add_relic(self, relic_id: str):
        """Record a relic as owned."""
        self.owned_relics.add(relic_id)

    def has_relic(self, relic_id: str) -> bool:
        """Check if a relic is owned."""
        return relic_id in self.owned_relics


@dataclass
class ShopInventory:
    """Complete shop inventory."""
    colored_cards: List[Tuple[Card, int]]       # (card, price)
    colorless_cards: List[Tuple[Card, int]]     # (card, price)
    relics: List[Tuple[Relic, int]]             # (relic, price)
    potions: List[Tuple[Potion, int]]           # (potion, price)
    purge_cost: int                             # Card removal cost
    purge_available: bool = True


# ============================================================================
# CARD REWARD GENERATION
# ============================================================================

def _get_card_rarity_thresholds(room_type: str) -> Dict[str, int]:
    """Get the base rare/uncommon thresholds for a room type."""
    return CARD_RARITY_THRESHOLDS.get(room_type, CARD_RARITY_THRESHOLDS["normal"])


def _roll_card_rarity(
    rng: Random,
    card_blizzard: CardBlizzardState,
    room_type: str = "normal",
    has_nloth_gift: bool = False,
) -> CardRarity:
    """
    Roll for card rarity using the game's algorithm.

    Args:
        rng: The card RNG stream
        card_blizzard: Current pity timer state
        room_type: "normal", "elite", or "shop"
        has_nloth_gift: Triple rare chance relic

    Returns:
        CardRarity for the rolled card
    """
    thresholds = _get_card_rarity_thresholds(room_type)
    rare_threshold = thresholds["rare"]
    uncommon_threshold = thresholds["uncommon"]

    # N'loth's Gift triples rare chance
    if has_nloth_gift:
        rare_threshold *= 3

    # Roll 0-99 and add blizzard offset
    roll = rng.random(99)
    roll += card_blizzard.offset

    if roll < rare_threshold:
        card_blizzard.on_rare()
        return CardRarity.RARE
    elif roll < rare_threshold + uncommon_threshold:
        card_blizzard.on_uncommon()
        return CardRarity.UNCOMMON
    else:
        card_blizzard.on_common()
        return CardRarity.COMMON


def _get_card_pool(
    player_class: str,
    rarity: CardRarity,
    has_prismatic_shard: bool = False,
) -> List[Card]:
    """
    Get the card pool for a given rarity in Java HashMap iteration order.

    The pool order is determined by Java's CardLibrary.cards HashMap iteration
    order, not alphabetical order. This is critical for seed-deterministic
    card rewards.

    Args:
        player_class: "WATCHER", "IRONCLAD", "SILENT", "DEFECT"
        rarity: The rarity to filter by
        has_prismatic_shard: Can get cards from any class

    Returns:
        List of cards matching the rarity, in HashMap iteration order
    """
    rarity_str = rarity.name  # "COMMON", "UNCOMMON", "RARE"

    # Import the card library order module (handles its own imports)
    _card_lib_module = _load_module(
        "card_library_order",
        os.path.join(_core_dir, "utils", "card_library_order.py")
    )

    if has_prismatic_shard:
        # All cards from all classes
        cards_dict = {
            **IRONCLAD_CARDS,
            **SILENT_CARDS,
            **DEFECT_CARDS,
            **WATCHER_CARDS,
            **COLORLESS_CARDS,
        }
        # Combine all pool orders for prismatic shard
        pool_order = (
            _card_lib_module.get_ironclad_pool_by_rarity(rarity_str) +
            _card_lib_module.get_silent_pool_by_rarity(rarity_str) +
            _card_lib_module.get_defect_pool_by_rarity(rarity_str) +
            _card_lib_module.get_watcher_pool_by_rarity(rarity_str)
        )
    elif player_class == "IRONCLAD":
        cards_dict = IRONCLAD_CARDS
        pool_order = _card_lib_module.get_ironclad_pool_by_rarity(rarity_str)
    elif player_class == "SILENT":
        cards_dict = SILENT_CARDS
        pool_order = _card_lib_module.get_silent_pool_by_rarity(rarity_str)
    elif player_class == "DEFECT":
        cards_dict = DEFECT_CARDS
        pool_order = _card_lib_module.get_defect_pool_by_rarity(rarity_str)
    else:
        # Default to Watcher
        cards_dict = WATCHER_CARDS
        pool_order = _card_lib_module.get_watcher_pool_by_rarity(rarity_str)

    # Build pool in HashMap iteration order
    pool = []
    for card_id in pool_order:
        if card_id in cards_dict and cards_dict[card_id].rarity == rarity:
            pool.append(cards_dict[card_id])

    return pool


def _get_upgrade_chance(act: int, ascension: int) -> float:
    """
    Get the card upgrade chance for a given act and ascension.

    IMPORTANT: This returns the chance, but the RNG call happens
    for ALL non-rare cards regardless of chance value.
    """
    if act == 1:
        return 0.0

    chances = CARD_UPGRADE_CHANCES.get(act, {"default": 0, "a12": 0})
    if isinstance(chances, dict):
        return chances["a12"] if ascension >= 12 else chances["default"]
    return float(chances)


def _check_upgrade(rng: Random, card: Card, upgrade_chance: float) -> bool:
    """
    Check if a card should be upgraded.

    CRITICAL: For non-rare cards, randomBoolean is ALWAYS called
    (consuming RNG) even if upgrade_chance is 0 or card can't upgrade.
    This matches the Java short-circuit evaluation order:
        if (c.rarity != RARE && cardRng.randomBoolean(chance) && c.canUpgrade())

    Args:
        rng: The card RNG stream
        card: The card to potentially upgrade
        upgrade_chance: Probability of upgrade (0.0 in Act 1)

    Returns:
        True if card should be upgraded, False otherwise
    """
    # Rare cards skip the randomBoolean call entirely (Java short-circuit)
    if card.rarity == CardRarity.RARE:
        return False

    # For non-rare cards, ALWAYS consume RNG (even if chance is 0)
    roll_result = rng.random_boolean(upgrade_chance)

    # Only upgrade if roll succeeds AND card can actually upgrade
    if roll_result and card.can_upgrade():
        return True

    return False


def generate_card_rewards(
    rng: Random,
    reward_state: RewardState,
    act: int = 1,
    player_class: str = "WATCHER",
    ascension: int = 0,
    room_type: str = "normal",
    num_cards: int = 3,
    has_prismatic_shard: bool = False,
    has_busted_crown: bool = False,
    has_question_card: bool = False,
    has_nloth_gift: bool = False,
) -> List[Card]:
    """
    Generate card rewards for a combat victory.

    Implements the full card reward algorithm from AbstractDungeon.getRewardCards():
    1. Determine number of cards (base 3, modified by relics)
    2. For each card, roll rarity with pity timer
    3. Get a card of that rarity (no duplicates in same reward)
    4. Possibly upgrade non-rare cards based on act/ascension

    Args:
        rng: The card RNG stream (cardRng)
        reward_state: State tracking pity timers
        act: Current act (1-4)
        player_class: Player's class
        ascension: Ascension level (0-20)
        room_type: "normal", "elite", or "shop"
        num_cards: Base number of cards (default 3)
        has_prismatic_shard: Relic that allows any class cards
        has_busted_crown: -2 card choices
        has_question_card: +1 card choice
        has_nloth_gift: Triple rare chance

    Returns:
        List of Card objects for the reward
    """
    # Apply relic modifiers to card count
    if has_busted_crown:
        num_cards -= 2
    if has_question_card:
        num_cards += 1

    num_cards = max(1, num_cards)  # At least 1 card

    cards: List[Card] = []
    card_ids_in_reward: Set[str] = set()

    # Phase 1: Select all cards (rarity roll + index roll for each)
    for _ in range(num_cards):
        # Roll rarity
        rarity = _roll_card_rarity(
            rng, reward_state.card_blizzard, room_type, has_nloth_gift
        )

        # Get card pool for this rarity
        pool = _get_card_pool(player_class, rarity, has_prismatic_shard)

        if not pool:
            continue

        # Pick a card (no duplicates within this reward - reroll on duplicate)
        attempts = 0
        max_attempts = 100
        while attempts < max_attempts:
            idx = rng.random(len(pool) - 1)
            card = pool[idx]
            if card.id not in card_ids_in_reward:
                break
            attempts += 1

        # Create a copy
        card_copy = card.copy()
        card_ids_in_reward.add(card_copy.id)
        cards.append(card_copy)

    # Phase 2: Upgrade checks AFTER all cards selected
    # This matches Java's separate loop for upgrade checks
    upgrade_chance = _get_upgrade_chance(act, ascension)
    for card in cards:
        if _check_upgrade(rng, card, upgrade_chance):
            card.upgrade()

    return cards


def generate_colorless_card_rewards(
    rng: Random,
    num_cards: int = 3,
) -> List[Card]:
    """
    Generate colorless card rewards (used in some events/shops).

    Colorless cards are rolled as UNCOMMON or RARE only.
    """
    cards: List[Card] = []
    card_ids: Set[str] = set()

    uncommon_pool = [c for c in COLORLESS_CARDS.values()
                    if c.rarity == CardRarity.UNCOMMON]
    rare_pool = [c for c in COLORLESS_CARDS.values()
                if c.rarity == CardRarity.RARE]

    for _ in range(num_cards):
        # 70% uncommon, 30% rare for colorless
        if rng.random(99) < 70:
            pool = uncommon_pool
        else:
            pool = rare_pool

        if not pool:
            continue

        attempts = 0
        while attempts < 100:
            idx = rng.random(len(pool) - 1)
            card = pool[idx]
            if card.id not in card_ids:
                break
            attempts += 1

        card_copy = card.copy()
        card_ids.add(card_copy.id)
        cards.append(card_copy)

    return cards


# ============================================================================
# RELIC REWARD GENERATION
# ============================================================================

def _get_relic_pool(
    tier: RelicTier,
    player_class: str,
    owned_relics: Set[str],
    act: int = 1,
) -> List[str]:
    """
    Get available relics for a tier, excluding owned ones.

    Also handles class restrictions and act restrictions (e.g., Ectoplasm Act 1 only).
    """
    if tier == RelicTier.COMMON:
        pool = COMMON_RELICS.copy()
    elif tier == RelicTier.UNCOMMON:
        pool = UNCOMMON_RELICS.copy()
    elif tier == RelicTier.RARE:
        pool = RARE_RELICS.copy()
    elif tier == RelicTier.BOSS:
        pool = BOSS_RELICS.copy()
    elif tier == RelicTier.SHOP:
        pool = SHOP_RELICS.copy()
    else:
        pool = []

    # Filter out owned relics
    pool = [r for r in pool if r not in owned_relics]

    # Filter by class restrictions
    filtered = []
    for relic_id in pool:
        relic = ALL_RELICS.get(relic_id)
        if relic is None:
            continue

        # Check class restriction
        if relic.player_class != PlayerClass.ALL:
            if relic.player_class.name != player_class:
                continue

        # Check act restriction (e.g., Ectoplasm only in Act 1)
        if relic.act_restriction is not None:
            if act > relic.act_restriction:
                continue

        # Check requires_relic (boss upgrades)
        if relic.requires_relic is not None:
            if relic.requires_relic not in owned_relics:
                continue

        filtered.append(relic_id)

    return filtered


def _roll_elite_relic_tier(rng: Random) -> RelicTier:
    """
    Roll relic tier for elite rewards.

    From MonsterRoomElite.returnRandomRelicTier():
    roll 0-99: <50 = COMMON, >82 = RARE, else UNCOMMON
    """
    roll = rng.random(99)
    if roll < ELITE_RELIC_THRESHOLDS["common"]:
        return RelicTier.COMMON
    if roll > ELITE_RELIC_THRESHOLDS["rare"]:
        return RelicTier.RARE
    return RelicTier.UNCOMMON


def _roll_normal_relic_tier(rng: Random) -> RelicTier:
    """
    Roll relic tier for normal monster rewards (rare drop).

    From MonsterRoom.returnRandomRelicTier():
    roll 0-99: <50 = COMMON, >85 = RARE, else UNCOMMON
    """
    roll = rng.random(99)
    if roll < NORMAL_RELIC_THRESHOLDS["common"]:
        return RelicTier.COMMON
    if roll > NORMAL_RELIC_THRESHOLDS["rare"]:
        return RelicTier.RARE
    return RelicTier.UNCOMMON


def _roll_shop_relic_tier(rng: Random) -> RelicTier:
    """
    Roll relic tier for shop relics.

    From ShopScreen.rollRelicTier():
    roll 0-99: <48 = COMMON, <82 = UNCOMMON, else RARE
    """
    roll = rng.random(99)
    if roll < SHOP_RELIC_THRESHOLDS["common"]:
        return RelicTier.COMMON
    if roll < SHOP_RELIC_THRESHOLDS["uncommon"]:
        return RelicTier.UNCOMMON
    return RelicTier.RARE


def _roll_random_relic_tier(rng: Random, act: int = 1) -> RelicTier:
    """
    Roll a random relic tier for events.

    From AbstractDungeon.returnRandomRelicTier():
    Acts 1-3: 50% common, 33% uncommon, 17% rare
    Act 4: 0% common, 100% uncommon
    """
    chances = RANDOM_RELIC_CHANCES["act4"] if act == 4 else RANDOM_RELIC_CHANCES["default"]

    roll = rng.random(99)
    if roll < chances["common"]:
        return RelicTier.COMMON
    if roll < chances["common"] + chances["uncommon"]:
        return RelicTier.UNCOMMON
    return RelicTier.RARE


def generate_relic_reward(
    rng: Random,
    tier: RelicTier,
    reward_state: RewardState,
    player_class: str = "WATCHER",
    act: int = 1,
) -> Optional[Relic]:
    """
    Generate a single relic reward of the specified tier.

    Falls back to higher tiers if the pool is exhausted.

    Args:
        rng: The relic RNG stream (relicRng)
        tier: Desired relic tier
        reward_state: State with owned relics
        player_class: Player's class
        act: Current act

    Returns:
        A Relic object, or None if all relics exhausted
    """
    original_tier = tier

    # Try to get a relic, falling back to higher tiers if needed
    while True:
        pool = _get_relic_pool(tier, player_class, reward_state.owned_relics, act)

        if pool:
            idx = rng.random(len(pool) - 1)
            relic_id = pool[idx]
            relic = get_relic(relic_id)
            reward_state.add_relic(relic_id)
            return relic

        # Fall back to higher tier
        if tier == RelicTier.COMMON:
            tier = RelicTier.UNCOMMON
        elif tier == RelicTier.UNCOMMON:
            tier = RelicTier.RARE
        else:
            # All pools exhausted - return Circlet
            return get_relic("Circlet")


def generate_elite_relic_reward(
    rng: Random,
    reward_state: RewardState,
    player_class: str = "WATCHER",
    act: int = 1,
) -> Relic:
    """Generate a relic reward from an elite fight."""
    tier = _roll_elite_relic_tier(rng)
    return generate_relic_reward(rng, tier, reward_state, player_class, act)


def generate_boss_relics(
    rng: Random,
    reward_state: RewardState,
    player_class: str = "WATCHER",
    act: int = 1,
    num_choices: int = 3,
) -> List[Relic]:
    """
    Generate boss relic choices (typically 3).

    Boss chest always offers BOSS tier relics.

    Args:
        rng: The relic RNG stream
        reward_state: State with owned relics
        player_class: Player's class
        act: Current act
        num_choices: Number of relics to offer (default 3)

    Returns:
        List of Relic objects to choose from
    """
    relics: List[Relic] = []
    temp_owned = reward_state.owned_relics.copy()

    for _ in range(num_choices):
        pool = _get_relic_pool(RelicTier.BOSS, player_class, temp_owned, act)

        if not pool:
            # Fallback to Circlet if boss pool exhausted
            relics.append(get_relic("Circlet"))
            continue

        idx = rng.random(len(pool) - 1)
        relic_id = pool[idx]
        relic = get_relic(relic_id)
        relics.append(relic)
        temp_owned.add(relic_id)

    return relics


# ============================================================================
# POTION DROP GENERATION
# ============================================================================

def _get_potion_pool(player_class: str) -> List[Potion]:
    """Get the potion pool for a player class."""
    # Map string class to enum
    class_map = {
        "WATCHER": PotionPlayerClass.WATCHER,
        "IRONCLAD": PotionPlayerClass.IRONCLAD,
        "SILENT": PotionPlayerClass.SILENT,
        "DEFECT": PotionPlayerClass.DEFECT,
    }
    pc = class_map.get(player_class, PotionPlayerClass.WATCHER)
    return get_potion_pool(pc)


def check_potion_drop(
    rng: Random,
    reward_state: RewardState,
    room_type: str = "normal",
    has_white_beast_statue: bool = False,
    has_sozu: bool = False,
    current_rewards: int = 0,
) -> Tuple[bool, Optional[Potion]]:
    """
    Check if a potion drops and generate it.

    Uses the blizzard modifier system:
    - Base 40% chance in monster/elite/event rooms
    - +10% for each combat without a drop
    - -10% for each drop

    Args:
        rng: The potion RNG stream (potionRng)
        reward_state: State with potion blizzard modifier
        room_type: "normal", "elite", or "event"
        has_white_beast_statue: 100% drop rate relic
        has_sozu: Prevents potion drops
        current_rewards: Number of rewards already (max 4)

    Returns:
        Tuple of (did_drop, potion or None)
    """
    if has_sozu:
        return (False, None)

    if current_rewards >= 4:
        return (False, None)

    # Calculate drop chance
    if has_white_beast_statue:
        chance = 100
    elif room_type in ("normal", "elite", "event"):
        chance = BASE_POTION_DROP_CHANCE + reward_state.potion_blizzard.modifier
        chance = max(0, min(100, chance))
    else:
        return (False, None)

    # Roll for drop
    roll = rng.random(99)

    if roll < chance:
        reward_state.potion_blizzard.on_drop()
        potion = _roll_potion(rng, "WATCHER")
        return (True, potion)
    else:
        reward_state.potion_blizzard.on_no_drop()
        return (False, None)


def _roll_potion(rng: Random, player_class: str) -> Potion:
    """
    Roll a random potion using the rarity distribution.

    From AbstractDungeon.returnRandomPotion():
    65% common, 25% uncommon, 10% rare
    """
    roll = rng.random(99)

    if roll < POTION_COMMON_CHANCE:
        rarity = PotionRarity.COMMON
    elif roll < POTION_COMMON_CHANCE + POTION_UNCOMMON_CHANCE:
        rarity = PotionRarity.UNCOMMON
    else:
        rarity = PotionRarity.RARE

    return _get_potion_of_rarity(rng, rarity, player_class)


def _get_potion_of_rarity(
    rng: Random,
    rarity: PotionRarity,
    player_class: str,
) -> Potion:
    """Get a random potion of the specified rarity."""
    pool = _get_potion_pool(player_class)
    rarity_pool = [p for p in pool if p.rarity == rarity]

    if not rarity_pool:
        # Fallback to any potion of correct rarity
        if rarity == PotionRarity.COMMON:
            rarity_pool = COMMON_POTIONS
        elif rarity == PotionRarity.UNCOMMON:
            rarity_pool = UNCOMMON_POTIONS
        else:
            rarity_pool = RARE_POTIONS

    idx = rng.random(len(rarity_pool) - 1)
    return rarity_pool[idx]


def generate_potion_reward(
    rng: Random,
    player_class: str = "WATCHER",
) -> Potion:
    """Generate a single potion reward (for shops, events, etc)."""
    return _roll_potion(rng, player_class)


# ============================================================================
# GOLD REWARD GENERATION
# ============================================================================

def generate_gold_reward(
    rng: Random,
    room_type: str,
    ascension: int = 0,
    has_golden_idol: bool = False,
) -> int:
    """
    Generate gold reward for a combat.

    From AbstractRoom.dropReward():
    - Boss: 100 +/- random(-5,5), 75% at A13+
    - Elite: random(25, 35), fixed 30 at A13+
    - Normal: random(10, 20), fixed 15 at A13+

    Args:
        rng: The treasure RNG stream (treasureRng)
        room_type: "boss", "elite", or "normal"
        ascension: Ascension level
        has_golden_idol: 25% more gold relic

    Returns:
        Gold amount
    """
    gold = 0

    if room_type == "boss":
        base = GOLD_REWARDS["boss"]["base"]
        variance = GOLD_REWARDS["boss"]["variance"]
        gold = base + rng.random_range(-variance, variance)
        if ascension >= 13:
            gold = int(gold * GOLD_REWARDS["boss"]["a13_multiplier"])

    elif room_type == "elite":
        if ascension >= 13:
            gold = GOLD_REWARDS["elite"]["a13_fixed"]
        else:
            gold = rng.random_range(
                GOLD_REWARDS["elite"]["min"],
                GOLD_REWARDS["elite"]["max"]
            )

    elif room_type == "normal":
        if ascension >= 13:
            gold = GOLD_REWARDS["normal"]["a13_fixed"]
        else:
            gold = rng.random_range(
                GOLD_REWARDS["normal"]["min"],
                GOLD_REWARDS["normal"]["max"]
            )

    # Golden Idol: 25% more gold
    if has_golden_idol:
        gold = int(gold * 1.25)

    return gold


# ============================================================================
# SHOP GENERATION
# ============================================================================

# Shop card prices by rarity
SHOP_CARD_PRICES = {
    CardRarity.COMMON: {"min": 45, "max": 55},
    CardRarity.UNCOMMON: {"min": 68, "max": 82},
    CardRarity.RARE: {"min": 135, "max": 165},
}

# Colorless card prices (higher than colored)
SHOP_COLORLESS_PRICES = {
    CardRarity.UNCOMMON: {"min": 81, "max": 99},
    CardRarity.RARE: {"min": 162, "max": 198},
}

# Relic prices by tier
SHOP_RELIC_PRICES = {
    RelicTier.COMMON: {"min": 143, "max": 157},
    RelicTier.UNCOMMON: {"min": 238, "max": 262},
    RelicTier.RARE: {"min": 285, "max": 315},
    RelicTier.SHOP: {"min": 143, "max": 157},  # Same as common
}

# Potion prices by rarity
SHOP_POTION_PRICES = {
    PotionRarity.COMMON: {"min": 48, "max": 52},
    PotionRarity.UNCOMMON: {"min": 72, "max": 78},
    PotionRarity.RARE: {"min": 95, "max": 105},
}

# Base card removal cost
BASE_PURGE_COST = 75
PURGE_COST_INCREMENT = 25  # +25 per removal


def _roll_price(rng: Random, min_price: int, max_price: int) -> int:
    """Roll a price with small variance (0.95-1.05)."""
    base = (min_price + max_price) // 2
    variance = rng.random_float_range(0.95, 1.05)
    return int(base * variance)


def generate_shop_inventory(
    rng: Random,
    reward_state: RewardState,
    act: int = 1,
    player_class: str = "WATCHER",
    ascension: int = 0,
    purge_count: int = 0,
    has_membership_card: bool = False,
    has_the_courier: bool = False,
) -> ShopInventory:
    """
    Generate a complete shop inventory.

    Shop contents from ShopScreen:
    - 5 colored cards (various rarities)
    - 2 colorless cards (uncommon/rare)
    - 3 relics (2 random tier + 1 SHOP tier)
    - 3 potions (random rarities)
    - Card removal option

    Args:
        rng: The merchant RNG stream (merchantRng)
        reward_state: State with owned relics
        act: Current act
        player_class: Player's class
        ascension: Ascension level
        purge_count: Number of previous card removals
        has_membership_card: 50% discount relic
        has_the_courier: 20% discount + always has removal

    Returns:
        ShopInventory with all items and prices
    """
    discount = 1.0
    # A16+: Shop prices increased by 10% (Java: ShopScreen.java:212)
    if ascension >= 16:
        discount *= 1.1
    if has_membership_card:
        discount *= 0.5
    if has_the_courier:
        discount *= 0.8

    # Generate colored cards (5 cards: mix of rarities)
    colored_cards: List[Tuple[Card, int]] = []
    # Shop typically has: 2 common, 2 uncommon, 1 rare (attack/skill mix)
    shop_rarities = [
        CardRarity.COMMON, CardRarity.COMMON,
        CardRarity.UNCOMMON, CardRarity.UNCOMMON,
        CardRarity.RARE
    ]

    card_ids_used: Set[str] = set()
    for rarity in shop_rarities:
        pool = _get_card_pool(player_class, rarity)
        pool = [c for c in pool if c.id not in card_ids_used]

        if pool:
            idx = rng.random(len(pool) - 1)
            card = pool[idx].copy()
            card_ids_used.add(card.id)

            price_range = SHOP_CARD_PRICES.get(rarity, SHOP_CARD_PRICES[CardRarity.COMMON])
            price = _roll_price(rng, price_range["min"], price_range["max"])
            price = int(price * discount)

            colored_cards.append((card, price))

    # Generate colorless cards (2 cards: uncommon or rare)
    colorless_cards: List[Tuple[Card, int]] = []
    for _ in range(2):
        # 70% uncommon, 30% rare
        if rng.random(99) < 70:
            rarity = CardRarity.UNCOMMON
        else:
            rarity = CardRarity.RARE

        pool = [c for c in COLORLESS_CARDS.values()
               if c.rarity == rarity and c.id not in card_ids_used]

        if pool:
            idx = rng.random(len(pool) - 1)
            card = pool[idx].copy()
            card_ids_used.add(card.id)

            price_range = SHOP_COLORLESS_PRICES.get(rarity, SHOP_COLORLESS_PRICES[CardRarity.UNCOMMON])
            price = _roll_price(rng, price_range["min"], price_range["max"])
            price = int(price * discount)

            colorless_cards.append((card, price))

    # Generate relics (3 relics: 2 random tier + 1 SHOP)
    relics: List[Tuple[Relic, int]] = []
    temp_owned = reward_state.owned_relics.copy()

    for i in range(3):
        if i == 2:
            tier = RelicTier.SHOP
        else:
            tier = _roll_shop_relic_tier(rng)

        pool = _get_relic_pool(tier, player_class, temp_owned, act)

        if pool:
            idx = rng.random(len(pool) - 1)
            relic_id = pool[idx]
            relic = get_relic(relic_id)
            temp_owned.add(relic_id)

            price_range = SHOP_RELIC_PRICES.get(tier, SHOP_RELIC_PRICES[RelicTier.COMMON])
            price = _roll_price(rng, price_range["min"], price_range["max"])
            price = int(price * discount)

            relics.append((relic, price))

    # Generate potions (3 potions)
    potions: List[Tuple[Potion, int]] = []
    for _ in range(3):
        potion = _roll_potion(rng, player_class)

        price_range = SHOP_POTION_PRICES.get(potion.rarity, SHOP_POTION_PRICES[PotionRarity.COMMON])
        price = _roll_price(rng, price_range["min"], price_range["max"])
        price = int(price * discount)

        potions.append((potion, price))

    # Calculate purge cost
    purge_cost = BASE_PURGE_COST + (purge_count * PURGE_COST_INCREMENT)
    purge_cost = int(purge_cost * discount)

    return ShopInventory(
        colored_cards=colored_cards,
        colorless_cards=colorless_cards,
        relics=relics,
        potions=potions,
        purge_cost=purge_cost,
        purge_available=True,
    )


# ============================================================================
# TESTING
# ============================================================================

if __name__ == "__main__":
    # seed_to_long is already imported at the top

    print("=== Reward Generation Tests ===\n")

    # Test with a known seed
    seed_str = "TESTSEED123"
    seed = seed_to_long(seed_str)
    print(f"Testing with seed: {seed_str} ({seed})\n")

    # Initialize RNG streams
    card_rng = Random(seed)
    relic_rng = Random(seed)
    potion_rng = Random(seed)
    treasure_rng = Random(seed)
    merchant_rng = Random(seed)

    # Initialize reward state
    state = RewardState()
    state.add_relic("PureWater")  # Starting relic

    print("--- Card Rewards (Normal Room, Act 1) ---")
    cards = generate_card_rewards(
        card_rng, state, act=1, player_class="WATCHER",
        room_type="normal"
    )
    for card in cards:
        upgraded = "+" if card.upgraded else ""
        print(f"  {card.name}{upgraded} ({card.rarity.name})")

    print(f"\nCard blizzard offset: {state.card_blizzard.offset}")

    print("\n--- Card Rewards (Elite Room, Act 2, A12) ---")
    cards = generate_card_rewards(
        card_rng, state, act=2, player_class="WATCHER",
        room_type="elite", ascension=12
    )
    for card in cards:
        upgraded = "+" if card.upgraded else ""
        print(f"  {card.name}{upgraded} ({card.rarity.name})")

    print("\n--- Elite Relic Reward ---")
    relic = generate_elite_relic_reward(relic_rng, state, "WATCHER", act=1)
    print(f"  {relic.name} ({relic.tier.name})")

    print("\n--- Boss Relic Choices ---")
    boss_relics = generate_boss_relics(relic_rng, state, "WATCHER", act=1)
    for relic in boss_relics:
        print(f"  {relic.name}")

    print("\n--- Potion Drop Check (3 combats) ---")
    for i in range(3):
        dropped, potion = check_potion_drop(potion_rng, state, "normal")
        if dropped:
            print(f"  Combat {i+1}: DROPPED - {potion.name} ({potion.rarity.name})")
        else:
            print(f"  Combat {i+1}: No drop (mod: {state.potion_blizzard.modifier})")

    print("\n--- Gold Rewards ---")
    for room_type in ["normal", "elite", "boss"]:
        gold = generate_gold_reward(treasure_rng, room_type, ascension=0)
        print(f"  {room_type.capitalize()}: {gold} gold")

    print("\n--- Shop Inventory ---")
    shop = generate_shop_inventory(
        merchant_rng, state, act=1, player_class="WATCHER"
    )
    print("  Colored Cards:")
    for card, price in shop.colored_cards:
        print(f"    {card.name} ({card.rarity.name}): {price}g")
    print("  Colorless Cards:")
    for card, price in shop.colorless_cards:
        print(f"    {card.name} ({card.rarity.name}): {price}g")
    print("  Relics:")
    for relic, price in shop.relics:
        print(f"    {relic.name} ({relic.tier.name}): {price}g")
    print("  Potions:")
    for potion, price in shop.potions:
        print(f"    {potion.name} ({potion.rarity.name}): {price}g")
    print(f"  Card Removal: {shop.purge_cost}g")
