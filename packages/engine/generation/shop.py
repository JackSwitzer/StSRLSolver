"""
Slay the Spire - Shop Inventory Prediction

Implements accurate shop inventory prediction matching the game's ShopScreen.java.

Shop Structure (from ShopScreen.init()):
- 5 colored cards (2 attack, 2 skill, 1 power)
- 2 colorless cards (1 uncommon, 1 rare)
- 3 relics (2 random tier, 1 shop tier)
- 3 potions
- Card removal service

RNG Streams Used:
- cardRng: Card selection (12-20+ calls due to retry logic)
- merchantRng: Prices, relic tiers, sale card selection (~16 calls)
- potionRng: Potion selection (6-15+ calls)

Card Selection Algorithm (ShopScreen.initCards()):
1. For each of 5 colored card slots:
   - Roll rarity (COMMON 60%, UNCOMMON 37%, RARE 3%)
   - Get card from pool matching rarity AND type
   - If card is colorless OR duplicate of same-type card already in shop, reroll
2. For each of 2 colorless card slots:
   - First slot: UNCOMMON colorless
   - Second slot: RARE colorless
3. Apply sale to one random colored card (25% discount)

Price Calculation (ShopScreen.setPrice()):
- Colored cards: base_price * 0.9-1.1 random variance
- Colorless cards: 1.2x colored price (same variance)
- Relics: tier-based price with variance
- Potions: rarity-based price with variance
- Membership Card: 50% discount
- The Courier: 20% discount (multiplicative)
"""

import os
import importlib.util
from dataclasses import dataclass, field
from typing import List, Dict, Set, Tuple, Optional, Any
from enum import Enum

# Load modules directly to avoid __init__.py circular imports
_core_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def _load_module(name: str, filepath: str):
    """Load a module directly from file path."""
    spec = importlib.util.spec_from_file_location(name, filepath)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


_rng_module = _load_module("rng", os.path.join(_core_dir, "state", "rng.py"))
Random = _rng_module.Random
seed_to_long = _rng_module.seed_to_long

_cards_module = _load_module("cards", os.path.join(_core_dir, "content", "cards.py"))
Card = _cards_module.Card
CardRarity = _cards_module.CardRarity
CardColor = _cards_module.CardColor
CardType = _cards_module.CardType
WATCHER_CARDS = _cards_module.WATCHER_CARDS
COLORLESS_CARDS = _cards_module.COLORLESS_CARDS
get_card = _cards_module.get_card

_card_lib_module = _load_module(
    "card_library_order",
    os.path.join(_core_dir, "utils", "card_library_order.py")
)
get_watcher_pool_by_rarity = _card_lib_module.get_watcher_pool_by_rarity
WATCHER_CARD_RARITIES = _card_lib_module.WATCHER_CARD_RARITIES

_relics_module = _load_module("relics", os.path.join(_core_dir, "content", "relics.py"))
Relic = _relics_module.Relic
RelicTier = _relics_module.RelicTier
PlayerClass = _relics_module.PlayerClass
ALL_RELICS = _relics_module.ALL_RELICS
get_relic = _relics_module.get_relic

_potions_module = _load_module("potions", os.path.join(_core_dir, "content", "potions.py"))
Potion = _potions_module.Potion
PotionRarity = _potions_module.PotionRarity
PotionPlayerClass = _potions_module.PlayerClass
ALL_POTIONS = _potions_module.ALL_POTIONS
get_potion_pool = _potions_module.get_potion_pool


# ============================================================================
# CONSTANTS - From ShopScreen.java
# ============================================================================

# Shop card type order: 2 attacks, 2 skills, 1 power
SHOP_CARD_TYPES = [
    CardType.ATTACK,
    CardType.ATTACK,
    CardType.SKILL,
    CardType.SKILL,
    CardType.POWER,
]

# Card rarity thresholds for shop (from AbstractDungeon.rollRarity())
# roll < 3 = RARE, roll < 3+37 = UNCOMMON, else COMMON
SHOP_RARITY_THRESHOLDS = {
    "rare": 3,
    "uncommon": 37,
    # common = 60 (implicit: 100 - 3 - 37)
}

# Base prices by card rarity (from ShopScreen)
CARD_BASE_PRICES = {
    CardRarity.COMMON: 50,
    CardRarity.UNCOMMON: 75,
    CardRarity.RARE: 150,
}

