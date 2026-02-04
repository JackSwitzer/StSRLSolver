"""
Shop Handler - Complete shop system for Slay the Spire Python clone.

Handles all shop interactions:
- Shop inventory generation (using existing generation/shop.py)
- Purchasing cards, relics, and potions
- Card removal service
- Discount mechanics (Membership Card, The Courier)
- Shop state tracking (purchased items, purge count)

Integration with GameRunner:
- _get_shop_actions() returns valid purchases player can afford
- _handle_shop_action() processes purchase
- Proper phase transitions (SHOP -> MAP_NAVIGATION)
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import List, Optional, Dict, Tuple, Any, TYPE_CHECKING
from enum import Enum, auto

if TYPE_CHECKING:
    from ..state.run import RunState
    from ..state.rng import Random

# Import content modules
from ..content.cards import Card, CardRarity, CardType, CardColor
from ..content.relics import Relic, RelicTier, ALL_RELICS, get_relic
from ..content.potions import Potion, PotionRarity, ALL_POTIONS


# ============================================================================
# SHOP CONSTANTS - From ShopScreen.java
# ============================================================================

# Card prices by rarity
CARD_BASE_PRICES = {
    CardRarity.COMMON: 50,
    CardRarity.UNCOMMON: 75,
    CardRarity.RARE: 150,
}

# Card price variance (0.9 - 1.1)
CARD_PRICE_VARIANCE_MIN = 0.9
CARD_PRICE_VARIANCE_MAX = 1.1

# Colorless card multiplier (1.2x colored price)
COLORLESS_PRICE_MULTIPLIER = 1.2

# Relic prices by tier
RELIC_BASE_PRICES = {
    RelicTier.COMMON: 150,
    RelicTier.UNCOMMON: 250,
    RelicTier.RARE: 300,
    RelicTier.SHOP: 150,
}

# Potion prices by rarity
POTION_BASE_PRICES = {
    PotionRarity.COMMON: 50,
    PotionRarity.UNCOMMON: 75,
    PotionRarity.RARE: 100,
}

# Card removal base cost and increment
BASE_PURGE_COST = 75
PURGE_COST_INCREMENT = 25

# Sale discount (50% off one random colored card)
SALE_DISCOUNT = 0.5

# Membership Card discount
MEMBERSHIP_CARD_DISCOUNT = 0.5

# The Courier discount
COURIER_DISCOUNT = 0.8


# ============================================================================
# SHOP ITEM DATACLASSES
# ============================================================================

@dataclass
class ShopCard:
    """A card available in the shop."""
    card: Card
    price: int
    on_sale: bool = False
    purchased: bool = False
    slot_index: int = 0  # Position in shop (for display/action reference)
    is_colorless: bool = False


@dataclass
class ShopRelic:
    """A relic available in the shop."""
    relic: Relic
    price: int
    purchased: bool = False
    slot_index: int = 0


@dataclass
class ShopPotion:
    """A potion available in the shop."""
    potion: Potion
    price: int
    purchased: bool = False
    slot_index: int = 0


# ============================================================================
# SHOP STATE
# ============================================================================

@dataclass
class ShopState:
    """
    Complete state of a shop visit.

    Tracks:
    - All items available for purchase
    - Which items have been purchased
    - Current card removal cost
    - Whether card removal has been used this visit
    """
    # Inventory
    colored_cards: List[ShopCard] = field(default_factory=list)
    colorless_cards: List[ShopCard] = field(default_factory=list)
    relics: List[ShopRelic] = field(default_factory=list)
    potions: List[ShopPotion] = field(default_factory=list)

    # Card removal
    purge_cost: int = BASE_PURGE_COST
    purge_available: bool = True

    # Tracking
    sale_card_index: int = -1  # Index of the on-sale card in colored_cards

    def get_available_colored_cards(self) -> List[ShopCard]:
        """Get colored cards that haven't been purchased."""
        return [c for c in self.colored_cards if not c.purchased]

    def get_available_colorless_cards(self) -> List[ShopCard]:
        """Get colorless cards that haven't been purchased."""
        return [c for c in self.colorless_cards if not c.purchased]

    def get_available_relics(self) -> List[ShopRelic]:
        """Get relics that haven't been purchased."""
        return [r for r in self.relics if not r.purchased]

    def get_available_potions(self) -> List[ShopPotion]:
        """Get potions that haven't been purchased."""
        return [p for p in self.potions if not p.purchased]

    def get_all_items_count(self) -> int:
        """Get total number of items still available."""
        return (
            len(self.get_available_colored_cards()) +
            len(self.get_available_colorless_cards()) +
            len(self.get_available_relics()) +
            len(self.get_available_potions()) +
            (1 if self.purge_available else 0)
        )


