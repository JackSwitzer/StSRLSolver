"""
Unified Registry System for Slay the Spire RL Engine.

Provides decorator-based registration for all game effects:
- Card effects (existing, from effects/registry.py)
- Relic triggers
- Power triggers
- Potion effects

Usage:
    from packages.engine.registry import relic_trigger, power_trigger, potion_effect

    @relic_trigger("atBattleStart", relic="Vajra")
    def vajra_start(ctx: RelicContext) -> None:
        ctx.apply_power("Strength", 1)

    @power_trigger("atEndOfTurn", power="Metallicize")
    def metallicize_end(ctx: PowerContext) -> None:
        ctx.gain_block(ctx.amount)

    @potion_effect("Fire Potion")
    def fire_potion(ctx: PotionContext) -> None:
        ctx.deal_damage_to_target(ctx.potency)
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import (
    Callable, Dict, List, Optional, Any, Tuple, TYPE_CHECKING, Union, Set
)
from enum import Enum
import functools

if TYPE_CHECKING:
    from ..state.combat import CombatState, EnemyCombatState, EntityState


# =============================================================================
# Trigger Hooks - All possible trigger points
# =============================================================================

class TriggerHook(Enum):
    """All trigger hooks in the game, matching Java AbstractPower/AbstractRelic."""
    # Combat lifecycle
    AT_PRE_BATTLE = "atPreBattle"
    AT_BATTLE_START = "atBattleStart"
    AT_BATTLE_START_PRE_DRAW = "atBattleStartPreDraw"
    AT_TURN_START = "atTurnStart"
    AT_TURN_START_POST_DRAW = "atTurnStartPostDraw"
    ON_PLAYER_END_TURN = "onPlayerEndTurn"
    AT_END_OF_TURN = "atEndOfTurn"
    AT_END_OF_TURN_PRE_END_TURN_CARDS = "atEndOfTurnPreEndTurnCards"
    AT_END_OF_ROUND = "atEndOfRound"
    ON_VICTORY = "onVictory"

    # Card hooks
    ON_CARD_DRAW = "onCardDraw"
    ON_USE_CARD = "onUseCard"
    ON_AFTER_USE_CARD = "onAfterUseCard"
    ON_AFTER_CARD_PLAYED = "onAfterCardPlayed"
    ON_PLAY_CARD = "onPlayCard"
    ON_EXHAUST = "onExhaust"
    ON_MANUAL_DISCARD = "onManualDiscard"

    # Damage hooks
    AT_DAMAGE_GIVE = "atDamageGive"
    AT_DAMAGE_FINAL_GIVE = "atDamageFinalGive"
    AT_DAMAGE_RECEIVE = "atDamageReceive"
    AT_DAMAGE_FINAL_RECEIVE = "atDamageFinalReceive"
    ON_ATTACKED = "onAttacked"
    ON_ATTACKED_TO_CHANGE_DAMAGE = "onAttackedToChangeDamage"
    ON_ATTACK = "onAttack"
    WAS_HP_LOST = "wasHPLost"
    ON_LOSE_HP = "onLoseHp"
    ON_LOSE_HP_LAST = "onLoseHpLast"

    # Block hooks
    MODIFY_BLOCK = "modifyBlock"
    MODIFY_BLOCK_LAST = "modifyBlockLast"
    ON_PLAYER_GAIN_BLOCK = "onPlayerGainBlock"
    ON_GAIN_BLOCK = "onGainBlock"
    ON_BLOCK_BROKEN = "onBlockBroken"

    # Stance hooks (Watcher)
    ON_CHANGE_STANCE = "onChangeStance"

    # Orb hooks (Defect)
    ON_EVOKE_ORB = "onEvokeOrb"
    ON_CHANNEL_ORB = "onChannelOrb"

    # Other
    ON_SHUFFLE = "onShuffle"
    ON_SCRY = "onScry"
    ON_APPLY_POWER = "onApplyPower"
    ON_MONSTER_DEATH = "onMonsterDeath"
    ON_PLAYER_HEAL = "onPlayerHeal"
    ON_EQUIP = "onEquip"
    ON_OBTAIN_CARD = "onObtainCard"
    ON_ENERGY_RECHARGE = "onEnergyRecharge"
    ON_GAIN_GOLD = "onGainGold"
    ON_ENTER_ROOM = "onEnterRoom"

    # Rest site & map
    ON_REST_OPTION = "onRestOption"
    ON_FLY = "onFly"

    # Potion-specific
    ON_USE_POTION = "onUsePotion"

    # Hand state
    ON_EMPTY_HAND = "onEmptyHand"


# =============================================================================
# Context Classes - Passed to trigger handlers
# =============================================================================

@dataclass
class BaseContext:
    """Base context for all trigger handlers."""
    state: CombatState

    @property
    def player(self) -> EntityState:
        return self.state.player

    @property
    def enemies(self) -> List[EnemyCombatState]:
        return self.state.enemies

    @property
    def living_enemies(self) -> List[EnemyCombatState]:
        return [e for e in self.state.enemies if not e.is_dead]

    def has_relic(self, relic_id: str) -> bool:
        return self.state.has_relic(relic_id)

    def get_relic_counter(self, relic_id: str, default: int = 0) -> int:
        return self.state.get_relic_counter(relic_id, default)

    def set_relic_counter(self, relic_id: str, value: int) -> None:
        self.state.set_relic_counter(relic_id, value)

    def _card_random_rng(self):
        """RNG stream for random card/power/relic effects."""
        return (
            getattr(self.state, "card_random_rng", None)
            or getattr(self.state, "card_rng", None)
        )

    def _shuffle_rng(self):
        """RNG stream for shuffles."""
        return (
            getattr(self.state, "shuffle_rng", None)
            or self._card_random_rng()
        )

    def random_choice(self, values: List[Any]) -> Any:
        """Deterministic random choice with RNG-stream fallback."""
        if not values:
            raise ValueError("Cannot choose from empty list")
        rng = self._card_random_rng()
        if rng is None:
            return values[0]
        idx = rng.random(len(values) - 1)
        return values[idx]

    def shuffle_in_place(self, values: List[Any]) -> None:
        """Deterministic in-place shuffle."""
        rng = self._shuffle_rng()
        if rng is None or len(values) <= 1:
            return
        for i in range(len(values) - 1, 0, -1):
            j = rng.random(i)
            values[i], values[j] = values[j], values[i]

    def apply_power(self, target: Union[EntityState, EnemyCombatState, str],
                    power_id: str, amount: int) -> bool:
        """Apply a power to a target."""
        if isinstance(target, str):
            if target == "player":
                target_obj = self.player
            else:
                return False
        else:
            target_obj = target

        # Check artifact for debuffs
        # Standard debuffs that are always blocked by Artifact
        debuffs = {"Weak", "Weakened", "Vulnerable", "Frail", "Poison", "Constricted"}
        # Negative Strength/Dexterity applications are also debuffs (e.g., from Wraith Form)
        is_stat_debuff = power_id in {"Strength", "Dexterity"} and amount < 0

        if power_id in debuffs or is_stat_debuff:
            artifact = target_obj.statuses.get("Artifact", 0)
            if artifact > 0:
                target_obj.statuses["Artifact"] = artifact - 1
                if target_obj.statuses["Artifact"] <= 0:
                    del target_obj.statuses["Artifact"]
                return False

        current = target_obj.statuses.get(power_id, 0)
        target_obj.statuses[power_id] = current + amount
        # Clean up if stat reduced to 0
        if power_id in {"Strength", "Dexterity"} and target_obj.statuses[power_id] == 0:
            del target_obj.statuses[power_id]
        return True

    def apply_power_to_player(self, power_id: str, amount: int) -> bool:
        return self.apply_power(self.player, power_id, amount)

    def apply_power_to_all_enemies(self, power_id: str, amount: int) -> int:
        """Apply power to all enemies. Returns count of successful applications."""
        count = 0
        for enemy in self.living_enemies:
            if self.apply_power(enemy, power_id, amount):
                count += 1
        return count

    def gain_block(self, amount: int) -> None:
        """Gain block for the player."""
        if amount > 0:
            self.player.block += amount

    def gain_energy(self, amount: int) -> None:
        """Gain energy."""
        if amount > 0:
            self.state.energy += amount

    def draw_cards(self, count: int) -> List[str]:
        """Draw cards from draw pile."""
        drawn = []
        for _ in range(count):
            if not self.state.draw_pile:
                if not self.state.discard_pile:
                    break
                # Shuffle discard into draw
                self.state.draw_pile = self.state.discard_pile.copy()
                self.shuffle_in_place(self.state.draw_pile)
                self.state.discard_pile.clear()

            if self.state.draw_pile and len(self.state.hand) < 10:
                card = self.state.draw_pile.pop()
                self.state.hand.append(card)
                drawn.append(card)

        return drawn

    def heal_player(self, amount: int) -> int:
        """Heal the player. Returns actual amount healed.

        Applies onPlayerHeal trigger for relics like Magic Flower that modify healing.
        """
        # Apply onPlayerHeal triggers to modify heal amount
        if hasattr(self.state, 'relics') and 'Magic Flower' in self.state.relics:
            # Magic Flower: Healing is 50% more effective (only in combat)
            amount = round(amount * 1.5)

        max_heal = self.player.max_hp - self.player.hp
        actual = min(amount, max_heal)
        if actual > 0:
            self.player.hp += actual
        return actual

    def add_card_to_hand(self, card_id: str) -> bool:
        """Add a card to hand if space available."""
        if len(self.state.hand) < 10:
            self.state.hand.append(card_id)
            return True
        return False

    def deal_damage_to_enemy(self, enemy, amount: int) -> int:
        """Deal damage to single enemy, respecting block. Returns HP damage dealt."""
        blocked = min(enemy.block, amount)
        hp_damage = amount - blocked
        enemy.block -= blocked
        enemy.hp = max(0, enemy.hp - hp_damage)
        return hp_damage

    def deal_damage_to_all_enemies(self, amount: int) -> int:
        """Deal damage to all living enemies. Returns total HP damage dealt."""
        total = 0
        for enemy in self.living_enemies:
            total += self.deal_damage_to_enemy(enemy, amount)
        return total


@dataclass
class RelicContext(BaseContext):
    """Context for relic trigger handlers."""
    relic_id: str = ""
    trigger_data: Dict[str, Any] = field(default_factory=dict)

    # Specific data for different hooks
    card: Optional[Any] = None  # For onPlayCard, onExhaust
    damage: int = 0  # For damage hooks
    hp_lost: int = 0  # For wasHPLost
    target: Optional[EnemyCombatState] = None


@dataclass
class PowerContext(BaseContext):
    """Context for power trigger handlers."""
    power_id: str = ""
    amount: int = 0  # Current stack count
    owner: Union[EntityState, EnemyCombatState, None] = None
    trigger_data: Dict[str, Any] = field(default_factory=dict)

    # Specific data
    card: Optional[Any] = None
    damage: int = 0
    damage_type: str = "NORMAL"


@dataclass
class PotionContext(BaseContext):
    """Context for potion effect handlers."""
    potion_id: str = ""
    potency: int = 0  # Effective potency (with Sacred Bark if applicable)
    base_potency: int = 0  # Base potency before Sacred Bark
    target: Optional[EnemyCombatState] = None
    target_idx: int = -1
    has_sacred_bark: bool = False
    result_data: Dict[str, Any] = field(default_factory=dict)

    def deal_damage_to_target(self, amount: int) -> int:
        """Deal damage to the potion target."""
        if self.target and not self.target.is_dead:
            # Potion damage uses THORNS type (not affected by strength)
            blocked = min(self.target.block, amount)
            hp_damage = amount - blocked
            self.target.block -= blocked
            self.target.hp -= hp_damage
            if self.target.hp < 0:
                self.target.hp = 0
            return hp_damage
        return 0

    def deal_damage_to_all_enemies(self, amount: int) -> int:
        """Deal damage to all enemies."""
        total = 0
        for enemy in self.living_enemies:
            blocked = min(enemy.block, amount)
            hp_damage = amount - blocked
            enemy.block -= blocked
            enemy.hp -= hp_damage
            if enemy.hp < 0:
                enemy.hp = 0
            total += hp_damage
        return total


# =============================================================================
# Registry Classes
# =============================================================================

class TriggerRegistry:
    """Base registry for trigger handlers."""

    def __init__(self, name: str):
        self.name = name
        # handlers[hook][entity_id] = (handler_func, priority)
        self._handlers: Dict[str, Dict[str, Tuple[Callable, int]]] = {}

    def register(self, hook: str, entity_id: str, handler: Callable, priority: int = 100):
        """Register a handler for a hook."""
        if hook not in self._handlers:
            self._handlers[hook] = {}
        self._handlers[hook][entity_id] = (handler, priority)

    def get_handlers(self, hook: str, entity_ids: Optional[Set[str]] = None) -> List[Tuple[str, Callable]]:
        """Get all handlers for a hook, filtered by entity IDs, sorted by priority."""
        if hook not in self._handlers:
            return []

        handlers = []
        for entity_id, (handler, priority) in self._handlers[hook].items():
            if entity_ids is None or entity_id in entity_ids:
                handlers.append((entity_id, handler, priority))

        # Sort by priority (lower = earlier)
        handlers.sort(key=lambda x: x[2])
        return [(h[0], h[1]) for h in handlers]

    def get_handler(self, hook: str, entity_id: str) -> Optional[Callable]:
        """Get a specific handler."""
        if hook in self._handlers and entity_id in self._handlers[hook]:
            return self._handlers[hook][entity_id][0]
        return None

    def has_handler(self, hook: str, entity_id: str) -> bool:
        """Check if a handler exists."""
        return hook in self._handlers and entity_id in self._handlers[hook]

    def list_hooks(self) -> List[str]:
        """List all registered hooks."""
        return list(self._handlers.keys())

    def list_entities(self, hook: str) -> List[str]:
        """List all entities registered for a hook."""
        return list(self._handlers.get(hook, {}).keys())


# Global registries
RELIC_REGISTRY = TriggerRegistry("relics")
POWER_REGISTRY = TriggerRegistry("powers")
POTION_REGISTRY = TriggerRegistry("potions")


# =============================================================================
# Decorators
# =============================================================================

def relic_trigger(hook: str, relic: str, priority: int = 100):
    """
    Decorator to register a relic trigger handler.

    Args:
        hook: Trigger hook name (e.g., "atBattleStart", "wasHPLost")
        relic: Relic ID that this handler is for
        priority: Execution priority (lower = earlier)

    Usage:
        @relic_trigger("atBattleStart", relic="Vajra")
        def vajra_start(ctx: RelicContext) -> None:
            ctx.apply_power_to_player("Strength", 1)
    """
    def decorator(func: Callable[[RelicContext], Any]) -> Callable:
        RELIC_REGISTRY.register(hook, relic, func, priority)

        @functools.wraps(func)
        def wrapper(ctx: RelicContext) -> Any:
            return func(ctx)

        return wrapper
    return decorator


def power_trigger(hook: str, power: str, priority: int = 100):
    """
    Decorator to register a power trigger handler.

    Args:
        hook: Trigger hook name (e.g., "atEndOfTurn", "modifyBlock")
        power: Power ID that this handler is for
        priority: Execution priority

    Usage:
        @power_trigger("atEndOfTurnPreEndTurnCards", power="Metallicize")
        def metallicize_block(ctx: PowerContext) -> None:
            ctx.gain_block(ctx.amount)
    """
    def decorator(func: Callable[[PowerContext], Any]) -> Callable:
        POWER_REGISTRY.register(hook, power, func, priority)

        @functools.wraps(func)
        def wrapper(ctx: PowerContext) -> Any:
            return func(ctx)

        return wrapper
    return decorator


def potion_effect(potion: str, requires_target: bool = False,
                   target_type: str = "enemy"):
    """
    Decorator to register a potion effect handler.

    Args:
        potion: Potion ID
        requires_target: Whether potion requires a target
        target_type: Type of target ("enemy", "card_in_discard", etc.)

    Usage:
        @potion_effect("Fire Potion", requires_target=True)
        def fire_potion(ctx: PotionContext) -> None:
            ctx.deal_damage_to_target(ctx.potency)
    """
    def decorator(func: Callable[[PotionContext], Any]) -> Callable:
        # Store metadata on function
        func._potion_metadata = {
            "requires_target": requires_target,
            "target_type": target_type,
        }
        POTION_REGISTRY.register("onUsePotion", potion, func)

        @functools.wraps(func)
        def wrapper(ctx: PotionContext) -> Any:
            return func(ctx)

        return wrapper
    return decorator


# =============================================================================
# Execution Functions
# =============================================================================

def execute_relic_triggers(hook: str, state: CombatState,
                           trigger_data: Optional[Dict[str, Any]] = None) -> None:
    """
    Execute all relic triggers for a hook.

    Args:
        hook: Trigger hook name
        state: Current combat state
        trigger_data: Additional data for the trigger
    """
    if trigger_data is None:
        trigger_data = {}

    # Get player's relics
    player_relics = set(state.relics)

    # Get handlers for relics the player has
    handlers = RELIC_REGISTRY.get_handlers(hook, player_relics)

    for relic_id, handler in handlers:
        ctx = RelicContext(
            state=state,
            relic_id=relic_id,
            trigger_data=trigger_data,
            card=trigger_data.get("card"),
            damage=trigger_data.get("damage", 0),
            hp_lost=trigger_data.get("hp_lost", 0),
            target=trigger_data.get("target"),
        )
        handler(ctx)


def execute_power_triggers(hook: str, state: CombatState,
                           owner: Union[EntityState, EnemyCombatState],
                           trigger_data: Optional[Dict[str, Any]] = None) -> Any:
    """
    Execute power triggers for a specific owner.

    Args:
        hook: Trigger hook name
        state: Current combat state
        owner: Entity that owns the powers
        trigger_data: Additional data

    Returns:
        Result from handlers (for modifier hooks like modifyBlock)
    """
    if trigger_data is None:
        trigger_data = {}

    result = trigger_data.get("value")

    # Get powers the owner has
    owner_powers = set(owner.statuses.keys())

    handlers = POWER_REGISTRY.get_handlers(hook, owner_powers)

    for power_id, handler in handlers:
        ctx = PowerContext(
            state=state,
            power_id=power_id,
            amount=owner.statuses.get(power_id, 0),
            owner=owner,
            trigger_data=trigger_data,
            card=trigger_data.get("card"),
            damage=trigger_data.get("damage", 0),
            damage_type=trigger_data.get("damage_type", "NORMAL"),
        )
        handler_result = handler(ctx)
        if handler_result is not None:
            result = handler_result

    return result


def execute_potion_effect(potion_id: str, state: CombatState,
                          target_idx: int = -1) -> Dict[str, Any]:
    """
    Execute a potion's effect.

    Args:
        potion_id: Potion ID to use
        state: Combat state
        target_idx: Target enemy index (-1 for no target)

    Returns:
        Dict with effect results
    """
    from ..content.potions import get_potion_by_id

    potion = get_potion_by_id(potion_id)
    if not potion:
        return {"success": False, "error": f"Unknown potion: {potion_id}"}

    has_sacred_bark = state.has_relic("SacredBark")
    potency = potion.get_effective_potency(has_sacred_bark)

    target = None
    if 0 <= target_idx < len(state.enemies):
        target = state.enemies[target_idx]

    ctx = PotionContext(
        state=state,
        potion_id=potion_id,
        potency=potency,
        base_potency=potion.potency,
        target=target,
        target_idx=target_idx,
        has_sacred_bark=has_sacred_bark,
    )

    handler = POTION_REGISTRY.get_handler("onUsePotion", potion_id)
    if handler:
        handler(ctx)
        if ctx.result_data.get("success") is False:
            return {
                "success": False,
                "error": ctx.result_data.get("error", f"Potion cannot be used: {potion_id}"),
                "potion": potion_id,
            }

        execute_relic_triggers("onUsePotion", state, {"potion": potion_id})
        result = {"success": True, "potion": potion_id, "potency": potency}
        for key, value in ctx.result_data.items():
            if key in {"success", "error"}:
                continue
            result[key] = value
        return result

    return {"success": False, "error": f"No effect handler for: {potion_id}"}


# =============================================================================
# Exports
# =============================================================================

__all__ = [
    # Enums
    "TriggerHook",

    # Context classes
    "BaseContext",
    "RelicContext",
    "PowerContext",
    "PotionContext",

    # Registry
    "TriggerRegistry",
    "RELIC_REGISTRY",
    "POWER_REGISTRY",
    "POTION_REGISTRY",

    # Decorators
    "relic_trigger",
    "power_trigger",
    "potion_effect",

    # Execution
    "execute_relic_triggers",
    "execute_power_triggers",
    "execute_potion_effect",
]

# Import handlers to register them (decorators populate the registries)
from . import potions as _potions  # noqa: F401, E402
from . import relics as _relics  # noqa: F401, E402
from . import powers as _powers  # noqa: F401, E402
from . import relics_passive as _relics_passive  # noqa: F401, E402