# Colorless card price multiplier (1.2x colored price)
COLORLESS_PRICE_MULTIPLIER = 1.2

# Relic prices by tier (from ShopScreen.getPrice(AbstractRelic))
RELIC_BASE_PRICES = {
    RelicTier.COMMON: 150,
    RelicTier.UNCOMMON: 250,
    RelicTier.RARE: 300,
    RelicTier.SHOP: 150,  # Same as common
}

# Potion prices by rarity (from ShopScreen.getPrice(AbstractPotion))
POTION_BASE_PRICES = {
    PotionRarity.COMMON: 50,
    PotionRarity.UNCOMMON: 75,
    PotionRarity.RARE: 100,
}

# Shop relic tier roll thresholds (from ShopScreen.rollRelicTier())
# roll < 48 = COMMON, roll < 82 = UNCOMMON, else RARE
SHOP_RELIC_THRESHOLDS = {"common": 48, "uncommon": 82}

# Card removal base cost and increment
BASE_PURGE_COST = 75
PURGE_COST_INCREMENT = 25

# Sale discount for one random colored card
SALE_DISCOUNT = 0.5  # 50% discount (sale price = original * 0.5)


# ============================================================================
# DATA CLASSES
# ============================================================================

@dataclass
class ShopCard:
    """A card in the shop with its price."""
    card: Card
    price: int
    on_sale: bool = False


@dataclass
class ShopRelic:
    """A relic in the shop with its price."""
    relic: Relic
    price: int


@dataclass
class ShopPotion:
    """A potion in the shop with its price."""
    potion: Potion
    price: int


@dataclass
class PredictedShopInventory:
    """Complete predicted shop inventory."""
    colored_cards: List[ShopCard]
    colorless_cards: List[ShopCard]
    relics: List[ShopRelic]
    potions: List[ShopPotion]
    purge_cost: int
    sale_card_index: int  # Index in colored_cards that's on sale

    # RNG consumption tracking
    card_rng_calls: int = 0
    merchant_rng_calls: int = 0
    potion_rng_calls: int = 0


@dataclass
class ShopPredictionResult:
    """Result of shop prediction including final counter values."""
    inventory: PredictedShopInventory
    final_card_counter: int
    final_merchant_counter: int
    final_potion_counter: int


# ============================================================================
# CARD POOL HELPERS
# ============================================================================

def _get_card_pool_by_type_and_rarity(
    card_type: CardType,
    rarity: CardRarity,
    player_class: str = "WATCHER",
) -> List[str]:
    """
    Get card pool filtered by type and rarity in HashMap iteration order.

    This matches the game's CardGroup.getCardByType() filtering.

    Args:
        card_type: ATTACK, SKILL, or POWER
        rarity: COMMON, UNCOMMON, or RARE
        player_class: Player class name

    Returns:
        List of card IDs matching both type and rarity
    """
    # Get rarity pool in HashMap order
    rarity_pool = get_watcher_pool_by_rarity(rarity.name)

    # Filter by card type
    result = []
    for card_id in rarity_pool:
        if card_id in WATCHER_CARDS:
            card = WATCHER_CARDS[card_id]
            if card.card_type == card_type:
                result.append(card_id)

    return result


def _roll_shop_rarity(rng: Random) -> CardRarity:
    """
    Roll card rarity for shop using the game's algorithm.

    From AbstractDungeon.rollRarity():
    - roll 0-99
    - < 3 = RARE
    - < 40 (3 + 37) = UNCOMMON
    - else COMMON

    Args:
        rng: The cardRng stream

    Returns:
        CardRarity
    """
    roll = rng.random(99)

    if roll < SHOP_RARITY_THRESHOLDS["rare"]:
        return CardRarity.RARE
    elif roll < SHOP_RARITY_THRESHOLDS["rare"] + SHOP_RARITY_THRESHOLDS["uncommon"]:
        return CardRarity.UNCOMMON
    else:
        return CardRarity.COMMON


def _is_colorless_card(card_id: str) -> bool:
    """Check if a card is colorless."""
    if card_id in WATCHER_CARDS:
        return WATCHER_CARDS[card_id].color == CardColor.COLORLESS
    if card_id in COLORLESS_CARDS:
        return True
    return False