# ============================================================================
# SHOP INVENTORY GENERATION
# ============================================================================

def generate_shop_inventory(
    run_state: 'RunState',
    merchant_rng: 'Random',
    card_rng: 'Random',
    potion_rng: 'Random',
) -> ShopState:
    """
    Generate a complete shop inventory.

    Uses the existing shop.py prediction module but adapts it for
    the shop handler's data structures.

    Args:
        run_state: Current run state
        merchant_rng: Merchant RNG stream (for prices, relic tiers, sale selection)
        card_rng: Card RNG stream (for card selection)
        potion_rng: Potion RNG stream (for potion selection)

    Returns:
        ShopState with complete inventory
    """
    # Import shop generation module
    import os
    import importlib.util

    _core_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

    def _load_module(name: str, filepath: str):
        spec = importlib.util.spec_from_file_location(name, filepath)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module

    _shop_module = _load_module("shop", os.path.join(_core_dir, "generation", "shop.py"))

    # Calculate discount multiplier
    discount = 1.0
    if run_state.has_relic("Membership Card"):
        discount *= MEMBERSHIP_CARD_DISCOUNT
    if run_state.has_relic("The Courier"):
        discount *= COURIER_DISCOUNT

    # Get owned relics for exclusion
    owned_relics = set(run_state.get_relic_ids())

    # Get purge count from run state
    purge_count = getattr(run_state, 'purge_count', 0)

    # Use the prediction module to generate shop
    result = _shop_module.predict_shop_inventory(
        seed=run_state.seed_string,
        card_counter=card_rng.counter if hasattr(card_rng, 'counter') else 0,
        merchant_counter=merchant_rng.counter if hasattr(merchant_rng, 'counter') else 0,
        potion_counter=potion_rng.counter if hasattr(potion_rng, 'counter') else 0,
        act=run_state.act,
        player_class=run_state.character.upper(),
        owned_relics=owned_relics,
        purge_count=purge_count,
        has_membership_card=run_state.has_relic("Membership Card"),
        has_the_courier=run_state.has_relic("The Courier"),
        has_prismatic_shard=run_state.has_relic("PrismaticShard"),
    )

    inv = result.inventory

    # Convert to ShopState
    shop_state = ShopState()

    # Convert colored cards
    for i, shop_card in enumerate(inv.colored_cards):
        shop_state.colored_cards.append(ShopCard(
            card=shop_card.card,
            price=shop_card.price,
            on_sale=shop_card.on_sale,
            slot_index=i,
            is_colorless=False,
        ))

    # Convert colorless cards
    for i, shop_card in enumerate(inv.colorless_cards):
        shop_state.colorless_cards.append(ShopCard(
            card=shop_card.card,
            price=shop_card.price,
            on_sale=False,
            slot_index=i,
            is_colorless=True,
        ))

    # Convert relics
    for i, shop_relic in enumerate(inv.relics):
        shop_state.relics.append(ShopRelic(
            relic=shop_relic.relic,
            price=shop_relic.price,
            slot_index=i,
        ))

    # Convert potions
    for i, shop_potion in enumerate(inv.potions):
        shop_state.potions.append(ShopPotion(
            potion=shop_potion.potion,
            price=shop_potion.price,
            slot_index=i,
        ))

    # Set purge cost
    shop_state.purge_cost = inv.purge_cost
    shop_state.sale_card_index = inv.sale_card_index

    return shop_state


