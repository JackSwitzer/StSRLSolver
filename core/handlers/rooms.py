"""
Room Handlers for Slay the Spire Python Recreation

Handles all non-combat room interactions:
- EventHandler: Event selection, choice filtering, and outcome application
- ShopHandler: Shop generation, purchases, and card removal
- RestHandler: Rest site options (rest, upgrade, dig, lift, recall, toke)
- TreasureHandler: Chest opening and key mechanics
- RewardHandler: Combat rewards (gold, potions, cards)

Each handler takes RunState and modifies it in place, using the appropriate
RNG streams for determinism.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import List, Optional, Dict, Set, Tuple, Any, TYPE_CHECKING

from ..state.run import RunState, CardInstance
from ..state.rng import Random, GameRNG
from ..content.events import (
    Event, EventChoice, Outcome, OutcomeType, Act,
    EXORDIUM_EVENTS, CITY_EVENTS, BEYOND_EVENTS, SHRINE_EVENTS,
    get_events_for_act, calculate_outcome_value,
)
from ..generation.rewards import (
    ShopInventory, RewardState,
    generate_shop_inventory, generate_card_rewards, generate_colorless_card_rewards,
    generate_gold_reward, generate_relic_reward, generate_elite_relic_reward,
    check_potion_drop, generate_potion_reward, RelicTier, get_relic,
)
from ..content.cards import Card, CardRarity
from ..content.potions import Potion


# ============================================================================
# RESULT DATACLASSES
# ============================================================================

@dataclass
class EventResult:
    """Result of applying an event choice."""
    event_id: str
    choice_idx: int
    outcomes_applied: List[str] = field(default_factory=list)
    hp_change: int = 0
    gold_change: int = 0
    cards_gained: List[str] = field(default_factory=list)
    cards_removed: List[str] = field(default_factory=list)
    relics_gained: List[str] = field(default_factory=list)
    curses_gained: List[str] = field(default_factory=list)
    requires_card_selection: bool = False
    card_selection_type: Optional[str] = None  # "remove", "upgrade", "transform"
    combat_triggered: bool = False


@dataclass
class ShopResult:
    """Result of a shop transaction."""
    item_type: str  # "card", "relic", "potion", "purge"
    item_id: str
    gold_spent: int
    success: bool
    message: str = ""


@dataclass
class RestResult:
    """Result of a rest site action."""
    action: str  # "rest", "upgrade", "dig", "lift", "recall", "toke"
    hp_healed: int = 0
    card_upgraded: Optional[str] = None
    relic_gained: Optional[str] = None
    strength_gained: int = 0
    max_hp_restored: int = 0
    card_removed: Optional[str] = None
    dream_catcher_triggered: bool = False


@dataclass
class TreasureResult:
    """Result of opening a treasure chest."""
    relic_id: str
    relic_name: str
    curse_added: Optional[str] = None
    sapphire_key_taken: bool = False


@dataclass
class CombatRewards:
    """Combat reward options."""
    gold: int = 0
    potion: Optional[Potion] = None
    card_choices: List[Card] = field(default_factory=list)
    relic: Optional[Any] = None
    emerald_key_available: bool = False


@dataclass
class RewardResult:
    """Result of taking a reward."""
    reward_type: str  # "gold", "potion", "card", "relic", "key"
    item_id: Optional[str] = None
    amount: int = 0
    success: bool = True
    message: str = ""


# ============================================================================
# EVENT HANDLER
# ============================================================================

class EventHandler:
    """
    Handles event room interactions.

    Responsibilities:
    - Select random event from act pool
    - Filter available choices based on requirements
    - Apply outcome effects to run state
    """

    # Curses that can be obtained from events
    CURSE_CARDS = [
        "Regret", "Doubt", "Pain", "Parasite", "Shame",
        "Decay", "Writhe", "Injury", "Normality"
    ]

    @staticmethod
    def get_event(run_state: RunState, event_rng: Random) -> Optional[Event]:
        """
        Select a random event from the current act's event pool.

        Args:
            run_state: Current run state
            event_rng: Event RNG stream

        Returns:
            Selected Event, or None if no events available
        """
        # Map act number to Act enum
        act_map = {1: Act.ACT_1, 2: Act.ACT_2, 3: Act.ACT_3}
        act = act_map.get(run_state.act, Act.ACT_1)

        # Get event pool for this act
        event_pool = get_events_for_act(act)

        if not event_pool:
            return None

        # Filter events based on conditions
        available_events = []
        for event_id, event in event_pool.items():
            if EventHandler._event_is_available(event, run_state):
                available_events.append(event)

        if not available_events:
            return None

        # Select random event
        idx = event_rng.random(len(available_events) - 1)
        return available_events[idx]

    @staticmethod
    def _event_is_available(event: Event, run_state: RunState) -> bool:
        """Check if an event meets its appearance conditions."""
        # Check floor restrictions
        if event.min_floor is not None and run_state.floor < event.min_floor:
            return False
        if event.max_floor is not None and run_state.floor > event.max_floor:
            return False

        # Check relic requirement
        if event.requires_relic is not None:
            if not run_state.has_relic(event.requires_relic):
                return False

        # Check curse requirement
        if event.requires_curse_in_deck:
            has_curse = any(
                c.id in EventHandler.CURSE_CARDS or c.id == "AscendersBane"
                for c in run_state.deck
            )
            if not has_curse:
                return False

        return True

    @staticmethod
    def get_choices(event: Event, run_state: RunState) -> List[EventChoice]:
        """
        Get available choices for an event, filtered by requirements.

        Args:
            event: The event to get choices for
            run_state: Current run state

        Returns:
            List of EventChoice objects that are available
        """
        available = []

        for choice in event.choices:
            if EventHandler._choice_is_available(choice, run_state):
                available.append(choice)

        return available

    @staticmethod
    def _choice_is_available(choice: EventChoice, run_state: RunState) -> bool:
        """Check if a choice meets its requirements."""
        # Gold requirement
        if choice.requires_gold is not None:
            # Adjust for A15+ if event has ascension modifier
            required = choice.requires_gold
            if run_state.ascension >= 15:
                # Some events have higher gold costs at A15+
                required = int(required * 1.25)  # Approximate
            if run_state.gold < required:
                return False

        # Relic requirement
        if choice.requires_relic is not None:
            if not run_state.has_relic(choice.requires_relic):
                return False

        # Min HP requirement
        if choice.requires_min_hp is not None:
            if run_state.current_hp < choice.requires_min_hp:
                return False

        # Upgradable cards requirement
        if choice.requires_upgradable_cards:
            upgradeable = run_state.get_upgradeable_cards()
            if not upgradeable:
                return False

        # Removable cards requirement
        if choice.requires_removable_cards:
            removable = run_state.get_removable_cards()
            if not removable:
                return False

        # Non-basic card requirement
        if choice.requires_non_basic_card:
            basic_ids = {"Strike_P", "Defend_P", "Eruption", "Vigilance", "AscendersBane"}
            has_non_basic = any(c.id not in basic_ids for c in run_state.deck)
            if not has_non_basic:
                return False

        # Card type requirement
        if choice.requires_card_type is not None:
            # Would need card type info - for now assume available
            pass

        # Curse requirement
        if choice.requires_curse:
            has_curse = any(c.id in EventHandler.CURSE_CARDS for c in run_state.deck)
            if not has_curse:
                return False

        # Potion requirement
        if choice.requires_potion:
            if run_state.count_potions() == 0:
                return False

        return True

    @staticmethod
    def apply_choice(
        event: Event,
        choice_idx: int,
        run_state: RunState,
        event_rng: Random,
        card_idx: Optional[int] = None,
    ) -> EventResult:
        """
        Apply the outcomes of an event choice to the run state.

        Args:
            event: The event being resolved
            choice_idx: Index of the chosen option
            run_state: Run state to modify
            event_rng: Event RNG stream
            card_idx: Optional card index for remove/upgrade/transform

        Returns:
            EventResult with details of what happened
        """
        result = EventResult(event_id=event.id, choice_idx=choice_idx)

        if choice_idx >= len(event.choices):
            return result

        choice = event.choices[choice_idx]

        for outcome in choice.outcomes:
            EventHandler._apply_outcome(outcome, run_state, event_rng, result, card_idx)

        return result

    @staticmethod
    def _apply_outcome(
        outcome: Outcome,
        run_state: RunState,
        event_rng: Random,
        result: EventResult,
        card_idx: Optional[int] = None,
    ) -> None:
        """Apply a single outcome effect."""
        outcome_type = outcome.type

        if outcome_type == OutcomeType.HP_CHANGE:
            value = calculate_outcome_value(
                outcome, run_state.max_hp, run_state.current_hp, run_state.ascension
            )
            if value > 0:
                run_state.heal(value)
            elif value < 0:
                run_state.damage(-value)
            result.hp_change += value
            result.outcomes_applied.append(f"HP: {value:+d}")

        elif outcome_type == OutcomeType.MAX_HP_CHANGE:
            value = calculate_outcome_value(
                outcome, run_state.max_hp, run_state.current_hp, run_state.ascension
            )
            if value > 0:
                run_state.gain_max_hp(value)
            elif value < 0:
                run_state.lose_max_hp(-value)
            result.outcomes_applied.append(f"Max HP: {value:+d}")

        elif outcome_type == OutcomeType.GOLD_CHANGE:
            value = outcome.value if outcome.value is not None else 0
            if outcome.random:
                # Random gold amounts typically have variance
                variance = abs(value) // 4
                value = event_rng.random_range(value - variance, value + variance)

            if value > 0:
                run_state.add_gold(value)
            elif value < 0:
                run_state.lose_gold(-value)
            result.gold_change += value
            result.outcomes_applied.append(f"Gold: {value:+d}")

        elif outcome_type == OutcomeType.CARD_GAIN:
            if outcome.card_id:
                # Specific card
                for _ in range(outcome.count):
                    run_state.add_card(outcome.card_id)
                    result.cards_gained.append(outcome.card_id)
            else:
                # Random card would be handled elsewhere
                result.outcomes_applied.append(f"Gain {outcome.count} cards")

        elif outcome_type == OutcomeType.RELIC_GAIN:
            if outcome.relic_id:
                run_state.add_relic(outcome.relic_id)
                result.relics_gained.append(outcome.relic_id)
            elif outcome.random:
                # Generate random relic
                reward_state = RewardState(owned_relics=set(run_state.get_relic_ids()))
                relic = generate_relic_reward(
                    event_rng, RelicTier.COMMON, reward_state,
                    run_state.character, run_state.act
                )
                if relic:
                    run_state.add_relic(relic.id)
                    result.relics_gained.append(relic.id)

        elif outcome_type == OutcomeType.RELIC_LOSE:
            if outcome.relic_id:
                # Remove specific relic
                for i, relic in enumerate(run_state.relics):
                    if relic.id == outcome.relic_id:
                        run_state.relics.pop(i)
                        result.outcomes_applied.append(f"Lost relic: {outcome.relic_id}")
                        break

        elif outcome_type == OutcomeType.CURSE_GAIN:
            curse_id = outcome.card_id
            if curse_id:
                for _ in range(outcome.count):
                    run_state.add_card(curse_id)
                    result.curses_gained.append(curse_id)

        elif outcome_type == OutcomeType.CARD_REMOVE:
            if card_idx is not None:
                removed = run_state.remove_card(card_idx)
                if removed:
                    result.cards_removed.append(removed.id)
            else:
                # Need card selection
                result.requires_card_selection = True
                result.card_selection_type = "remove"

        elif outcome_type == OutcomeType.CARD_UPGRADE:
            if card_idx is not None:
                if run_state.upgrade_card(card_idx):
                    result.outcomes_applied.append(f"Upgraded card at index {card_idx}")
            elif outcome.random:
                # Upgrade random cards
                upgradeable = run_state.get_upgradeable_cards()
                for _ in range(outcome.count):
                    if upgradeable:
                        idx = event_rng.random(len(upgradeable) - 1)
                        card_idx_to_upgrade, _ = upgradeable.pop(idx)
                        run_state.upgrade_card(card_idx_to_upgrade)
            else:
                result.requires_card_selection = True
                result.card_selection_type = "upgrade"

        elif outcome_type == OutcomeType.CARD_TRANSFORM:
            if card_idx is not None:
                removed = run_state.remove_card(card_idx)
                if removed:
                    result.cards_removed.append(removed.id)
                    # Add a random card of same rarity (simplified)
                    reward_state = RewardState()
                    cards = generate_card_rewards(
                        event_rng, reward_state, run_state.act,
                        run_state.character, run_state.ascension,
                        num_cards=1
                    )
                    if cards:
                        run_state.add_card(cards[0].id, cards[0].upgraded)
                        result.cards_gained.append(cards[0].id)
            else:
                result.requires_card_selection = True
                result.card_selection_type = "transform"

        elif outcome_type == OutcomeType.POTION_GAIN:
            count = outcome.count if outcome.count > 0 else 1
            for _ in range(count):
                if run_state.count_empty_potion_slots() > 0:
                    # Generate random potion (already imported at top of module)
                    potion = generate_potion_reward(event_rng, run_state.character)
                    run_state.add_potion(potion.id)
                    result.outcomes_applied.append(f"Gained potion: {potion.id}")

        elif outcome_type == OutcomeType.COMBAT:
            result.combat_triggered = True
            result.outcomes_applied.append("Combat triggered")

        elif outcome_type == OutcomeType.NOTHING:
            result.outcomes_applied.append("No effect")


# ============================================================================
# SHOP HANDLER
# ============================================================================

class ShopHandler:
    """
    Handles shop room interactions.

    Responsibilities:
    - Generate shop inventory
    - Filter purchasable items by gold
    - Process purchases
    - Handle card removal (purge)
    """

    @staticmethod
    def generate_shop(run_state: RunState, merchant_rng: Random) -> ShopInventory:
        """
        Generate a shop inventory for the current floor.

        Args:
            run_state: Current run state
            merchant_rng: Merchant RNG stream

        Returns:
            ShopInventory with all items and prices
        """
        reward_state = RewardState(owned_relics=set(run_state.get_relic_ids()))

        purge_count = getattr(run_state, 'purge_count', 0)

        shop = generate_shop_inventory(
            rng=merchant_rng,
            reward_state=reward_state,
            act=run_state.act,
            player_class=run_state.character,
            ascension=run_state.ascension,
            purge_count=purge_count,
            has_membership_card=run_state.has_relic("MembershipCard"),
            has_the_courier=run_state.has_relic("TheCourier"),
        )

        # A15+ purge cost cap at 175g
        if run_state.ascension >= 15:
            shop.purge_cost = min(75 + 25 * purge_count, 175)

        return shop

    @staticmethod
    def get_purchasable_items(shop: ShopInventory, run_state: RunState) -> Dict[str, List]:
        """
        Get items the player can currently afford.

        Args:
            shop: Shop inventory
            run_state: Current run state

        Returns:
            Dict with "cards", "relics", "potions", "purge" keys
        """
        gold = run_state.gold

        purchasable = {
            "colored_cards": [(c, p) for c, p in shop.colored_cards if p <= gold],
            "colorless_cards": [(c, p) for c, p in shop.colorless_cards if p <= gold],
            "relics": [(r, p) for r, p in shop.relics if p <= gold],
            "potions": [(p, price) for p, price in shop.potions if price <= gold],
            "purge": shop.purge_available and shop.purge_cost <= gold,
        }

        return purchasable

    @staticmethod
    def buy_card(
        shop: ShopInventory,
        card_idx: int,
        run_state: RunState,
        is_colorless: bool = False,
    ) -> ShopResult:
        """
        Purchase a card from the shop.

        Args:
            shop: Shop inventory
            card_idx: Index of card to buy
            run_state: Run state to modify
            is_colorless: Whether buying from colorless section

        Returns:
            ShopResult with transaction details
        """
        card_list = shop.colorless_cards if is_colorless else shop.colored_cards

        if card_idx < 0 or card_idx >= len(card_list):
            return ShopResult(
                item_type="card",
                item_id="",
                gold_spent=0,
                success=False,
                message="Invalid card index"
            )

        card, price = card_list[card_idx]

        if run_state.gold < price:
            return ShopResult(
                item_type="card",
                item_id=card.id,
                gold_spent=0,
                success=False,
                message="Not enough gold"
            )

        # Process purchase
        run_state.lose_gold(price)
        run_state.add_card(card.id, card.upgraded)

        # Remove from shop
        card_list.pop(card_idx)

        return ShopResult(
            item_type="card",
            item_id=card.id,
            gold_spent=price,
            success=True,
            message=f"Purchased {card.name}"
        )

    @staticmethod
    def buy_relic(
        shop: ShopInventory,
        relic_idx: int,
        run_state: RunState,
    ) -> ShopResult:
        """
        Purchase a relic from the shop.

        Args:
            shop: Shop inventory
            relic_idx: Index of relic to buy
            run_state: Run state to modify

        Returns:
            ShopResult with transaction details
        """
        if relic_idx < 0 or relic_idx >= len(shop.relics):
            return ShopResult(
                item_type="relic",
                item_id="",
                gold_spent=0,
                success=False,
                message="Invalid relic index"
            )

        relic, price = shop.relics[relic_idx]

        if run_state.gold < price:
            return ShopResult(
                item_type="relic",
                item_id=relic.id,
                gold_spent=0,
                success=False,
                message="Not enough gold"
            )

        # Process purchase
        run_state.lose_gold(price)
        run_state.add_relic(relic.id)  # add_relic handles on-obtain effects

        # Remove from shop
        shop.relics.pop(relic_idx)

        return ShopResult(
            item_type="relic",
            item_id=relic.id,
            gold_spent=price,
            success=True,
            message=f"Purchased {relic.name}"
        )

    @staticmethod
    def buy_potion(
        shop: ShopInventory,
        potion_idx: int,
        run_state: RunState,
    ) -> ShopResult:
        """
        Purchase a potion from the shop.

        Args:
            shop: Shop inventory
            potion_idx: Index of potion to buy
            run_state: Run state to modify

        Returns:
            ShopResult with transaction details
        """
        if potion_idx < 0 or potion_idx >= len(shop.potions):
            return ShopResult(
                item_type="potion",
                item_id="",
                gold_spent=0,
                success=False,
                message="Invalid potion index"
            )

        potion, price = shop.potions[potion_idx]

        if run_state.gold < price:
            return ShopResult(
                item_type="potion",
                item_id=potion.id,
                gold_spent=0,
                success=False,
                message="Not enough gold"
            )

        if run_state.count_empty_potion_slots() == 0:
            return ShopResult(
                item_type="potion",
                item_id=potion.id,
                gold_spent=0,
                success=False,
                message="No empty potion slots"
            )

        # Process purchase
        run_state.lose_gold(price)
        run_state.add_potion(potion.id)

        # Remove from shop
        shop.potions.pop(potion_idx)

        return ShopResult(
            item_type="potion",
            item_id=potion.id,
            gold_spent=price,
            success=True,
            message=f"Purchased {potion.name}"
        )

    @staticmethod
    def purge_card(
        shop: ShopInventory,
        run_state: RunState,
        card_idx: int,
    ) -> ShopResult:
        """
        Remove a card from deck at the shop.

        Args:
            shop: Shop inventory
            run_state: Run state to modify
            card_idx: Index of card to remove

        Returns:
            ShopResult with transaction details
        """
        if not shop.purge_available:
            return ShopResult(
                item_type="purge",
                item_id="",
                gold_spent=0,
                success=False,
                message="Card removal not available"
            )

        if run_state.gold < shop.purge_cost:
            return ShopResult(
                item_type="purge",
                item_id="",
                gold_spent=0,
                success=False,
                message="Not enough gold"
            )

        if card_idx < 0 or card_idx >= len(run_state.deck):
            return ShopResult(
                item_type="purge",
                item_id="",
                gold_spent=0,
                success=False,
                message="Invalid card index"
            )

        # Process purge
        removed_card = run_state.remove_card(card_idx)
        run_state.lose_gold(shop.purge_cost)
        shop.purge_available = False
        run_state.purge_count = getattr(run_state, 'purge_count', 0) + 1

        return ShopResult(
            item_type="purge",
            item_id=removed_card.id if removed_card else "",
            gold_spent=shop.purge_cost,
            success=True,
            message=f"Removed {removed_card.id if removed_card else 'card'}"
        )


# ============================================================================
# REST HANDLER
# ============================================================================

class RestHandler:
    """
    Handles rest site (campfire) interactions.

    Available actions depend on relics:
    - Rest: Heal 30% max HP (blocked by Coffee Dripper)
    - Smith (Upgrade): Upgrade a card (blocked by Fusion Hammer)
    - Dig: Get random relic (requires Shovel)
    - Lift: Gain 1 permanent strength (requires Girya, max 3 times)
    - Recall: Obtain ruby key (Act 3+, skip rest/upgrade)
    - Toke: Remove a card (requires Peace Pipe)

    Relic Modifiers:
    - Regal Pillow: Rest heals extra 15 HP (flat bonus)
    - Eternal Feather: Heal 3 HP per 5 cards in deck on entering rest site
    - Coffee Dripper: Cannot rest
    - Fusion Hammer: Cannot smith/upgrade
    - Dream Catcher: Get card reward when resting
    """

    REST_HEAL_PERCENT = 0.30
    REGAL_PILLOW_BONUS_HP = 15  # Flat HP bonus, not percentage
    ETERNAL_FEATHER_HEAL_PER_5 = 3  # Heal 3 HP per 5 cards
    GIRYA_MAX_LIFTS = 3

    @staticmethod
    def get_options(run_state: RunState) -> List[str]:
        """
        Get available rest site options based on relics.

        Args:
            run_state: Current run state

        Returns:
            List of available action strings
        """
        options = []

        # Rest is available unless Coffee Dripper or at full HP
        if not run_state.has_relic("Coffee Dripper"):
            if run_state.current_hp < run_state.max_hp:
                options.append("rest")

        # Smith/Upgrade available unless Fusion Hammer
        if not run_state.has_relic("Fusion Hammer"):
            if run_state.get_upgradeable_cards():
                options.append("smith")

        # Dig with Shovel
        if run_state.has_relic("Shovel"):
            options.append("dig")

        # Lift with Girya (max 3 times)
        girya = run_state.get_relic("Girya")
        if girya:
            counter = girya.counter if girya.counter >= 0 else 0
            if counter < RestHandler.GIRYA_MAX_LIFTS:
                options.append("lift")

        # Toke with Peace Pipe
        if run_state.has_relic("Peace Pipe"):
            if run_state.get_removable_cards():
                options.append("toke")

        # Recall for ruby key (Act 3, key not obtained)
        if run_state.act == 3 and not run_state.has_ruby_key:
            options.append("recall")

        return options

    @staticmethod
    def on_enter_rest_site(run_state: RunState) -> int:
        """
        Called when entering a rest site. Applies Eternal Feather healing.

        Args:
            run_state: Run state to modify

        Returns:
            HP healed from Eternal Feather (0 if not present)
        """
        if run_state.has_relic("Eternal Feather"):
            deck_size = len(run_state.deck)
            heal_amount = (deck_size // 5) * RestHandler.ETERNAL_FEATHER_HEAL_PER_5
            if heal_amount > 0:
                old_hp = run_state.current_hp
                run_state.heal(heal_amount)
                return run_state.current_hp - old_hp
        return 0

    @staticmethod
    def rest(run_state: RunState) -> RestResult:
        """
        Rest at campfire - heal 30% max HP (rounded down).

        Relic modifiers:
        - Regal Pillow: +15 HP flat
        - Coffee Dripper: Blocks this action (checked in get_options)

        Args:
            run_state: Run state to modify

        Returns:
            RestResult with heal amount
        """
        if run_state.has_relic("Coffee Dripper"):
            return RestResult(action="rest", hp_healed=0)

        # Base heal: 30% of max HP, rounded down
        heal_amount = int(run_state.max_hp * RestHandler.REST_HEAL_PERCENT)

        # Regal Pillow: +15 HP flat
        if run_state.has_relic("Regal Pillow"):
            heal_amount += RestHandler.REGAL_PILLOW_BONUS_HP

        old_hp = run_state.current_hp
        run_state.heal(heal_amount)
        actual_heal = run_state.current_hp - old_hp

        result = RestResult(
            action="rest",
            hp_healed=actual_heal
        )

        # Dream Catcher: flag that a card reward should be generated
        if run_state.has_relic("Dream Catcher"):
            result.dream_catcher_triggered = True

        return result

    @staticmethod
    def smith(run_state: RunState, card_idx: int) -> RestResult:
        """
        Smith (upgrade) a card at the campfire.

        Args:
            run_state: Run state to modify
            card_idx: Index of card to upgrade

        Returns:
            RestResult with upgraded card info
        """
        if run_state.has_relic("Fusion Hammer"):
            return RestResult(action="smith")

        if card_idx < 0 or card_idx >= len(run_state.deck):
            return RestResult(action="smith")

        card = run_state.deck[card_idx]
        card_id = card.id

        if run_state.upgrade_card(card_idx):
            return RestResult(
                action="smith",
                card_upgraded=card_id
            )

        return RestResult(action="smith")

    # Alias for backwards compatibility
    @staticmethod
    def upgrade(run_state: RunState, card_idx: int) -> RestResult:
        """Alias for smith() for backwards compatibility."""
        return RestHandler.smith(run_state, card_idx)

    @staticmethod
    def dig(run_state: RunState, relic_rng: Random) -> RestResult:
        """
        Dig for a relic with the Shovel.

        Args:
            run_state: Run state to modify
            relic_rng: Relic RNG stream

        Returns:
            RestResult with relic info
        """
        if not run_state.has_relic("Shovel"):
            return RestResult(action="dig")

        # Generate random relic (dig uses standard tier roll)
        reward_state = RewardState(owned_relics=set(run_state.get_relic_ids()))
        relic = generate_relic_reward(
            relic_rng, RelicTier.COMMON, reward_state,
            run_state.character, run_state.act
        )

        if relic:
            run_state.add_relic(relic.id)
            return RestResult(
                action="dig",
                relic_gained=relic.id
            )

        return RestResult(action="dig")

    @staticmethod
    def lift(run_state: RunState) -> RestResult:
        """
        Lift with Girya - gain 1 permanent strength.

        Args:
            run_state: Run state to modify

        Returns:
            RestResult with strength gain
        """
        girya = run_state.get_relic("Girya")
        if not girya:
            return RestResult(action="lift")

        # Initialize counter if needed
        if girya.counter == -1:
            girya.counter = 0

        if girya.counter >= RestHandler.GIRYA_MAX_LIFTS:
            return RestResult(action="lift", strength_gained=0)

        girya.counter += 1

        return RestResult(
            action="lift",
            strength_gained=1
        )

    @staticmethod
    def toke(run_state: RunState, card_idx: int) -> RestResult:
        """
        Remove a card with Peace Pipe.

        Args:
            run_state: Run state to modify
            card_idx: Index of card to remove

        Returns:
            RestResult with removed card info
        """
        if not run_state.has_relic("Peace Pipe"):
            return RestResult(action="toke")

        if card_idx < 0 or card_idx >= len(run_state.deck):
            return RestResult(action="toke")

        removed = run_state.remove_card(card_idx)

        return RestResult(
            action="toke",
            card_removed=removed.id if removed else None
        )

    @staticmethod
    def recall(run_state: RunState) -> RestResult:
        """
        Take the Ruby Key at a rest site in Act 3.

        This skips the normal rest/smith action to obtain the key.

        Args:
            run_state: Run state to modify

        Returns:
            RestResult
        """
        if run_state.has_ruby_key:
            return RestResult(action="recall")

        if run_state.act != 3:
            return RestResult(action="recall")

        run_state.obtain_ruby_key()

        return RestResult(
            action="recall",
            max_hp_restored=0  # Ruby key doesn't restore HP
        )

    @staticmethod
    def get_dream_catcher_reward(
        run_state: RunState,
        card_rng: Random,
    ) -> List[Any]:
        """
        Generate card reward for Dream Catcher after resting.

        Args:
            run_state: Current run state
            card_rng: Card RNG stream

        Returns:
            List of Card objects to choose from
        """
        if not run_state.has_relic("Dream Catcher"):
            return []

        reward_state = RewardState()
        return generate_card_rewards(
            card_rng, reward_state,
            act=run_state.act,
            player_class=run_state.character,
            ascension=run_state.ascension,
            room_type="normal",
            num_cards=3,
        )


# ============================================================================
# TREASURE HANDLER
# ============================================================================

class ChestType(Enum):
    """Types of treasure chests."""
    SMALL = "Small"
    MEDIUM = "Medium"
    LARGE = "Large"


@dataclass
class ChestReward:
    """Result of opening a chest."""
    chest_type: ChestType
    relic_tier: str  # "COMMON", "UNCOMMON", "RARE"
    relic_id: str
    relic_name: str
    gold_amount: int = 0
    curse_added: Optional[str] = None
    sapphire_key_taken: bool = False
    matryoshka_relics: Optional[List[str]] = None  # Additional relics from Matryoshka


class TreasureHandler:
    """
    Handles treasure room (chest) interactions.

    Chest Types & Relic Tier Probabilities:
    - Small chest:  Common (50%), Uncommon (33%), Rare (17%)
    - Medium chest: Uncommon (75%), Rare (25%)
    - Large chest:  Always Rare (100%)

    Relic Interactions:
    - Cursed Key: Get a random curse when taking a relic from a chest
    - Matryoshka: Get 2 relics instead of 1 from first 2 non-boss chests
    - Sapphire Key: Can skip relic to obtain the key (Act 3)

    Chest Type Roll (treasureRng 0-99):
    - < 50: Small chest
    - < 83: Medium chest
    - else: Large chest
    """

    # Chest type thresholds (roll 0-99)
    SMALL_THRESHOLD = 50
    MEDIUM_THRESHOLD = 83

    # Relic tier chances by chest type
    CHEST_RELIC_CHANCES = {
        ChestType.SMALL: {"common": 50, "uncommon": 83},   # 50% common, 33% uncommon, 17% rare
        ChestType.MEDIUM: {"common": 0, "uncommon": 75},   # 0% common, 75% uncommon, 25% rare
        ChestType.LARGE: {"common": 0, "uncommon": 0},     # 100% rare
    }

    @staticmethod
    def determine_chest_type(treasure_rng: Random) -> ChestType:
        """
        Determine the type of chest using treasureRng.

        Args:
            treasure_rng: Treasure RNG stream

        Returns:
            ChestType enum value
        """
        roll = treasure_rng.random_int(99)

        if roll < TreasureHandler.SMALL_THRESHOLD:
            return ChestType.SMALL
        elif roll < TreasureHandler.MEDIUM_THRESHOLD:
            return ChestType.MEDIUM
        else:
            return ChestType.LARGE

    @staticmethod
    def roll_relic_tier(treasure_rng: Random, chest_type: ChestType) -> str:
        """
        Roll the relic tier based on chest type.

        Args:
            treasure_rng: Treasure RNG stream
            chest_type: Type of chest being opened

        Returns:
            Relic tier string: "COMMON", "UNCOMMON", or "RARE"
        """
        chances = TreasureHandler.CHEST_RELIC_CHANCES[chest_type]
        roll = treasure_rng.random_int(99)

        if roll < chances["common"]:
            return "COMMON"
        elif roll < chances["uncommon"]:
            return "UNCOMMON"
        else:
            return "RARE"

    @staticmethod
    def open_chest(
        run_state: RunState,
        treasure_rng: Random,
        relic_rng: Random,
        take_sapphire_key: bool = False,
        chest_type: Optional[ChestType] = None,
    ) -> ChestReward:
        """
        Open a treasure chest and get the reward.

        Process:
        1. Determine chest type (if not specified)
        2. Roll relic tier based on chest type
        3. Get relic from appropriate pool
        4. Handle Matryoshka (2 relics from small/medium chests)
        5. Handle Cursed Key (add curse)
        6. Handle Sapphire Key (skip relic for key)

        Args:
            run_state: Run state to modify
            treasure_rng: Treasure RNG stream (for chest type and tier rolls)
            relic_rng: Relic RNG stream (for relic selection)
            take_sapphire_key: If True, take key instead of relic
            chest_type: Override chest type (if known from map generation)

        Returns:
            ChestReward with all reward details
        """
        # Step 1: Determine chest type
        if chest_type is None:
            chest_type = TreasureHandler.determine_chest_type(treasure_rng)

        # Step 2: Roll relic tier
        relic_tier = TreasureHandler.roll_relic_tier(treasure_rng, chest_type)

        # Map tier string to RelicTier enum
        tier_map = {
            "COMMON": RelicTier.COMMON,
            "UNCOMMON": RelicTier.UNCOMMON,
            "RARE": RelicTier.RARE,
        }

        # Step 3: Get relic from pool
        reward_state = RewardState(owned_relics=set(run_state.get_relic_ids()))
        relic = generate_relic_reward(
            relic_rng, tier_map[relic_tier], reward_state,
            run_state.character, run_state.act
        )

        result = ChestReward(
            chest_type=chest_type,
            relic_tier=relic_tier,
            relic_id=relic.id if relic else "Circlet",
            relic_name=relic.name if relic else "Circlet",
        )

        # Handle Sapphire Key (Act 3)
        if take_sapphire_key and not run_state.has_sapphire_key and run_state.act == 3:
            run_state.obtain_sapphire_key()
            result.sapphire_key_taken = True
            return result

        # Take the relic
        if relic:
            run_state.add_relic(relic.id)

        # Step 4: Handle Matryoshka (2 relics from first 2 non-boss chests)
        if run_state.has_relic("Matryoshka"):
            matryoshka = run_state.get_relic("Matryoshka")
            if matryoshka and (matryoshka.counter < 2 or matryoshka.counter == -1):
                if matryoshka.counter == -1:
                    matryoshka.counter = 0

                if chest_type in (ChestType.SMALL, ChestType.MEDIUM, ChestType.LARGE):
                    # Get a second relic
                    matryoshka.counter += 1
                    reward_state.owned_relics.add(relic.id if relic else "Circlet")
                    second_tier = TreasureHandler.roll_relic_tier(treasure_rng, chest_type)
                    second_relic = generate_relic_reward(
                        relic_rng, tier_map[second_tier], reward_state,
                        run_state.character, run_state.act
                    )
                    if second_relic:
                        run_state.add_relic(second_relic.id)
                        result.matryoshka_relics = [second_relic.id]

        # Step 5: Handle Cursed Key (adds curse when taking relic)
        if run_state.has_relic("Cursed Key") and not result.sapphire_key_taken:
            curse = TreasureHandler.apply_cursed_key(run_state, relic_rng)
            result.curse_added = curse

        return result

    @staticmethod
    def apply_cursed_key(run_state: RunState, relic_rng: Random) -> str:
        """
        Apply Cursed Key effect - add a random curse.

        Args:
            run_state: Run state to modify
            relic_rng: RNG for curse selection

        Returns:
            ID of the curse added
        """
        # Basic curses that can be added (not special event-only curses)
        curses = ["Pain", "Parasite", "Clumsy", "Decay", "Doubt", "Injury", "Normality", "Regret", "Shame", "Writhe"]

        curse_idx = relic_rng.random(len(curses) - 1)
        curse_id = curses[curse_idx]

        run_state.add_card(curse_id)

        return curse_id

    @staticmethod
    def get_treasure_actions(run_state: RunState) -> List[str]:
        """
        Get available actions in a treasure room.

        Args:
            run_state: Current run state

        Returns:
            List of available action strings
        """
        actions = ["open"]

        # Sapphire key option in Act 3
        if run_state.act == 3 and not run_state.has_sapphire_key:
            actions.append("take_sapphire_key")

        return actions


# ============================================================================
# NEOW HANDLER
# ============================================================================

class NeowBlessingType(Enum):
    """Types of Neow blessings available."""
    # Common blessings (no drawback)
    HUNDRED_GOLD = "hundred_gold"
    THREE_CARDS = "three_cards"
    RANDOM_COMMON_RELIC = "random_common_relic"
    TEN_PERCENT_HP_BONUS = "ten_percent_hp_bonus"
    THREE_ENEMY_KILL = "three_enemy_kill"
    UPGRADE_CARD = "upgrade_card"
    ONE_RANDOM_RARE_CARD = "one_random_rare_card"
    REMOVE_CARD = "remove_card"
    TRANSFORM_CARD = "transform_card"
    THREE_POTIONS = "three_potions"

    # Rare blessings (with drawbacks)
    RANDOM_COLORLESS_RARE = "random_colorless_rare"
    REMOVE_TWO = "remove_two"
    TRANSFORM_TWO = "transform_two"
    RANDOM_RARE_RELIC = "random_rare_relic"
    BOSS_SWAP = "boss_swap"


class NeowDrawbackType(Enum):
    """Types of Neow drawbacks."""
    NONE = "none"
    LOSE_GOLD = "lose_gold"
    LOSE_HP = "lose_hp"
    GAIN_CURSE = "gain_curse"
    LOSE_MAX_HP = "lose_max_hp"


@dataclass
class NeowBlessing:
    """A Neow blessing option."""
    blessing_type: NeowBlessingType
    description: str
    drawback_type: NeowDrawbackType = NeowDrawbackType.NONE
    drawback_description: str = ""
    drawback_value: int = 0


@dataclass
class NeowResult:
    """Result of choosing a Neow blessing."""
    blessing_type: NeowBlessingType
    blessing_applied: str = ""
    drawback_applied: str = ""
    gold_change: int = 0
    hp_change: int = 0
    max_hp_change: int = 0
    relics_gained: List[str] = field(default_factory=list)
    cards_gained: List[str] = field(default_factory=list)
    cards_removed: List[str] = field(default_factory=list)
    cards_transformed: List[str] = field(default_factory=list)
    cards_upgraded: List[str] = field(default_factory=list)
    curse_added: Optional[str] = None
    requires_card_selection: bool = False
    card_selection_type: Optional[str] = None  # "upgrade", "remove", "transform", "choose"
    card_choices: List[Any] = field(default_factory=list)
    potions_gained: List[str] = field(default_factory=list)


class NeowHandler:
    """
    Handles Neow's blessing selection at the start of a run.

    On first run: Fixed set of safe options
    On subsequent runs: Options based on previous run score

    Blessing Categories:
    1. Simple blessings (no drawback)
    2. Card manipulation blessings
    3. Rare blessings (require a drawback)

    Drawbacks:
    - Lose gold (varies by ascension)
    - Lose 10% current HP
    - Gain a curse
    - Lose 10% max HP (for best rewards)
    """

    # Gold amounts
    HUNDRED_GOLD_AMOUNT = 100
    GOLD_LOSS_AMOUNT = 100  # For "lose all gold" drawback it's variable

    # HP/Max HP changes
    HP_BONUS_PERCENT = 0.10  # +10% max HP
    HP_LOSS_PERCENT = 0.10   # -10% current HP (drawback)
    MAX_HP_LOSS_PERCENT = 0.10  # -10% max HP (severe drawback)

    # Card counts
    THREE_ENEMY_KILL_HP = 1  # Set first 3 combat enemies to 1 HP

    # Basic curses for drawback
    BASIC_CURSES = ["Regret", "Doubt", "Pain", "Parasite", "Shame", "Decay", "Writhe"]

    @staticmethod
    def get_first_run_options() -> List[NeowBlessing]:
        """
        Get Neow options for a first-time player (no previous run).

        Returns safe, easy-to-understand options.
        """
        return [
            NeowBlessing(
                NeowBlessingType.THREE_ENEMY_KILL,
                "Enemies in your first 3 combats have 1 HP"
            ),
            NeowBlessing(
                NeowBlessingType.HUNDRED_GOLD,
                "Gain 100 Gold"
            ),
            NeowBlessing(
                NeowBlessingType.TEN_PERCENT_HP_BONUS,
                "Gain 10% Max HP"
            ),
            NeowBlessing(
                NeowBlessingType.THREE_CARDS,
                "Choose a card to add to your deck"
            ),
        ]

    @staticmethod
    def get_blessing_options(
        neow_rng: Random,
        previous_score: int = 0,
        is_first_run: bool = False,
    ) -> List[NeowBlessing]:
        """
        Get 4 Neow blessing options based on previous run score.

        Args:
            neow_rng: Neow RNG stream
            previous_score: Score from previous run (affects rare options)
            is_first_run: If True, return safe beginner options

        Returns:
            List of 4 NeowBlessing options
        """
        if is_first_run:
            return NeowHandler.get_first_run_options()

        options = []

        # Option 1: Simple blessing (no drawback)
        simple_options = [
            NeowBlessing(NeowBlessingType.HUNDRED_GOLD, "Gain 100 Gold"),
            NeowBlessing(NeowBlessingType.THREE_CARDS, "Choose a card to add to your deck"),
            NeowBlessing(NeowBlessingType.RANDOM_COMMON_RELIC, "Obtain a random common relic"),
            NeowBlessing(NeowBlessingType.TEN_PERCENT_HP_BONUS, "Gain 10% Max HP"),
            NeowBlessing(NeowBlessingType.THREE_ENEMY_KILL, "Enemies in your first 3 combats have 1 HP"),
            NeowBlessing(NeowBlessingType.THREE_POTIONS, "Obtain 3 random potions"),
        ]
        idx = neow_rng.random(len(simple_options) - 1)
        options.append(simple_options[idx])

        # Option 2: Card manipulation (moderate, no drawback)
        card_options = [
            NeowBlessing(NeowBlessingType.UPGRADE_CARD, "Upgrade a card"),
            NeowBlessing(NeowBlessingType.REMOVE_CARD, "Remove a card from your deck"),
            NeowBlessing(NeowBlessingType.TRANSFORM_CARD, "Transform a card"),
            NeowBlessing(NeowBlessingType.ONE_RANDOM_RARE_CARD, "Obtain a random rare card"),
        ]
        idx = neow_rng.random(len(card_options) - 1)
        options.append(card_options[idx])

        # Option 3: Rare blessing with drawback based on score
        drawback = NeowHandler._select_drawback(neow_rng, previous_score)
        rare_options = [
            NeowBlessing(
                NeowBlessingType.RANDOM_COLORLESS_RARE,
                "Obtain a random rare colorless card",
                drawback[0], drawback[1], drawback[2]
            ),
            NeowBlessing(
                NeowBlessingType.REMOVE_TWO,
                "Remove 2 cards from your deck",
                drawback[0], drawback[1], drawback[2]
            ),
            NeowBlessing(
                NeowBlessingType.TRANSFORM_TWO,
                "Transform 2 cards",
                drawback[0], drawback[1], drawback[2]
            ),
        ]
        idx = neow_rng.random(len(rare_options) - 1)
        options.append(rare_options[idx])

        # Option 4: Boss swap (swap starter relic for random boss relic)
        options.append(NeowBlessing(
            NeowBlessingType.BOSS_SWAP,
            "Swap your starting relic for a random boss relic"
        ))

        return options

    @staticmethod
    def _select_drawback(
        neow_rng: Random,
        previous_score: int,
    ) -> Tuple[NeowDrawbackType, str, int]:
        """
        Select a drawback for rare blessings based on score.

        Higher scores unlock better (less severe) drawbacks.

        Args:
            neow_rng: RNG stream
            previous_score: Previous run score

        Returns:
            Tuple of (drawback_type, description, value)
        """
        drawbacks = [
            (NeowDrawbackType.LOSE_HP, "Lose 10% of your current HP", 10),
            (NeowDrawbackType.GAIN_CURSE, "Gain a curse", 0),
            (NeowDrawbackType.LOSE_GOLD, "Lose all your gold", 0),
            (NeowDrawbackType.LOSE_MAX_HP, "Lose 10% of your Max HP", 10),
        ]

        idx = neow_rng.random(len(drawbacks) - 1)
        return drawbacks[idx]

    @staticmethod
    def apply_blessing(
        run_state: RunState,
        blessing: NeowBlessing,
        neow_rng: Random,
        card_rng: Random,
        relic_rng: Random,
        potion_rng: Random,
        card_selection_idx: Optional[int] = None,
    ) -> NeowResult:
        """
        Apply a Neow blessing to the run state.

        Some blessings require card selection (upgrade, remove, transform).
        These return requires_card_selection=True and must be called again
        with card_selection_idx to complete.

        Args:
            run_state: Run state to modify
            blessing: The blessing to apply
            neow_rng: Neow RNG stream
            card_rng: Card RNG stream
            relic_rng: Relic RNG stream
            potion_rng: Potion RNG stream
            card_selection_idx: Index of selected card (for upgrade/remove/transform)

        Returns:
            NeowResult with all changes
        """
        result = NeowResult(blessing_type=blessing.blessing_type)

        # Apply drawback first
        if blessing.drawback_type != NeowDrawbackType.NONE:
            NeowHandler._apply_drawback(run_state, blessing, neow_rng, result)

        # Apply blessing based on type
        btype = blessing.blessing_type

        if btype == NeowBlessingType.HUNDRED_GOLD:
            run_state.add_gold(NeowHandler.HUNDRED_GOLD_AMOUNT)
            result.gold_change = NeowHandler.HUNDRED_GOLD_AMOUNT
            result.blessing_applied = f"Gained {NeowHandler.HUNDRED_GOLD_AMOUNT} gold"

        elif btype == NeowBlessingType.THREE_CARDS:
            # Generate 3 card choices
            reward_state = RewardState()
            cards = generate_card_rewards(
                card_rng, reward_state,
                act=1, player_class=run_state.character,
                room_type="normal", num_cards=3
            )
            result.card_choices = cards
            result.requires_card_selection = True
            result.card_selection_type = "choose"

        elif btype == NeowBlessingType.RANDOM_COMMON_RELIC:
            reward_state = RewardState(owned_relics=set(run_state.get_relic_ids()))
            relic = generate_relic_reward(
                relic_rng, RelicTier.COMMON, reward_state,
                run_state.character, 1
            )
            if relic:
                run_state.add_relic(relic.id)
                result.relics_gained.append(relic.id)
                result.blessing_applied = f"Obtained {relic.name}"

        elif btype == NeowBlessingType.TEN_PERCENT_HP_BONUS:
            bonus = int(run_state.max_hp * NeowHandler.HP_BONUS_PERCENT)
            bonus = max(1, bonus)  # At least 1 HP
            run_state.gain_max_hp(bonus)
            run_state.heal(bonus)
            result.max_hp_change = bonus
            result.hp_change = bonus
            result.blessing_applied = f"Gained {bonus} Max HP"

        elif btype == NeowBlessingType.THREE_ENEMY_KILL:
            # This flag is stored in run state and checked during combat
            run_state.neow_bonus_first_three_enemies = True
            result.blessing_applied = "First 3 combat enemies will have 1 HP"

        elif btype == NeowBlessingType.UPGRADE_CARD:
            upgradeable = run_state.get_upgradeable_cards()
            if card_selection_idx is not None and 0 <= card_selection_idx < len(run_state.deck):
                card = run_state.deck[card_selection_idx]
                if run_state.upgrade_card(card_selection_idx):
                    result.cards_upgraded.append(card.id)
                    result.blessing_applied = f"Upgraded {card.id}"
            else:
                result.requires_card_selection = True
                result.card_selection_type = "upgrade"

        elif btype == NeowBlessingType.ONE_RANDOM_RARE_CARD:
            # Get a random rare card
            reward_state = RewardState()
            reward_state.card_blizzard.offset = -100  # Force rare
            cards = generate_card_rewards(
                card_rng, reward_state,
                act=1, player_class=run_state.character,
                room_type="normal", num_cards=1
            )
            if cards:
                card = cards[0]
                run_state.add_card(card.id, card.upgraded)
                result.cards_gained.append(card.id)
                result.blessing_applied = f"Obtained {card.name}"

        elif btype == NeowBlessingType.REMOVE_CARD:
            removable = run_state.get_removable_cards()
            if card_selection_idx is not None and 0 <= card_selection_idx < len(run_state.deck):
                removed = run_state.remove_card(card_selection_idx)
                if removed:
                    result.cards_removed.append(removed.id)
                    result.blessing_applied = f"Removed {removed.id}"
            else:
                result.requires_card_selection = True
                result.card_selection_type = "remove"

        elif btype == NeowBlessingType.TRANSFORM_CARD:
            if card_selection_idx is not None and 0 <= card_selection_idx < len(run_state.deck):
                old_card = run_state.deck[card_selection_idx]
                # Transform: remove old, get random new of same rarity
                run_state.remove_card(card_selection_idx)
                reward_state = RewardState()
                cards = generate_card_rewards(
                    card_rng, reward_state,
                    act=1, player_class=run_state.character,
                    room_type="normal", num_cards=1
                )
                if cards:
                    new_card = cards[0]
                    run_state.add_card(new_card.id, new_card.upgraded)
                    result.cards_transformed.append(f"{old_card.id}->{new_card.id}")
                    result.blessing_applied = f"Transformed {old_card.id} into {new_card.id}"
            else:
                result.requires_card_selection = True
                result.card_selection_type = "transform"

        elif btype == NeowBlessingType.THREE_POTIONS:
            for _ in range(3):
                potion = generate_potion_reward(potion_rng, run_state.character)
                if potion and run_state.count_empty_potion_slots() > 0:
                    run_state.add_potion(potion.id)
                    result.potions_gained.append(potion.id)
            result.blessing_applied = f"Obtained {len(result.potions_gained)} potions"

        elif btype == NeowBlessingType.RANDOM_COLORLESS_RARE:
            # Get a random rare colorless card
            cards = generate_colorless_card_rewards(card_rng, num_cards=1)
            # Filter to rare only
            rare_cards = [c for c in cards if c.rarity == CardRarity.RARE]
            if not rare_cards:
                # Generate more until we get a rare
                for _ in range(10):
                    cards = generate_colorless_card_rewards(card_rng, num_cards=1)
                    rare_cards = [c for c in cards if c.rarity == CardRarity.RARE]
                    if rare_cards:
                        break
            if rare_cards:
                card = rare_cards[0]
                run_state.add_card(card.id, card.upgraded)
                result.cards_gained.append(card.id)
                result.blessing_applied = f"Obtained {card.name}"

        elif btype == NeowBlessingType.REMOVE_TWO:
            # Remove 2 cards - requires two selections
            # For simplicity, if card_selection_idx is a list, remove both
            result.requires_card_selection = True
            result.card_selection_type = "remove_two"

        elif btype == NeowBlessingType.TRANSFORM_TWO:
            # Transform 2 cards
            result.requires_card_selection = True
            result.card_selection_type = "transform_two"

        elif btype == NeowBlessingType.RANDOM_RARE_RELIC:
            reward_state = RewardState(owned_relics=set(run_state.get_relic_ids()))
            relic = generate_relic_reward(
                relic_rng, RelicTier.RARE, reward_state,
                run_state.character, 1
            )
            if relic:
                run_state.add_relic(relic.id)
                result.relics_gained.append(relic.id)
                result.blessing_applied = f"Obtained {relic.name}"

        elif btype == NeowBlessingType.BOSS_SWAP:
            # Swap starting relic for boss relic
            starter_relic = run_state.get_starter_relic()
            if starter_relic:
                # Remove starter relic
                run_state.remove_relic(starter_relic)
                result.blessing_applied = f"Swapped {starter_relic}"

                # Get boss relic from pool (first in shuffled boss pool)
                reward_state = RewardState(owned_relics=set(run_state.get_relic_ids()))
                # For boss swap, we get the first boss relic from the pool
                # This is handled specially - we use relicRng but get from boss pool
                boss_relic = generate_relic_reward(
                    relic_rng, RelicTier.BOSS, reward_state,
                    run_state.character, 1
                )
                if boss_relic:
                    run_state.add_relic(boss_relic.id)
                    result.relics_gained.append(boss_relic.id)
                    result.blessing_applied += f" for {boss_relic.name}"

        return result

    @staticmethod
    def _apply_drawback(
        run_state: RunState,
        blessing: NeowBlessing,
        neow_rng: Random,
        result: NeowResult,
    ):
        """
        Apply the drawback of a blessing.

        Args:
            run_state: Run state to modify
            blessing: The blessing with drawback info
            neow_rng: RNG stream
            result: NeowResult to update
        """
        dtype = blessing.drawback_type

        if dtype == NeowDrawbackType.LOSE_GOLD:
            gold_lost = run_state.gold
            run_state.lose_gold(gold_lost)
            result.gold_change = -gold_lost
            result.drawback_applied = f"Lost {gold_lost} gold"

        elif dtype == NeowDrawbackType.LOSE_HP:
            hp_lost = int(run_state.current_hp * NeowHandler.HP_LOSS_PERCENT)
            hp_lost = max(1, hp_lost)
            run_state.damage(hp_lost)
            result.hp_change = -hp_lost
            result.drawback_applied = f"Lost {hp_lost} HP"

        elif dtype == NeowDrawbackType.GAIN_CURSE:
            curse_idx = neow_rng.random(len(NeowHandler.BASIC_CURSES) - 1)
            curse_id = NeowHandler.BASIC_CURSES[curse_idx]
            run_state.add_card(curse_id)
            result.curse_added = curse_id
            result.drawback_applied = f"Gained curse: {curse_id}"

        elif dtype == NeowDrawbackType.LOSE_MAX_HP:
            max_hp_lost = int(run_state.max_hp * NeowHandler.MAX_HP_LOSS_PERCENT)
            max_hp_lost = max(1, max_hp_lost)
            run_state.lose_max_hp(max_hp_lost)
            result.max_hp_change = -max_hp_lost
            result.drawback_applied = f"Lost {max_hp_lost} Max HP"

    @staticmethod
    def get_neow_actions(
        run_state: RunState,
        blessing_options: List[NeowBlessing],
    ) -> List[Tuple[int, str]]:
        """
        Get available Neow actions as (index, description) tuples.

        Args:
            run_state: Current run state
            blessing_options: Available blessings

        Returns:
            List of (blessing_index, description) tuples
        """
        actions = []
        for i, blessing in enumerate(blessing_options):
            desc = blessing.description
            if blessing.drawback_type != NeowDrawbackType.NONE:
                desc += f" ({blessing.drawback_description})"
            actions.append((i, desc))
        return actions


# ============================================================================
# REWARD HANDLER
# ============================================================================

class RewardHandler:
    """
    Handles combat reward collection.

    Responsibilities:
    - Generate combat rewards (gold, potion, cards, relic)
    - Process reward selection
    - Handle emerald key (from burning elite)
    """

    @staticmethod
    def generate_combat_rewards(
        run_state: RunState,
        room_type: str,
        card_rng: Random,
        treasure_rng: Random,
        potion_rng: Random,
        relic_rng: Random,
        is_burning_elite: bool = False,
    ) -> CombatRewards:
        """
        Generate rewards after combat victory.

        Args:
            run_state: Current run state
            room_type: "normal", "elite", or "boss"
            card_rng: Card RNG stream
            treasure_rng: Treasure RNG stream (for gold)
            potion_rng: Potion RNG stream
            relic_rng: Relic RNG stream
            is_burning_elite: If True, offer emerald key

        Returns:
            CombatRewards with all available rewards
        """
        rewards = CombatRewards()

        # Gold reward
        rewards.gold = generate_gold_reward(
            treasure_rng,
            room_type,
            run_state.ascension,
            run_state.has_relic("GoldenIdol")
        )

        # Potion drop check
        reward_state = RewardState()
        reward_state.potion_blizzard.modifier = run_state.potion_blizzard

        dropped, potion = check_potion_drop(
            potion_rng,
            reward_state,
            room_type,
            run_state.has_relic("WhiteBeastStatue"),
            run_state.has_relic("Sozu"),
        )

        if dropped:
            rewards.potion = potion
            run_state.on_potion_drop_check(True)
        else:
            run_state.on_potion_drop_check(False)

        # Card rewards
        card_reward_state = RewardState()
        card_reward_state.card_blizzard.offset = run_state.card_blizzard

        rewards.card_choices = generate_card_rewards(
            card_rng,
            card_reward_state,
            run_state.act,
            run_state.character,
            run_state.ascension,
            room_type,
            num_cards=3,
            has_prismatic_shard=run_state.has_relic("PrismaticShard"),
            has_busted_crown=run_state.has_relic("BustedCrown"),
            has_question_card=run_state.has_relic("QuestionCard"),
            has_nloth_gift=run_state.has_relic("NlothsGift"),
        )

        # Elite relic
        if room_type == "elite":
            rewards.relic = generate_elite_relic_reward(
                relic_rng,
                RewardState(owned_relics=set(run_state.get_relic_ids())),
                run_state.character,
                run_state.act
            )

        # Emerald key for burning elite
        if is_burning_elite and not run_state.has_emerald_key:
            rewards.emerald_key_available = True

        return rewards

    @staticmethod
    def take_card_reward(
        run_state: RunState,
        card: Card,
    ) -> RewardResult:
        """
        Take a card from the card reward options.

        Args:
            run_state: Run state to modify
            card: Card to add to deck

        Returns:
            RewardResult with details
        """
        run_state.add_card(card.id, card.upgraded)

        # Update card blizzard based on rarity
        is_rare = card.rarity == CardRarity.RARE
        run_state.on_card_reward_taken(is_rare)

        return RewardResult(
            reward_type="card",
            item_id=card.id,
            success=True,
            message=f"Added {card.name} to deck"
        )

    @staticmethod
    def take_gold(run_state: RunState, gold_amount: int) -> RewardResult:
        """
        Take gold reward.

        Args:
            run_state: Run state to modify
            gold_amount: Amount of gold

        Returns:
            RewardResult with details
        """
        run_state.add_gold(gold_amount)

        return RewardResult(
            reward_type="gold",
            amount=gold_amount,
            success=True,
            message=f"Gained {gold_amount} gold"
        )

    @staticmethod
    def take_potion(run_state: RunState, potion: Potion) -> RewardResult:
        """
        Take potion reward.

        Args:
            run_state: Run state to modify
            potion: Potion to add

        Returns:
            RewardResult with details
        """
        if run_state.count_empty_potion_slots() == 0:
            return RewardResult(
                reward_type="potion",
                item_id=potion.id,
                success=False,
                message="No empty potion slots"
            )

        run_state.add_potion(potion.id)

        return RewardResult(
            reward_type="potion",
            item_id=potion.id,
            success=True,
            message=f"Obtained {potion.name}"
        )

    @staticmethod
    def take_relic(run_state: RunState, relic: Any) -> RewardResult:
        """
        Take relic reward.

        Args:
            run_state: Run state to modify
            relic: Relic to add

        Returns:
            RewardResult with details
        """
        run_state.add_relic(relic.id)

        return RewardResult(
            reward_type="relic",
            item_id=relic.id,
            success=True,
            message=f"Obtained {relic.name}"
        )

    @staticmethod
    def take_emerald_key(run_state: RunState) -> RewardResult:
        """
        Take the emerald key from a burning elite.

        Args:
            run_state: Run state to modify

        Returns:
            RewardResult with details
        """
        if run_state.has_emerald_key:
            return RewardResult(
                reward_type="key",
                item_id="emerald_key",
                success=False,
                message="Already have emerald key"
            )

        run_state.obtain_emerald_key()

        return RewardResult(
            reward_type="key",
            item_id="emerald_key",
            success=True,
            message="Obtained Emerald Key"
        )

    @staticmethod
    def skip_rewards(run_state: RunState) -> RewardResult:
        """
        Skip remaining rewards and continue.

        Args:
            run_state: Run state (unchanged)

        Returns:
            RewardResult indicating skip
        """
        return RewardResult(
            reward_type="skip",
            success=True,
            message="Skipped remaining rewards"
        )


# ============================================================================
# TESTING
# ============================================================================

if __name__ == "__main__":
    from ..state.run import create_watcher_run
    from ..state.rng import seed_to_long

    print("=== Room Handlers Test ===\n")

    # Create a test run
    seed = seed_to_long("TESTHANDLERS")
    run = create_watcher_run("TESTHANDLERS", ascension=20)

    print(f"Created run: {run}")
    print(f"Starting gold: {run.gold}")
    print(f"Starting HP: {run.current_hp}/{run.max_hp}\n")

    # Initialize RNG streams
    event_rng = Random(seed)
    merchant_rng = Random(seed)
    relic_rng = Random(seed)
    card_rng = Random(seed)
    treasure_rng = Random(seed)
    potion_rng = Random(seed)

    # Test Event Handler
    print("--- Event Handler ---")
    event = EventHandler.get_event(run, event_rng)
    if event:
        print(f"Selected event: {event.name}")
        choices = EventHandler.get_choices(event, run)
        print(f"Available choices: {len(choices)}")
        for choice in choices:
            print(f"  - {choice.description}")

    # Test Shop Handler
    print("\n--- Shop Handler ---")
    shop = ShopHandler.generate_shop(run, merchant_rng)
    print(f"Generated shop with {len(shop.colored_cards)} colored cards")
    print(f"Purge cost: {shop.purge_cost}")

    purchasable = ShopHandler.get_purchasable_items(shop, run)
    print(f"Can afford: {len(purchasable['colored_cards'])} cards, {len(purchasable['relics'])} relics")

    # Test Rest Handler
    print("\n--- Rest Handler ---")
    run.damage(20)  # Take some damage first
    print(f"HP before rest: {run.current_hp}/{run.max_hp}")
    options = RestHandler.get_options(run)
    print(f"Available options: {options}")

    result = RestHandler.rest(run)
    print(f"After rest: {run.current_hp}/{run.max_hp} (healed {result.hp_healed})")

    # Test Treasure Handler
    print("\n--- Treasure Handler ---")
    result = TreasureHandler.open_chest(run, relic_rng)
    print(f"Opened chest, got: {result.relic_name}")
    print(f"Now have relics: {[r.id for r in run.relics]}")

    # Test Reward Handler
    print("\n--- Reward Handler ---")
    rewards = RewardHandler.generate_combat_rewards(
        run, "normal", card_rng, treasure_rng, potion_rng, relic_rng
    )
    print(f"Gold reward: {rewards.gold}")
    print(f"Potion: {rewards.potion.name if rewards.potion else 'None'}")
    print(f"Card choices: {[c.name for c in rewards.card_choices]}")

    result = RewardHandler.take_gold(run, rewards.gold)
    print(f"Took gold: {result.message}")
    print(f"Final gold: {run.gold}")

    print("\n=== All tests passed ===")