# ============================================================================
# COLORED CARD GENERATION
# ============================================================================

def _generate_shop_colored_cards(
    card_rng: Random,
    player_class: str = "WATCHER",
) -> Tuple[List[str], int]:
    """
    Generate the 5 colored cards for the shop.

    Algorithm from ShopScreen.initCards():
    1. For each card type slot (ATTACK, ATTACK, SKILL, SKILL, POWER):
       a. Roll rarity
       b. Get card from pool matching type+rarity
       c. If card is colorless OR is same ID as previous card of same type, reroll
       d. Track cards per type to avoid same-type duplicates

    Args:
        card_rng: The card RNG stream
        player_class: Player class name

    Returns:
        Tuple of (list of card IDs, number of RNG calls made)
    """
    cards: List[str] = []
    calls_made = 0

    # Track cards by type to avoid duplicates within same type
    # The game tracks "previous card of same type" for duplicate checking
    cards_by_type: Dict[CardType, List[str]] = {
        CardType.ATTACK: [],
        CardType.SKILL: [],
        CardType.POWER: [],
    }

    for card_type in SHOP_CARD_TYPES:
        max_attempts = 100  # Safety limit
        attempts = 0

        while attempts < max_attempts:
            # Roll rarity
            rarity = _roll_shop_rarity(card_rng)
            calls_made += 1

            # Get pool for this type+rarity
            pool = _get_card_pool_by_type_and_rarity(card_type, rarity, player_class)

            if not pool:
                # No cards available for this combo, try different rarity
                attempts += 1
                continue

            # Pick random card from pool
            idx = card_rng.random(len(pool) - 1)
            calls_made += 1
            card_id = pool[idx]

            # Check if card is colorless (shouldn't be in colored shop)
            if _is_colorless_card(card_id):
                attempts += 1
                continue

            # Check if card is duplicate of previous same-type card in shop
            # Game checks: is this card already in the shop as the same type?
            if card_id in cards_by_type[card_type]:
                attempts += 1
                continue

            # Valid card found
            cards.append(card_id)
            cards_by_type[card_type].append(card_id)
            break

        if attempts >= max_attempts:
            # Fallback: just use the last card we rolled
            # This shouldn't happen in practice
            pass

    return cards, calls_made


# ============================================================================
# COLORLESS CARD GENERATION
# ============================================================================

def _get_colorless_pool_by_rarity(rarity: CardRarity) -> List[str]:
    """
    Get colorless card pool by rarity in HashMap iteration order.

    Note: For shop, only UNCOMMON and RARE colorless cards are available.
    """
    result = []
    for card_id, card in COLORLESS_CARDS.items():
        if card.rarity == rarity:
            result.append(card_id)
    return result


def _generate_shop_colorless_cards(
    card_rng: Random,
    existing_card_ids: Set[str],
) -> Tuple[List[str], int]:
    """
    Generate the 2 colorless cards for the shop.

    From ShopScreen.initCards():
    - First colorless slot: UNCOMMON
    - Second colorless slot: RARE
    - No duplicate checking against colored cards
    - No duplicate checking between colorless cards

    Args:
        card_rng: The card RNG stream
        existing_card_ids: Cards already in shop (not used in game, but available)

    Returns:
        Tuple of (list of card IDs, number of RNG calls made)
    """
    cards: List[str] = []
    calls_made = 0

    # First colorless: UNCOMMON
    uncommon_pool = _get_colorless_pool_by_rarity(CardRarity.UNCOMMON)
    if uncommon_pool:
        idx = card_rng.random(len(uncommon_pool) - 1)
        calls_made += 1
        cards.append(uncommon_pool[idx])

    # Second colorless: RARE
    rare_pool = _get_colorless_pool_by_rarity(CardRarity.RARE)
    if rare_pool:
        idx = card_rng.random(len(rare_pool) - 1)
        calls_made += 1
        cards.append(rare_pool[idx])

    return cards, calls_made


# ============================================================================
# PRICE CALCULATION
# ============================================================================