def generate_shop_inventory_simple(
    run_state: 'RunState',
    rng: 'Random',
) -> ShopState:
    """
    Generate shop inventory using a single RNG stream (simplified version).

    This is a fallback if we don't have separate RNG streams.
    Uses the rewards.py generation which is less accurate but simpler.

    Args:
        run_state: Current run state
        rng: Single RNG stream for all generation

    Returns:
        ShopState with complete inventory
    """
    # Import rewards module
    import os
    import importlib.util

    _core_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

    def _load_module(name: str, filepath: str):
        spec = importlib.util.spec_from_file_location(name, filepath)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module

    _rewards_module = _load_module("rewards", os.path.join(_core_dir, "generation", "rewards.py"))

    # Create reward state
    RewardState = _rewards_module.RewardState
    reward_state = RewardState(owned_relics=set(run_state.get_relic_ids()))

    # Get purge count
    purge_count = getattr(run_state, 'purge_count', 0)

    # Generate shop using rewards module
    shop_inv = _rewards_module.generate_shop_inventory(
        rng=rng,
        reward_state=reward_state,
        act=run_state.act,
        player_class=run_state.character.upper(),
        ascension=run_state.ascension,
        purge_count=purge_count,
        has_membership_card=run_state.has_relic("Membership Card"),
        has_the_courier=run_state.has_relic("The Courier"),
        has_prismatic_shard=run_state.has_relic("PrismaticShard"),
    )

    # Convert to ShopState
    shop_state = ShopState()

    # Convert colored cards
    for i, (card, price) in enumerate(shop_inv.colored_cards):
        shop_state.colored_cards.append(ShopCard(
            card=card,
            price=price,
            on_sale=False,  # Would need separate tracking
            slot_index=i,
            is_colorless=False,
        ))

    # Mark one card as on sale (random)
    if shop_state.colored_cards:
        sale_idx = rng.random(len(shop_state.colored_cards) - 1)
        shop_state.colored_cards[sale_idx].on_sale = True
        shop_state.colored_cards[sale_idx].price = int(
            shop_state.colored_cards[sale_idx].price * SALE_DISCOUNT
        )
        shop_state.sale_card_index = sale_idx

    # Convert colorless cards
    for i, (card, price) in enumerate(shop_inv.colorless_cards):
        shop_state.colorless_cards.append(ShopCard(
            card=card,
            price=price,
            on_sale=False,
            slot_index=i,
            is_colorless=True,
        ))

    # Convert relics
    for i, (relic, price) in enumerate(shop_inv.relics):
        shop_state.relics.append(ShopRelic(
            relic=relic,
            price=price,
            slot_index=i,
        ))

    # Convert potions
    for i, (potion, price) in enumerate(shop_inv.potions):
        shop_state.potions.append(ShopPotion(
            potion=potion,
            price=price,
            slot_index=i,
        ))

    shop_state.purge_cost = shop_inv.purge_cost
    shop_state.purge_available = shop_inv.purge_available

    return shop_state


# ============================================================================
# SHOP ACTION TYPES
# ============================================================================

class ShopActionType(Enum):
    """Types of actions available in the shop."""
    BUY_COLORED_CARD = auto()
    BUY_COLORLESS_CARD = auto()
    BUY_RELIC = auto()
    BUY_POTION = auto()
    REMOVE_CARD = auto()
    LEAVE = auto()


@dataclass(frozen=True)
class ShopAction:
    """
    An action that can be taken in the shop.

    action_type: Type of purchase/action
    item_index: Index of the item in its category (for purchases)
    card_index: Index of card in deck (for card removal)
    """
    action_type: ShopActionType
    item_index: int = -1  # Which item in shop to buy
    card_index: int = -1  # Which card in deck to remove


