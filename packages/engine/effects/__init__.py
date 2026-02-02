"""
Card Effects System for Slay the Spire RL.

This module provides a comprehensive system for defining and executing
card effects. It uses a registry pattern with decorators for clean,
maintainable effect definitions.

Main Components:
- EffectRegistry: Decorator-based effect registration
- EffectExecutor: Central class for executing card effects
- EffectContext: Context object passed to effect handlers
- Card effects: All Watcher card effect implementations

Usage:
    from core.effects import EffectExecutor, EffectContext
    from core.state.combat import CombatState

    # Create executor from combat state
    executor = EffectExecutor(state)

    # Play a card
    result = executor.play_card(card, target_idx=0)

    # Check results
    if result.success:
        print(f"Dealt {result.damage_dealt} damage")
        print(f"Gained {result.block_gained} block")

Effect Registration:
    from core.effects import effect, effect_simple

    @effect("draw")
    def draw_cards(ctx: EffectContext, amount: int) -> None:
        ctx.draw_cards(amount)

    @effect_simple("enter_wrath")
    def enter_wrath(ctx: EffectContext) -> None:
        ctx.change_stance("Wrath")
"""

from .registry import (
    # Core types
    EffectContext,
    EffectTiming,

    # Decorators
    effect,
    effect_simple,
    effect_custom,

    # Registry functions
    get_effect_handler,
    execute_effect,
    list_registered_effects,
)

from .executor import (
    EffectExecutor,
    EffectResult,
    create_executor,
)

# Import cards module to register all effects
from . import cards as _cards  # noqa: F401

__all__ = [
    # Core types
    "EffectContext",
    "EffectTiming",
    "EffectResult",

    # Main class
    "EffectExecutor",

    # Factory
    "create_executor",

    # Decorators
    "effect",
    "effect_simple",
    "effect_custom",

    # Registry functions
    "get_effect_handler",
    "execute_effect",
    "list_registered_effects",
]