def _calculate_card_price(
    rng: Random,
    card: Card,
    is_colorless: bool = False,
    discount_multiplier: float = 1.0,
) -> Tuple[int, int]:
    """
    Calculate card price with random variance.

    From ShopScreen.getPrice(AbstractCard):
    - Base price by rarity
    - Multiply by 0.9-1.1 random variance
    - Colorless cards: 1.2x multiplier
    - Apply any discount (Membership Card, Courier)

    Args:
        rng: The merchant RNG stream
        card: The card
        is_colorless: Whether this is a colorless card
        discount_multiplier: Discount to apply (0.5 for Membership, 0.8 for Courier)

    Returns:
        Tuple of (price, number of RNG calls made)
    """
    base_price = CARD_BASE_PRICES.get(card.rarity, CARD_BASE_PRICES[CardRarity.COMMON])

    if is_colorless:
        base_price = int(base_price * COLORLESS_PRICE_MULTIPLIER)

    # Random variance 0.9-1.1
    variance = rng.random_float_range(0.9, 1.1)
    price = int(base_price * variance * discount_multiplier)

    return price, 1


def _calculate_relic_price(
    rng: Random,
    relic: Relic,
    discount_multiplier: float = 1.0,
) -> Tuple[int, int]:
    """
    Calculate relic price with random variance.

    Args:
        rng: The merchant RNG stream
        relic: The relic
        discount_multiplier: Discount to apply

    Returns:
        Tuple of (price, number of RNG calls made)
    """
    base_price = RELIC_BASE_PRICES.get(relic.tier, RELIC_BASE_PRICES[RelicTier.COMMON])

    # Random variance 0.95-1.05
    variance = rng.random_float_range(0.95, 1.05)
    price = int(base_price * variance * discount_multiplier)

    return price, 1


def _calculate_potion_price(
    rng: Random,
    potion: Potion,
    discount_multiplier: float = 1.0,
) -> Tuple[int, int]:
    """
    Calculate potion price with random variance.

    Args:
        rng: The merchant RNG stream
        potion: The potion
        discount_multiplier: Discount to apply

    Returns:
        Tuple of (price, number of RNG calls made)
    """
    base_price = POTION_BASE_PRICES.get(potion.rarity, POTION_BASE_PRICES[PotionRarity.COMMON])

    # Random variance 0.95-1.05
    variance = rng.random_float_range(0.95, 1.05)
    price = int(base_price * variance * discount_multiplier)

    return price, 1


# ============================================================================
# RELIC GENERATION
# ============================================================================

def _roll_shop_relic_tier(rng: Random) -> RelicTier:
    """
    Roll relic tier for shop relics.

    From ShopScreen.rollRelicTier():
    - roll 0-99
    - < 48 = COMMON
    - < 82 = UNCOMMON
    - else RARE

    Args:
        rng: The merchant RNG stream

    Returns:
        RelicTier
    """
    roll = rng.random(99)

    if roll < SHOP_RELIC_THRESHOLDS["common"]:
        return RelicTier.COMMON
    elif roll < SHOP_RELIC_THRESHOLDS["uncommon"]:
        return RelicTier.UNCOMMON
    else:
        return RelicTier.RARE


def _get_available_relics(
    tier: RelicTier,
    owned_relics: Set[str],
    player_class: str = "WATCHER",
    act: int = 1,
) -> List[str]:
    """
    Get available relics of a tier, excluding owned ones.

    Also handles class restrictions and act restrictions.
    """
    # Map player class string to enum
    class_map = {
        "WATCHER": PlayerClass.WATCHER,
        "IRONCLAD": PlayerClass.IRONCLAD,
        "SILENT": PlayerClass.SILENT,
        "DEFECT": PlayerClass.DEFECT,
    }
    pc = class_map.get(player_class, PlayerClass.WATCHER)

    result = []
    for relic_id, relic in ALL_RELICS.items():
        # Check tier
        if relic.tier != tier:
            continue

        # Check if owned
        if relic_id in owned_relics:
            continue

        # Check class restriction
        if relic.player_class != PlayerClass.ALL and relic.player_class != pc:
            continue

        # Check act restriction (e.g., Ectoplasm only in Act 1)
        if relic.act_restriction is not None and act > relic.act_restriction:
            continue

        # Check requires_relic (boss upgrades)
        if relic.requires_relic is not None:
            if relic.requires_relic not in owned_relics:
                continue

        result.append(relic_id)

    return result