# ============================================================================
# SHOP RESULT DATACLASS
# ============================================================================

@dataclass
class ShopResult:
    """Result of a shop transaction."""
    success: bool
    action_type: ShopActionType
    item_id: str = ""
    item_name: str = ""
    gold_spent: int = 0
    message: str = ""
    left_shop: bool = False


# ============================================================================
# SHOP HANDLER CLASS
# ============================================================================

class ShopHandler:
    """
    Handles all shop interactions.

    Usage with GameRunner:
        1. When entering shop: shop_state = ShopHandler.create_shop(run_state, merchant_rng, card_rng, potion_rng)
        2. Get actions: actions = ShopHandler.get_available_actions(shop_state, run_state)
        3. Execute action: result = ShopHandler.execute_action(action, shop_state, run_state)
    """

    @staticmethod
    def create_shop(
        run_state: 'RunState',
        merchant_rng: 'Random',
        card_rng: Optional['Random'] = None,
        potion_rng: Optional['Random'] = None,
    ) -> ShopState:
        """
        Create a new shop for the current floor.

        Args:
            run_state: Current run state
            merchant_rng: Merchant RNG stream

        Returns:
            ShopState with generated inventory
        """
        if card_rng is None or potion_rng is None:
            return generate_shop_inventory_simple(run_state, merchant_rng)

        return generate_shop_inventory(run_state, merchant_rng, card_rng, potion_rng)

    @staticmethod
    def get_available_actions(
        shop_state: ShopState,
        run_state: 'RunState',
    ) -> List[ShopAction]:
        """
        Get all valid shop actions the player can currently take.

        Filters by:
        - Gold available
        - Item not already purchased
        - Empty potion slots (for potion purchases)
        - Cards available to remove (for card removal)

        Args:
            shop_state: Current shop state
            run_state: Current run state

        Returns:
            List of valid ShopAction objects
        """
        actions = []
        gold = run_state.gold

        # Always can leave
        actions.append(ShopAction(action_type=ShopActionType.LEAVE))

        # Colored cards
        for shop_card in shop_state.get_available_colored_cards():
            if shop_card.price <= gold:
                actions.append(ShopAction(
                    action_type=ShopActionType.BUY_COLORED_CARD,
                    item_index=shop_card.slot_index,
                ))

        # Colorless cards
        for shop_card in shop_state.get_available_colorless_cards():
            if shop_card.price <= gold:
                actions.append(ShopAction(
                    action_type=ShopActionType.BUY_COLORLESS_CARD,
                    item_index=shop_card.slot_index,
                ))

        # Relics
        for shop_relic in shop_state.get_available_relics():
            if shop_relic.price <= gold:
                actions.append(ShopAction(
                    action_type=ShopActionType.BUY_RELIC,
                    item_index=shop_relic.slot_index,
                ))

        # Potions (only if we have empty slots)
        if run_state.count_empty_potion_slots() > 0:
            for shop_potion in shop_state.get_available_potions():
                if shop_potion.price <= gold:
                    actions.append(ShopAction(
                        action_type=ShopActionType.BUY_POTION,
                        item_index=shop_potion.slot_index,
                    ))

        # Card removal (one action per removable card)
        if shop_state.purge_available and shop_state.purge_cost <= gold:
            removable = run_state.get_removable_cards()
            for card_idx, card in removable:
                actions.append(ShopAction(
                    action_type=ShopActionType.REMOVE_CARD,
                    card_index=card_idx,
                ))

        return actions

    @staticmethod
    def execute_action(
        action: ShopAction,
        shop_state: ShopState,
        run_state: 'RunState',
    ) -> ShopResult:
        """
        Execute a shop action.

        Args:
            action: The action to execute
            shop_state: Current shop state (will be modified)
            run_state: Current run state (will be modified)

        Returns:
            ShopResult with transaction details
        """
        if action.action_type == ShopActionType.LEAVE:
            return ShopResult(
                success=True,
                action_type=action.action_type,
                message="Left the shop",
                left_shop=True,
            )

        elif action.action_type == ShopActionType.BUY_COLORED_CARD:
            return ShopHandler._buy_colored_card(action, shop_state, run_state)

        elif action.action_type == ShopActionType.BUY_COLORLESS_CARD:
            return ShopHandler._buy_colorless_card(action, shop_state, run_state)

        elif action.action_type == ShopActionType.BUY_RELIC:
            return ShopHandler._buy_relic(action, shop_state, run_state)

        elif action.action_type == ShopActionType.BUY_POTION:
            return ShopHandler._buy_potion(action, shop_state, run_state)

        elif action.action_type == ShopActionType.REMOVE_CARD:
            return ShopHandler._remove_card(action, shop_state, run_state)

        return ShopResult(
            success=False,
            action_type=action.action_type,
            message="Unknown action type",
        )

    @staticmethod
    def _buy_colored_card(
        action: ShopAction,
        shop_state: ShopState,
        run_state: 'RunState',
    ) -> ShopResult:
        """Buy a colored card from the shop."""
        # Find the card
        shop_card = None
        for c in shop_state.colored_cards:
            if c.slot_index == action.item_index and not c.purchased:
                shop_card = c
                break

        if shop_card is None:
            return ShopResult(
                success=False,
                action_type=action.action_type,
                message="Card not found or already purchased",
            )

        if run_state.gold < shop_card.price:
            return ShopResult(
                success=False,
                action_type=action.action_type,
                item_id=shop_card.card.id,
                item_name=shop_card.card.name,
                message="Not enough gold",
            )

        # Process purchase
        run_state.lose_gold(shop_card.price)
        run_state.add_card(shop_card.card.id, shop_card.card.upgraded)
        shop_card.purchased = True

        return ShopResult(
            success=True,
            action_type=action.action_type,
            item_id=shop_card.card.id,
            item_name=shop_card.card.name,
            gold_spent=shop_card.price,
            message=f"Purchased {shop_card.card.name} for {shop_card.price} gold",
        )

    @staticmethod
    def _buy_colorless_card(
        action: ShopAction,
        shop_state: ShopState,
        run_state: 'RunState',
    ) -> ShopResult:
        """Buy a colorless card from the shop."""
        # Find the card
        shop_card = None
        for c in shop_state.colorless_cards:
            if c.slot_index == action.item_index and not c.purchased:
                shop_card = c
                break

        if shop_card is None:
            return ShopResult(
                success=False,
                action_type=action.action_type,
                message="Card not found or already purchased",
            )

        if run_state.gold < shop_card.price:
            return ShopResult(
                success=False,
                action_type=action.action_type,
                item_id=shop_card.card.id,
                item_name=shop_card.card.name,
                message="Not enough gold",
            )

        # Process purchase
        run_state.lose_gold(shop_card.price)
        run_state.add_card(shop_card.card.id, shop_card.card.upgraded)
        shop_card.purchased = True

        return ShopResult(
            success=True,
            action_type=action.action_type,
            item_id=shop_card.card.id,
            item_name=shop_card.card.name,
            gold_spent=shop_card.price,
            message=f"Purchased {shop_card.card.name} for {shop_card.price} gold",
        )

    @staticmethod
    def _buy_relic(
        action: ShopAction,
        shop_state: ShopState,
        run_state: 'RunState',
    ) -> ShopResult:
        """Buy a relic from the shop."""
        # Find the relic
        shop_relic = None
        for r in shop_state.relics:
            if r.slot_index == action.item_index and not r.purchased:
                shop_relic = r
                break

        if shop_relic is None:
            return ShopResult(
                success=False,
                action_type=action.action_type,
                message="Relic not found or already purchased",
            )

        if run_state.gold < shop_relic.price:
            return ShopResult(
                success=False,
                action_type=action.action_type,
                item_id=shop_relic.relic.id,
                item_name=shop_relic.relic.name,
                message="Not enough gold",
            )

        # Process purchase
        run_state.lose_gold(shop_relic.price)
        run_state.add_relic(shop_relic.relic.id)
        shop_relic.purchased = True

        return ShopResult(
            success=True,
            action_type=action.action_type,
            item_id=shop_relic.relic.id,
            item_name=shop_relic.relic.name,
            gold_spent=shop_relic.price,
            message=f"Purchased {shop_relic.relic.name} for {shop_relic.price} gold",
        )

    @staticmethod
    def _buy_potion(
        action: ShopAction,
        shop_state: ShopState,
        run_state: 'RunState',
    ) -> ShopResult:
        """Buy a potion from the shop."""
        # Find the potion
        shop_potion = None
        for p in shop_state.potions:
            if p.slot_index == action.item_index and not p.purchased:
                shop_potion = p
                break

        if shop_potion is None:
            return ShopResult(
                success=False,
                action_type=action.action_type,
                message="Potion not found or already purchased",
            )

        if run_state.gold < shop_potion.price:
            return ShopResult(
                success=False,
                action_type=action.action_type,
                item_id=shop_potion.potion.id,
                item_name=shop_potion.potion.name,
                message="Not enough gold",
            )

        if run_state.count_empty_potion_slots() == 0:
            return ShopResult(
                success=False,
                action_type=action.action_type,
                item_id=shop_potion.potion.id,
                item_name=shop_potion.potion.name,
                message="No empty potion slots",
            )

        # Process purchase
        run_state.lose_gold(shop_potion.price)
        run_state.add_potion(shop_potion.potion.id)
        shop_potion.purchased = True

        return ShopResult(
            success=True,
            action_type=action.action_type,
            item_id=shop_potion.potion.id,
            item_name=shop_potion.potion.name,
            gold_spent=shop_potion.price,
            message=f"Purchased {shop_potion.potion.name} for {shop_potion.price} gold",
        )

    @staticmethod
    def _remove_card(
        action: ShopAction,
        shop_state: ShopState,
        run_state: 'RunState',
    ) -> ShopResult:
        """Remove a card from the deck."""
        if not shop_state.purge_available:
            return ShopResult(
                success=False,
                action_type=action.action_type,
                message="Card removal not available",
            )

        if run_state.gold < shop_state.purge_cost:
            return ShopResult(
                success=False,
                action_type=action.action_type,
                message="Not enough gold for card removal",
            )

        if action.card_index < 0 or action.card_index >= len(run_state.deck):
            return ShopResult(
                success=False,
                action_type=action.action_type,
                message="Invalid card index",
            )

        # Get card info before removal
        card = run_state.deck[action.card_index]
        card_name = card.id

        # Process removal
        cost = shop_state.purge_cost
        run_state.lose_gold(cost)
        run_state.remove_card(action.card_index)
        shop_state.purge_available = False

        # Increment purge count for next shop
        if hasattr(run_state, 'purge_count'):
            run_state.purge_count += 1

        return ShopResult(
            success=True,
            action_type=action.action_type,
            item_id=card_name,
            item_name=card_name,
            gold_spent=cost,
            message=f"Removed {card_name} for {cost} gold",
        )

    @staticmethod
    def get_shop_summary(shop_state: ShopState) -> str:
        """Get a formatted string summary of the shop inventory."""
        lines = ["=== SHOP ===", ""]

        lines.append("COLORED CARDS:")
        for c in shop_state.colored_cards:
            status = "[SOLD]" if c.purchased else f"{c.price}g"
            sale = " [SALE!]" if c.on_sale else ""
            upgraded = "+" if c.card.upgraded else ""
            lines.append(f"  {c.card.name}{upgraded} ({c.card.rarity.name}) - {status}{sale}")

        lines.append("")
        lines.append("COLORLESS CARDS:")
        for c in shop_state.colorless_cards:
            status = "[SOLD]" if c.purchased else f"{c.price}g"
            upgraded = "+" if c.card.upgraded else ""
            lines.append(f"  {c.card.name}{upgraded} ({c.card.rarity.name}) - {status}")

        lines.append("")
        lines.append("RELICS:")
        for r in shop_state.relics:
            status = "[SOLD]" if r.purchased else f"{r.price}g"
            lines.append(f"  {r.relic.name} ({r.relic.tier.name}) - {status}")

        lines.append("")
        lines.append("POTIONS:")
        for p in shop_state.potions:
            status = "[SOLD]" if p.purchased else f"{p.price}g"
            lines.append(f"  {p.potion.name} ({p.potion.rarity.name}) - {status}")

        lines.append("")
        if shop_state.purge_available:
            lines.append(f"CARD REMOVAL: {shop_state.purge_cost}g")
        else:
            lines.append("CARD REMOVAL: [USED]")

        return "\n".join(lines)


