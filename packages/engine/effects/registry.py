"""
Effect Registry System for Slay the Spire RL.

Provides a decorator-based registration system for card effects.
Effects are pure functions that take combat state and parameters,
returning the modified state.

Usage:
    @effect("draw")
    def draw_cards(ctx: EffectContext, amount: int) -> None:
        ctx.draw_cards(amount)

    # Later, in effect execution:
    execute_effect("draw_2", ctx)  # Parses "draw_2" -> draw(2)
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import (
    Callable, Dict, List, Optional, Any, Tuple, TYPE_CHECKING, Union
)
from enum import Enum
import re

if TYPE_CHECKING:
    from ..state.combat import CombatState, EnemyCombatState, EntityState
    from ..content.cards import Card


class EffectTiming(Enum):
    """When an effect triggers."""
    ON_PLAY = "on_play"  # When card is played
    ON_DRAW = "on_draw"  # When card is drawn
    ON_DISCARD = "on_discard"  # When card is discarded
    ON_EXHAUST = "on_exhaust"  # When card is exhausted
    ON_RETAIN = "on_retain"  # When card is retained at end of turn
    START_OF_TURN = "start_of_turn"  # At start of player turn
    END_OF_TURN = "end_of_turn"  # At end of player turn
    ON_STANCE_CHANGE = "on_stance_change"  # When stance changes
    ON_SCRY = "on_scry"  # When scry is performed
    ON_DAMAGE_DEALT = "on_damage_dealt"  # When player deals damage
    ON_DAMAGE_TAKEN = "on_damage_taken"  # When player takes damage
    ON_BLOCK_GAINED = "on_block_gained"  # When player gains block
    ON_ENEMY_DEATH = "on_enemy_death"  # When an enemy dies
    PASSIVE = "passive"  # Always active (for powers)


@dataclass
class EffectContext:
    """
    Context passed to effect handlers.

    Provides convenient methods for modifying combat state
    while tracking all changes for logging/debugging.
    """
    state: CombatState
    card: Optional[Card] = None
    target: Optional[EnemyCombatState] = None
    target_idx: int = -1

    # Tracking for effect execution
    damage_dealt: int = 0
    block_gained: int = 0
    cards_drawn: List[str] = field(default_factory=list)
    cards_discarded: List[str] = field(default_factory=list)
    cards_exhausted: List[str] = field(default_factory=list)
    statuses_applied: List[Tuple[str, str, int]] = field(default_factory=list)
    energy_gained: int = 0
    energy_spent: int = 0
    stance_changed_to: Optional[str] = None
    mantra_gained: int = 0

    # For scry
    scried_cards: List[str] = field(default_factory=list)

    # Additional context
    is_upgraded: bool = False
    magic_number: int = 0
    extra_data: Dict[str, Any] = field(default_factory=dict)

    # ------------------------------------------------------------------
    # Player State Access
    # ------------------------------------------------------------------

    @property
    def player(self) -> EntityState:
        return self.state.player

    @property
    def enemies(self) -> List[EnemyCombatState]:
        return self.state.enemies

    @property
    def living_enemies(self) -> List[EnemyCombatState]:
        return [e for e in self.state.enemies if not e.is_dead]

    @property
    def hand(self) -> List[str]:
        return self.state.hand

    @property
    def draw_pile(self) -> List[str]:
        return self.state.draw_pile

    @property
    def discard_pile(self) -> List[str]:
        return self.state.discard_pile

    @property
    def exhaust_pile(self) -> List[str]:
        return self.state.exhaust_pile

    @property
    def energy(self) -> int:
        return self.state.energy

    @property
    def stance(self) -> str:
        return self.state.stance

    @property
    def turn(self) -> int:
        return self.state.turn

    # ------------------------------------------------------------------
    # Card Manipulation
    # ------------------------------------------------------------------

    def draw_cards(self, count: int) -> List[str]:
        """Draw cards from draw pile to hand."""
        drawn = []
        for _ in range(count):
            if not self.state.draw_pile:
                # Shuffle discard into draw
                if not self.state.discard_pile:
                    break
                self._shuffle_discard_into_draw()

            if self.state.draw_pile:
                card = self.state.draw_pile.pop()
                self.state.hand.append(card)
                drawn.append(card)
                self.cards_drawn.append(card)

        return drawn

    def _shuffle_discard_into_draw(self) -> None:
        """Shuffle discard pile into draw pile."""
        import random
        self.state.draw_pile = self.state.discard_pile.copy()
        random.shuffle(self.state.draw_pile)
        self.state.discard_pile.clear()

    def discard_card(self, card_id: str, from_hand: bool = True) -> bool:
        """Discard a card."""
        if from_hand and card_id in self.state.hand:
            self.state.hand.remove(card_id)
            self.state.discard_pile.append(card_id)
            self.cards_discarded.append(card_id)
            return True
        return False

    def discard_hand_idx(self, idx: int) -> Optional[str]:
        """Discard card at index in hand."""
        if 0 <= idx < len(self.state.hand):
            card = self.state.hand.pop(idx)
            self.state.discard_pile.append(card)
            self.cards_discarded.append(card)
            return card
        return None

    def exhaust_card(self, card_id: str, from_hand: bool = True) -> bool:
        """Exhaust a card."""
        if from_hand and card_id in self.state.hand:
            self.state.hand.remove(card_id)
            self.state.exhaust_pile.append(card_id)
            self.cards_exhausted.append(card_id)
            return True
        return False

    def exhaust_hand_idx(self, idx: int) -> Optional[str]:
        """Exhaust card at index in hand."""
        if 0 <= idx < len(self.state.hand):
            card = self.state.hand.pop(idx)
            self.state.exhaust_pile.append(card)
            self.cards_exhausted.append(card)
            return card
        return None

    def add_card_to_hand(self, card_id: str) -> bool:
        """Add a card to hand (up to hand limit of 10)."""
        if len(self.state.hand) < 10:
            self.state.hand.append(card_id)
            return True
        return False

    def add_card_to_draw_pile(self, card_id: str, position: str = "random") -> None:
        """
        Add a card to draw pile.

        Args:
            card_id: Card to add
            position: "top", "bottom", or "random"
        """
        import random
        if position == "top":
            self.state.draw_pile.append(card_id)
        elif position == "bottom":
            self.state.draw_pile.insert(0, card_id)
        else:  # random
            if self.state.draw_pile:
                idx = random.randint(0, len(self.state.draw_pile))
                self.state.draw_pile.insert(idx, card_id)
            else:
                self.state.draw_pile.append(card_id)

    def add_card_to_discard(self, card_id: str) -> None:
        """Add a card to discard pile."""
        self.state.discard_pile.append(card_id)

    def move_card_from_discard_to_hand(self, card_id: str) -> bool:
        """Move a specific card from discard to hand."""
        if card_id in self.state.discard_pile and len(self.state.hand) < 10:
            self.state.discard_pile.remove(card_id)
            self.state.hand.append(card_id)
            return True
        return False

    # ------------------------------------------------------------------
    # Damage and Block
    # ------------------------------------------------------------------

    def deal_damage_to_target(self, amount: int) -> int:
        """Deal damage to the current target."""
        if self.target and not self.target.is_dead:
            actual = self._apply_damage_to_enemy(self.target, amount)
            self.damage_dealt += actual
            return actual
        return 0

    def deal_damage_to_enemy(self, enemy: EnemyCombatState, amount: int) -> int:
        """Deal damage to a specific enemy."""
        if not enemy.is_dead:
            actual = self._apply_damage_to_enemy(enemy, amount)
            self.damage_dealt += actual
            return actual
        return 0

    def deal_damage_to_all_enemies(self, amount: int) -> int:
        """Deal damage to all living enemies."""
        total = 0
        for enemy in self.living_enemies:
            total += self.deal_damage_to_enemy(enemy, amount)
        return total

    def deal_damage_to_random_enemy(self, amount: int) -> int:
        """Deal damage to a random living enemy."""
        import random
        living = self.living_enemies
        if living:
            target = random.choice(living)
            return self.deal_damage_to_enemy(target, amount)
        return 0

    def _apply_damage_to_enemy(self, enemy: EnemyCombatState, amount: int) -> int:
        """Apply damage accounting for block."""
        if amount <= 0:
            return 0

        # Block absorbs damage first
        blocked = min(enemy.block, amount)
        enemy.block -= blocked
        hp_damage = amount - blocked

        # Apply HP damage
        enemy.hp -= hp_damage
        if enemy.hp < 0:
            enemy.hp = 0

        return hp_damage

    def gain_block(self, amount: int) -> int:
        """Gain block for the player."""
        if amount > 0:
            self.state.player.block += amount
            self.block_gained += amount
        return amount

    def deal_damage_to_player(self, amount: int) -> int:
        """Deal damage to the player."""
        if amount <= 0:
            return 0

        # Block absorbs damage first
        blocked = min(self.state.player.block, amount)
        self.state.player.block -= blocked
        hp_damage = amount - blocked

        # Apply HP damage
        self.state.player.hp -= hp_damage
        if self.state.player.hp < 0:
            self.state.player.hp = 0

        return hp_damage

    def heal_player(self, amount: int) -> int:
        """Heal the player."""
        max_heal = self.state.player.max_hp - self.state.player.hp
        actual_heal = min(amount, max_heal)
        if actual_heal > 0:
            self.state.player.hp += actual_heal
        return actual_heal

    # ------------------------------------------------------------------
    # Status Effects
    # ------------------------------------------------------------------

    def apply_status_to_target(self, status: str, amount: int) -> bool:
        """Apply a status effect to the current target."""
        if self.target:
            return self.apply_status_to_enemy(self.target, status, amount)
        return False

    def apply_status_to_enemy(self, enemy: EnemyCombatState, status: str, amount: int) -> bool:
        """Apply a status effect to an enemy."""
        if enemy.is_dead:
            return False

        # Check Artifact for debuffs
        debuffs = {"Weak", "Vulnerable", "Frail", "Poison", "Mark", "Constricted"}
        if status in debuffs:
            artifact = enemy.statuses.get("Artifact", 0)
            if artifact > 0:
                enemy.statuses["Artifact"] = artifact - 1
                if enemy.statuses["Artifact"] <= 0:
                    del enemy.statuses["Artifact"]
                return False  # Blocked by artifact

        current = enemy.statuses.get(status, 0)
        enemy.statuses[status] = current + amount
        self.statuses_applied.append((enemy.id, status, amount))
        return True

    def apply_status_to_all_enemies(self, status: str, amount: int) -> int:
        """Apply a status effect to all living enemies."""
        count = 0
        for enemy in self.living_enemies:
            if self.apply_status_to_enemy(enemy, status, amount):
                count += 1
        return count

    def apply_status_to_player(self, status: str, amount: int) -> bool:
        """Apply a status effect to the player."""
        # Check Artifact for debuffs
        debuffs = {"Weak", "Vulnerable", "Frail", "Poison", "Constricted"}
        if status in debuffs:
            artifact = self.state.player.statuses.get("Artifact", 0)
            if artifact > 0:
                self.state.player.statuses["Artifact"] = artifact - 1
                if self.state.player.statuses["Artifact"] <= 0:
                    del self.state.player.statuses["Artifact"]
                return False  # Blocked by artifact

        current = self.state.player.statuses.get(status, 0)
        self.state.player.statuses[status] = current + amount
        self.statuses_applied.append(("player", status, amount))
        return True

    def remove_status_from_player(self, status: str) -> int:
        """Remove a status from the player, returning the amount removed."""
        amount = self.state.player.statuses.pop(status, 0)
        return amount

    def get_player_status(self, status: str) -> int:
        """Get the amount of a player status."""
        return self.state.player.statuses.get(status, 0)

    def get_enemy_status(self, enemy: EnemyCombatState, status: str) -> int:
        """Get the amount of an enemy status."""
        return enemy.statuses.get(status, 0)

    # ------------------------------------------------------------------
    # Energy
    # ------------------------------------------------------------------

    def gain_energy(self, amount: int) -> None:
        """Gain energy."""
        if amount > 0:
            self.state.energy += amount
            self.energy_gained += amount

    def spend_energy(self, amount: int) -> bool:
        """Spend energy if available."""
        if self.state.energy >= amount:
            self.state.energy -= amount
            self.energy_spent += amount
            return True
        return False

    # ------------------------------------------------------------------
    # Stance
    # ------------------------------------------------------------------

    def change_stance(self, new_stance: str) -> Dict[str, Any]:
        """
        Change to a new stance.

        Returns dict with effects triggered (energy gained, etc.)
        """
        old_stance = self.state.stance
        result = {"old_stance": old_stance, "new_stance": new_stance, "energy_gained": 0}

        if old_stance == new_stance:
            return result

        # Track state energy before relic triggers
        energy_before_relics = self.state.energy

        # Exit current stance
        if old_stance == "Calm":
            # Gain 2 energy base (Violet Lotus adds +1 via relic trigger)
            self.gain_energy(2)
            result["energy_gained"] += 2

        # Enter new stance
        self.state.stance = new_stance
        self.stance_changed_to = new_stance

        if new_stance == "Divinity":
            # Gain 3 energy on entering Divinity
            self.gain_energy(3)
            result["energy_gained"] += 3

        # Execute relic triggers for stance change (Violet Lotus)
        from ..registry import execute_relic_triggers
        execute_relic_triggers("onChangeStance", self.state, {"new_stance": new_stance, "old_stance": old_stance})

        # Update result with any additional energy from relic triggers
        # (RelicContext.gain_energy modifies state.energy but doesn't track it,
        #  so we check the state energy difference)
        energy_after_relics = self.state.energy
        energy_from_relics = energy_after_relics - energy_before_relics - result["energy_gained"]
        if energy_from_relics > 0:
            result["energy_gained"] += energy_from_relics
            # Also update our tracking to stay consistent
            self.energy_gained += energy_from_relics

        # Trigger Mental Fortress (gain block on stance change)
        mental_fortress = self.get_player_status("MentalFortress")
        if mental_fortress > 0:
            self.gain_block(mental_fortress)
            result["block_gained"] = mental_fortress

        # Trigger Flurry of Blows
        self._trigger_flurry_of_blows()

        # Trigger Rushdown (draw on entering Wrath)
        if new_stance == "Wrath":
            rushdown = self.get_player_status("Rushdown")
            if rushdown > 0:
                self.draw_cards(rushdown)
                result["cards_drawn"] = rushdown

        return result

    def exit_stance(self) -> Dict[str, Any]:
        """Exit to Neutral stance."""
        return self.change_stance("Neutral")

    def _trigger_flurry_of_blows(self) -> None:
        """Move Flurry of Blows from discard to hand on stance change."""
        flurries = [c for c in self.state.discard_pile if c.startswith("FlurryOfBlows")]
        for f in flurries:
            if len(self.state.hand) < 10:
                self.state.discard_pile.remove(f)
                self.state.hand.append(f)

    # ------------------------------------------------------------------
    # Mantra
    # ------------------------------------------------------------------

    def gain_mantra(self, amount: int) -> Dict[str, Any]:
        """
        Gain mantra and potentially enter Divinity.

        Returns dict with results including whether Divinity was triggered.
        """
        current = self.get_player_status("Mantra")
        new_total = current + amount
        self.mantra_gained += amount

        result = {"mantra_gained": amount, "divinity_triggered": False}

        if new_total >= 10:
            # Enter Divinity
            remainder = new_total - 10
            self.apply_status_to_player("Mantra", -current + remainder)  # Reset to remainder
            stance_result = self.change_stance("Divinity")
            result["divinity_triggered"] = True
            result.update(stance_result)
        else:
            self.apply_status_to_player("Mantra", amount)

        return result

    # ------------------------------------------------------------------
    # Scry
    # ------------------------------------------------------------------

    def scry(self, amount: int) -> List[str]:
        """
        Scry X cards.

        In a real implementation, this would let the player choose which
        cards to discard. For simulation, we'll just reveal the cards.

        Returns list of cards that were scryed.
        """
        cards_to_scry = []

        for _ in range(amount):
            if not self.state.draw_pile:
                break
            card = self.state.draw_pile.pop()
            cards_to_scry.append(card)

        self.scried_cards = cards_to_scry

        # Trigger Nirvana (gain block on scry)
        nirvana = self.get_player_status("Nirvana")
        if nirvana > 0:
            self.gain_block(nirvana * len(cards_to_scry))

        # Trigger Weave (play from discard on scry)
        self._trigger_weave()

        # Put cards back on top of draw pile (in reverse order so first is on top)
        # In actual game, player chooses which go to discard
        for card in reversed(cards_to_scry):
            self.state.draw_pile.append(card)

        return cards_to_scry

    def _trigger_weave(self) -> None:
        """Move Weave from discard to hand on scry."""
        weaves = [c for c in self.state.discard_pile if c.startswith("Weave")]
        for w in weaves:
            if len(self.state.hand) < 10:
                self.state.discard_pile.remove(w)
                self.state.hand.append(w)

    # ------------------------------------------------------------------
    # Utility
    # ------------------------------------------------------------------

    def has_relic(self, relic_id: str) -> bool:
        """Check if player has a relic."""
        return self.state.has_relic(relic_id)

    def is_enemy_attacking(self, enemy: Optional[EnemyCombatState] = None) -> bool:
        """Check if an enemy (or the target) is attacking."""
        target = enemy or self.target
        if target:
            return target.is_attacking
        return False

    def any_enemy_attacking(self) -> bool:
        """Check if any enemy is attacking."""
        return any(e.is_attacking for e in self.living_enemies)

    def count_cards_in_hand_of_type(self, card_type: str) -> int:
        """Count cards of a specific type in hand."""
        # This would need card registry lookup
        # For now, use naming convention
        count = 0
        for card_id in self.state.hand:
            if card_type == "ATTACK":
                # Attacks typically have damage
                if card_id in ["Strike_P", "Eruption", "BowlingBash", "CutThroughFate",
                              "EmptyFist", "FlurryOfBlows", "FlyingSleeves", "Tantrum",
                              "Ragnarok", "Brilliance", "LessonLearned"]:  # etc.
                    count += 1
        return count

    def get_last_card_type(self) -> Optional[str]:
        """Get the type of the last card played this turn."""
        return self.extra_data.get("last_card_type")

    def set_last_card_type(self, card_type: str) -> None:
        """Set the type of the last card played."""
        self.extra_data["last_card_type"] = card_type

    def end_turn(self) -> None:
        """Mark that the turn should end (for cards like Conclude)."""
        self.extra_data["end_turn"] = True

    def should_end_turn(self) -> bool:
        """Check if turn should end early."""
        return self.extra_data.get("end_turn", False)


# =============================================================================
# Effect Registry
# =============================================================================

# Global registry of effect handlers
_EFFECT_REGISTRY: Dict[str, Callable[[EffectContext, Any], None]] = {}

# Mapping of effect patterns to handlers and param extractors
_EFFECT_PATTERNS: List[Tuple[re.Pattern, str, Callable[[re.Match], Tuple[Any, ...]]]] = []


def effect(name: str, pattern: Optional[str] = None):
    """
    Decorator to register an effect handler.

    Args:
        name: Base name of the effect (e.g., "draw", "scry")
        pattern: Optional regex pattern for parsing effect strings.
                 If None, uses "{name}_{number}" pattern.

    Usage:
        @effect("draw")
        def draw_cards(ctx: EffectContext, amount: int) -> None:
            ctx.draw_cards(amount)

        # This registers handlers for "draw", "draw_1", "draw_2", etc.
    """
    def decorator(func: Callable[[EffectContext, Any], None]) -> Callable:
        # Register the base name
        _EFFECT_REGISTRY[name] = func

        # Create pattern for "{name}_{number}" format
        if pattern:
            regex = re.compile(pattern)
        else:
            regex = re.compile(rf"^{re.escape(name)}_(\d+)$")

        def extractor(match: re.Match) -> Tuple[Any, ...]:
            groups = match.groups()
            if groups:
                return (int(groups[0]),)
            return ()

        _EFFECT_PATTERNS.append((regex, name, extractor))

        return func

    return decorator


def effect_simple(name: str):
    """
    Decorator for simple effects with no parameters.

    Usage:
        @effect_simple("end_turn")
        def end_turn_effect(ctx: EffectContext) -> None:
            ctx.end_turn()
    """
    def decorator(func: Callable[[EffectContext], None]) -> Callable:
        _EFFECT_REGISTRY[name] = func
        return func

    return decorator


def effect_custom(name: str, pattern: str, param_types: List[type] = None):
    """
    Decorator for effects with custom patterns.

    Args:
        name: Effect name
        pattern: Regex pattern with capture groups
        param_types: Types for each capture group (default: all int)

    Usage:
        @effect_custom("damage_x_times", r"damage_(\d+)_times_(\d+)")
        def damage_x_times(ctx: EffectContext, damage: int, times: int):
            for _ in range(times):
                ctx.deal_damage_to_target(damage)
    """
    def decorator(func: Callable) -> Callable:
        _EFFECT_REGISTRY[name] = func

        regex = re.compile(pattern)
        types = param_types or []

        def extractor(match: re.Match) -> Tuple[Any, ...]:
            groups = match.groups()
            result = []
            for i, g in enumerate(groups):
                if i < len(types):
                    result.append(types[i](g))
                else:
                    # Default to int
                    try:
                        result.append(int(g))
                    except ValueError:
                        result.append(g)
            return tuple(result)

        _EFFECT_PATTERNS.append((regex, name, extractor))

        return func

    return decorator


def get_effect_handler(effect_name: str) -> Optional[Tuple[Callable, Tuple[Any, ...]]]:
    """
    Get the handler and parameters for an effect string.

    Args:
        effect_name: Effect string like "draw_2" or "gain_block_5"

    Returns:
        Tuple of (handler_function, params) or None if not found
    """
    # Check direct match first
    if effect_name in _EFFECT_REGISTRY:
        return (_EFFECT_REGISTRY[effect_name], ())

    # Try pattern matching
    for pattern, base_name, extractor in _EFFECT_PATTERNS:
        match = pattern.match(effect_name)
        if match:
            handler = _EFFECT_REGISTRY.get(base_name)
            if handler:
                params = extractor(match)
                return (handler, params)

    return None


def execute_effect(effect_name: str, ctx: EffectContext) -> bool:
    """
    Execute a named effect.

    Args:
        effect_name: Effect string like "draw_2"
        ctx: Effect context

    Returns:
        True if effect was executed, False if not found
    """
    result = get_effect_handler(effect_name)
    if result:
        handler, params = result
        if params:
            handler(ctx, *params)
        else:
            handler(ctx)
        return True
    return False


def list_registered_effects() -> List[str]:
    """List all registered effect names."""
    return list(_EFFECT_REGISTRY.keys())