def _generate_shop_relics(
    merchant_rng: Random,
    owned_relics: Set[str],
    player_class: str = "WATCHER",
    act: int = 1,
) -> Tuple[List[str], int]:
    """
    Generate the 3 relics for the shop.

    From ShopScreen.initRelics():
    - First 2 relics: random tier roll
    - Third relic: always SHOP tier

    Args:
        merchant_rng: The merchant RNG stream
        owned_relics: Set of owned relic IDs
        player_class: Player class name
        act: Current act

    Returns:
        Tuple of (list of relic IDs, number of RNG calls made)
    """
    relics: List[str] = []
    calls_made = 0
    temp_owned = owned_relics.copy()

    # First two relics: random tier
    for _ in range(2):
        tier = _roll_shop_relic_tier(merchant_rng)
        calls_made += 1

        pool = _get_available_relics(tier, temp_owned, player_class, act)

        # Fallback to higher tiers if pool empty
        while not pool and tier != RelicTier.RARE:
            if tier == RelicTier.COMMON:
                tier = RelicTier.UNCOMMON
            elif tier == RelicTier.UNCOMMON:
                tier = RelicTier.RARE
            pool = _get_available_relics(tier, temp_owned, player_class, act)

        if pool:
            idx = merchant_rng.random(len(pool) - 1)
            calls_made += 1
            relic_id = pool[idx]
            relics.append(relic_id)
            temp_owned.add(relic_id)
        else:
            # No relics available - add Circlet
            relics.append("Circlet")
            calls_made += 1  # Still consume RNG

    # Third relic: SHOP tier
    shop_pool = _get_available_relics(RelicTier.SHOP, temp_owned, player_class, act)
    if shop_pool:
        idx = merchant_rng.random(len(shop_pool) - 1)
        calls_made += 1
        relics.append(shop_pool[idx])
    else:
        relics.append("Circlet")
        calls_made += 1

    return relics, calls_made


# ============================================================================
# POTION GENERATION
# ============================================================================

def _roll_potion_rarity(rng: Random) -> PotionRarity:
    """
    Roll potion rarity.

    From PotionHelper:
    - 65% COMMON
    - 25% UNCOMMON
    - 10% RARE
    """
    roll = rng.random(99)

    if roll < 65:
        return PotionRarity.COMMON
    elif roll < 90:
        return PotionRarity.UNCOMMON
    else:
        return PotionRarity.RARE


def _generate_shop_potions(
    potion_rng: Random,
    player_class: str = "WATCHER",
) -> Tuple[List[str], int]:
    """
    Generate the 3 potions for the shop.

    Args:
        potion_rng: The potion RNG stream
        player_class: Player class name

    Returns:
        Tuple of (list of potion IDs, number of RNG calls made)
    """
    # Get potion pool for player class
    class_map = {
        "WATCHER": PotionPlayerClass.WATCHER,
        "IRONCLAD": PotionPlayerClass.IRONCLAD,
        "SILENT": PotionPlayerClass.SILENT,
        "DEFECT": PotionPlayerClass.DEFECT,
    }
    pc = class_map.get(player_class, PotionPlayerClass.WATCHER)
    pool = get_potion_pool(pc)

    potions: List[str] = []
    calls_made = 0

    for _ in range(3):
        # Roll rarity
        rarity = _roll_potion_rarity(potion_rng)
        calls_made += 1

        # Filter pool by rarity
        rarity_pool = [p for p in pool if p.rarity == rarity]

        if rarity_pool:
            idx = potion_rng.random(len(rarity_pool) - 1)
            calls_made += 1
            potions.append(rarity_pool[idx].id)
        elif pool:
            # Fallback to any potion
            idx = potion_rng.random(len(pool) - 1)
            calls_made += 1
            potions.append(pool[idx].id)

    return potions, calls_made


# ============================================================================
# MAIN PREDICTION FUNCTION
# ============================================================================

