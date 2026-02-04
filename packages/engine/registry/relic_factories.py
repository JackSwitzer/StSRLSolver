"""Factory functions for common relic trigger patterns."""
from typing import Callable, Optional, Any
from functools import wraps

from . import relic_trigger, RelicContext


def counter_relic(
    hook: str,
    relic: str,
    threshold: int,
    action: Callable[[RelicContext], None],
    card_type_filter: Optional[str] = None,
    priority: int = 100,
):
    """
    Factory for counter-based relic triggers.

    Many relics follow: increment counter -> check threshold -> action -> reset
    Examples: Shuriken, Kunai, Ornamental Fan, Ink Bottle, Nunchaku

    Args:
        hook: The trigger hook (e.g., "onPlayCard")
        relic: Relic name
        threshold: Counter value that triggers action
        action: Function to call when threshold reached
        card_type_filter: Optional CardType name to filter ("ATTACK", "SKILL", etc.)
        priority: Trigger priority (default 100)
    """
    @relic_trigger(hook, relic=relic, priority=priority)
    def handler(ctx: RelicContext) -> None:
        # Check card type filter if specified
        if card_type_filter and ctx.card:
            from ..content.cards import CardType
            expected_type = getattr(CardType, card_type_filter, None)
            if ctx.card.card_type != expected_type:
                return

        counter = ctx.get_relic_counter(relic, 0) + 1
        if counter >= threshold:
            action(ctx)
            counter = 0
        ctx.set_relic_counter(relic, counter)

    return handler


def simple_power_relic(
    hook: str,
    relic: str,
    power_id: str,
    amount: int,
    to_player: bool = True,
    priority: int = 100,
):
    """
    Factory for relics that simply apply a power.

    Examples: Vajra (+1 Strength), Oddly Smooth Stone (+1 Dexterity)

    Args:
        hook: The trigger hook
        relic: Relic name
        power_id: Power to apply
        amount: Amount of power
        to_player: True for player, False for all enemies
        priority: Trigger priority
    """
    @relic_trigger(hook, relic=relic, priority=priority)
    def handler(ctx: RelicContext) -> None:
        if to_player:
            ctx.apply_power_to_player(power_id, amount)
        else:
            for enemy in ctx.living_enemies:
                ctx.apply_power_to_enemy(enemy, power_id, amount)

    return handler


def damage_all_enemies_relic(
    hook: str,
    relic: str,
    damage: int,
    priority: int = 100,
):
    """
    Factory for relics that deal damage to all enemies.

    Examples: Mercury Hourglass (3 damage at turn start)
    """
    @relic_trigger(hook, relic=relic, priority=priority)
    def handler(ctx: RelicContext) -> None:
        ctx.deal_damage_to_all_enemies(damage)

    return handler


def heal_on_event_relic(
    hook: str,
    relic: str,
    heal_amount: int,
    condition: Optional[Callable[[RelicContext], bool]] = None,
    priority: int = 100,
):
    """
    Factory for relics that heal on certain events.

    Examples: Bird-Faced Urn (heal 2 on power play), Toy Ornithopter (heal 5 on potion)
    """
    @relic_trigger(hook, relic=relic, priority=priority)
    def handler(ctx: RelicContext) -> None:
        if condition is None or condition(ctx):
            ctx.heal_player(heal_amount)

    return handler


def block_on_event_relic(
    hook: str,
    relic: str,
    block_amount: int,
    condition: Optional[Callable[[RelicContext], bool]] = None,
    priority: int = 100,
):
    """
    Factory for relics that grant block on certain events.

    Examples: Orichalcum (6 block if no block), Anchor (10 block at battle start)
    """
    @relic_trigger(hook, relic=relic, priority=priority)
    def handler(ctx: RelicContext) -> None:
        if condition is None or condition(ctx):
            ctx.gain_block(block_amount)

    return handler