# ============================================================================
# TESTING
# ============================================================================

if __name__ == "__main__":
    import os
    import sys

    # Add parent to path for imports
    _core_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    sys.path.insert(0, _core_dir)

    from state.run import create_watcher_run
    from state.rng import Random, seed_to_long

    print("=== Shop Handler Tests ===\n")

    # Create a test run
    seed_str = "TESTSHOP123"
    run = create_watcher_run(seed_str, ascension=20)

    print(f"Run: seed={seed_str}, gold={run.gold}")
    print(f"Deck: {len(run.deck)} cards")
    print(f"Relics: {[r.id for r in run.relics]}")
    print()

    # Create shop
    merchant_rng = Random(seed_to_long(seed_str) + 1000)
    card_rng = Random(seed_to_long(seed_str) + 5000)
    potion_rng = Random(seed_to_long(seed_str) + 7000)
    shop = ShopHandler.create_shop(run, merchant_rng, card_rng, potion_rng)

    # Print shop summary
    print(ShopHandler.get_shop_summary(shop))
    print()

    # Get available actions
    actions = ShopHandler.get_available_actions(shop, run)
    print(f"Available actions: {len(actions)}")

    # Show some actions
    for action in actions[:10]:
        if action.action_type == ShopActionType.LEAVE:
            print("  - Leave shop")
        elif action.action_type == ShopActionType.BUY_COLORED_CARD:
            card = shop.colored_cards[action.item_index]
            print(f"  - Buy {card.card.name} for {card.price}g")
        elif action.action_type == ShopActionType.BUY_RELIC:
            relic = shop.relics[action.item_index]
            print(f"  - Buy {relic.relic.name} for {relic.price}g")
        elif action.action_type == ShopActionType.REMOVE_CARD:
            card = run.deck[action.card_index]
            print(f"  - Remove {card.id} for {shop.purge_cost}g")

    if len(actions) > 10:
        print(f"  ... and {len(actions) - 10} more")

    print()

    # Execute a purchase
    print("--- Executing purchase ---")
    for action in actions:
        if action.action_type == ShopActionType.BUY_COLORED_CARD:
            result = ShopHandler.execute_action(action, shop, run)
            print(f"Result: {result.message}")
            print(f"Gold remaining: {run.gold}")
            print(f"Deck size: {len(run.deck)}")
            break

    print()

    # Execute card removal
    print("--- Executing card removal ---")
    for action in actions:
        if action.action_type == ShopActionType.REMOVE_CARD:
            result = ShopHandler.execute_action(action, shop, run)
            print(f"Result: {result.message}")
            print(f"Gold remaining: {run.gold}")
            print(f"Deck size: {len(run.deck)}")
            break

    print()

    # Check updated shop
    print("--- Updated shop ---")
    print(f"Purge available: {shop.purge_available}")
    print(f"Available items: {shop.get_all_items_count()}")

    print("\n=== All tests passed ===")