def predict_shop_inventory(
    seed: str,
    card_counter: int,
    merchant_counter: int,
    potion_counter: int,
    act: int = 1,
    player_class: str = "WATCHER",
    owned_relics: Optional[Set[str]] = None,
    purge_count: int = 0,
    has_membership_card: bool = False,
    has_the_courier: bool = False,
    has_smiling_mask: bool = False,
    ascension_level: int = 0,
) -> ShopPredictionResult:
    """
    Predict complete shop inventory for a given game state.

    This function initializes RNG streams at the specified counter values
    and generates the shop inventory exactly as the game would.

    Args:
        seed: The game seed string (e.g., "ABC123")
        card_counter: Current cardRng counter value
        merchant_counter: Current merchantRng counter value
        potion_counter: Current potionRng counter value
        act: Current act (1-4)
        player_class: Player class name
        owned_relics: Set of owned relic IDs (for exclusion)
        purge_count: Number of previous card removals
        has_membership_card: 50% shop discount
        has_the_courier: 20% shop discount + always has removal
        has_smiling_mask: Overrides purge cost to flat 50g
        ascension_level: Ascension level (A16+ adds 10% price markup)

    Returns:
        ShopPredictionResult with inventory and final counter values
    """
    # Convert seed string to long
    seed_long = seed_to_long(seed)

    # Initialize RNG streams at specified counters
    card_rng = Random(seed_long, card_counter)
    merchant_rng = Random(seed_long, merchant_counter)
    potion_rng = Random(seed_long, potion_counter)

    # Calculate discount multiplier
    discount = 1.0
    # A16+ applies 10% price increase (Java: applyDiscount(1.1f))
    if ascension_level >= 16:
        discount *= 1.1
    if has_membership_card:
        discount *= 0.5
    if has_the_courier:
        discount *= 0.8

    # Default owned relics to starter only
    if owned_relics is None:
        owned_relics = {"PureWater"} if player_class == "WATCHER" else set()

    # ========== GENERATE CARDS ==========

    # Generate colored cards
    colored_card_ids, colored_rng_calls = _generate_shop_colored_cards(
        card_rng, player_class
    )

    # Generate colorless cards
    colorless_card_ids, colorless_rng_calls = _generate_shop_colorless_cards(
        card_rng, set(colored_card_ids)
    )

    # ========== GENERATE RELICS ==========

    relic_ids, relic_rng_calls = _generate_shop_relics(
        merchant_rng, owned_relics, player_class, act
    )

    # ========== GENERATE POTIONS ==========

    potion_ids, potion_rng_calls = _generate_shop_potions(
        potion_rng, player_class
    )

    # ========== CALCULATE PRICES ==========

    merchant_price_calls = 0

    # Colored card prices
    colored_cards: List[ShopCard] = []
    for card_id in colored_card_ids:
        card = get_card(card_id)
        price, calls = _calculate_card_price(merchant_rng, card, False, discount)
        merchant_price_calls += calls
        colored_cards.append(ShopCard(card=card, price=price))

    # Colorless card prices
    colorless_cards: List[ShopCard] = []
    for card_id in colorless_card_ids:
        card = get_card(card_id)
        price, calls = _calculate_card_price(merchant_rng, card, True, discount)
        merchant_price_calls += calls
        colorless_cards.append(ShopCard(card=card, price=price))

    # Relic prices
    relics: List[ShopRelic] = []
    for relic_id in relic_ids:
        try:
            relic = get_relic(relic_id)
        except ValueError:
            # Handle Circlet or unknown relics
            relic = Relic(id=relic_id, name=relic_id, tier=RelicTier.SPECIAL)
        price, calls = _calculate_relic_price(merchant_rng, relic, discount)
        merchant_price_calls += calls
        relics.append(ShopRelic(relic=relic, price=price))

    # Potion prices
    potions: List[ShopPotion] = []
    for potion_id in potion_ids:
        potion = ALL_POTIONS.get(potion_id)
        if potion:
            price, calls = _calculate_potion_price(merchant_rng, potion, discount)
            merchant_price_calls += calls
            potions.append(ShopPotion(potion=potion, price=price))

    # ========== SELECT SALE CARD ==========

    # One random colored card gets 50% off
    sale_index = merchant_rng.random(len(colored_cards) - 1) if colored_cards else 0
    if colored_cards:
        colored_cards[sale_index].price = int(colored_cards[sale_index].price * SALE_DISCOUNT)
        colored_cards[sale_index].on_sale = True
    merchant_price_calls += 1

    # ========== CALCULATE PURGE COST ==========

    # Smiling Mask overrides purge cost to flat 50g
    if has_smiling_mask:
        purge_cost = 50
    else:
        purge_cost = BASE_PURGE_COST + (purge_count * PURGE_COST_INCREMENT)
    purge_cost = int(purge_cost * discount)

    # ========== BUILD RESULT ==========

    inventory = PredictedShopInventory(
        colored_cards=colored_cards,
        colorless_cards=colorless_cards,
        relics=relics,
        potions=potions,
        purge_cost=purge_cost,
        sale_card_index=sale_index,
        card_rng_calls=colored_rng_calls + colorless_rng_calls,
        merchant_rng_calls=relic_rng_calls + merchant_price_calls,
        potion_rng_calls=potion_rng_calls,
    )

    return ShopPredictionResult(
        inventory=inventory,
        final_card_counter=card_rng.counter,
        final_merchant_counter=merchant_rng.counter,
        final_potion_counter=potion_rng.counter,
    )


# ============================================================================
# UTILITY FUNCTIONS
# ============================================================================

def format_shop_inventory(result: ShopPredictionResult, verbose: bool = False) -> str:
    """Format shop inventory as a readable string."""
    inv = result.inventory
    lines = []

    lines.append("=== SHOP INVENTORY ===")
    lines.append("")

    lines.append("COLORED CARDS:")
    for i, sc in enumerate(inv.colored_cards):
        sale_marker = " [ON SALE]" if sc.on_sale else ""
        type_info = f" [{sc.card.card_type.name}]" if verbose else ""
        lines.append(f"  {sc.card.name} ({sc.card.rarity.name}){type_info} - {sc.price}g{sale_marker}")

    lines.append("")
    lines.append("COLORLESS CARDS:")
    for sc in inv.colorless_cards:
        lines.append(f"  {sc.card.name} ({sc.card.rarity.name}) - {sc.price}g")

    lines.append("")
    lines.append("RELICS:")
    for sr in inv.relics:
        lines.append(f"  {sr.relic.name} ({sr.relic.tier.name}) - {sr.price}g")

    lines.append("")
    lines.append("POTIONS:")
    for sp in inv.potions:
        lines.append(f"  {sp.potion.name} ({sp.potion.rarity.name}) - {sp.price}g")

    lines.append("")
    lines.append(f"Card Removal: {inv.purge_cost}g")

    lines.append("")
    lines.append("=== RNG CONSUMPTION ===")
    lines.append(f"cardRng calls: {inv.card_rng_calls}")
    lines.append(f"merchantRng calls: {inv.merchant_rng_calls}")
    lines.append(f"potionRng calls: {inv.potion_rng_calls}")

    lines.append("")
    lines.append("=== FINAL COUNTERS ===")
    lines.append(f"cardRng: {result.final_card_counter}")
    lines.append(f"merchantRng: {result.final_merchant_counter}")
    lines.append(f"potionRng: {result.final_potion_counter}")

    return "\n".join(lines)


# ============================================================================
# TESTING
# ============================================================================

if __name__ == "__main__":
    print("=== Shop Inventory Prediction Tests ===\n")

    # Test with a known seed
    seed = "TESTSEED123"
    print(f"Seed: {seed}")
    print(f"Player: WATCHER")
    print(f"Act: 1")
    print(f"Starting counters: card=0, merchant=0, potion=0")
    print()

    result = predict_shop_inventory(
        seed=seed,
        card_counter=0,
        merchant_counter=0,
        potion_counter=0,
        act=1,
        player_class="WATCHER",
    )

    print(format_shop_inventory(result))

    print("\n" + "="*50 + "\n")

    # Test with higher counters (simulating mid-run shop)
    print("Testing with higher counters (simulating mid-run)")
    print(f"Starting counters: card=50, merchant=20, potion=10")
    print()

    result2 = predict_shop_inventory(
        seed=seed,
        card_counter=50,
        merchant_counter=20,
        potion_counter=10,
        act=2,
        player_class="WATCHER",
        owned_relics={"PureWater", "Akabeko", "Kunai"},
    )

    print(format_shop_inventory(result2))

    print("\n" + "="*50 + "\n")

    # Test with discount relics
    print("Testing with Membership Card + The Courier")
    print()

    result3 = predict_shop_inventory(
        seed=seed,
        card_counter=0,
        merchant_counter=0,
        potion_counter=0,
        act=1,
        player_class="WATCHER",
        has_membership_card=True,
        has_the_courier=True,
    )

    print(format_shop_inventory(result3))
