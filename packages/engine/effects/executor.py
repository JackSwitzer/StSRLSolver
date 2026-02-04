"""
Effect Executor for Slay the Spire RL.

Central class that executes card effects by integrating with
the effect registry and managing combat state modifications.

Usage:
    from core.effects import EffectExecutor
    from core.state.combat import CombatState

    executor = EffectExecutor(combat_state)
    result = executor.play_card(card, target_idx=0)
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import (
    Dict, List, Optional, Any, Tuple, TYPE_CHECKING, Callable
)

from .registry import (
    EffectContext, execute_effect, get_effect_handler,
    EffectTiming, list_registered_effects
)

if TYPE_CHECKING:
    from ..state.combat import CombatState, EnemyCombatState
    from ..content.cards import Card, CardType


@dataclass
class EffectResult:
    """Result of executing an effect or playing a card."""
    success: bool
    effects_executed: List[str] = field(default_factory=list)
    damage_dealt: int = 0
    block_gained: int = 0
    energy_spent: int = 0
    energy_gained: int = 0
    cards_drawn: List[str] = field(default_factory=list)
    cards_discarded: List[str] = field(default_factory=list)
    cards_exhausted: List[str] = field(default_factory=list)
    statuses_applied: List[Tuple[str, str, int]] = field(default_factory=list)
    stance_changed_to: Optional[str] = None
    mantra_gained: int = 0
    scried_cards: List[str] = field(default_factory=list)
    errors: List[str] = field(default_factory=list)
    extra: Dict[str, Any] = field(default_factory=dict)

    @property
    def should_end_turn(self) -> bool:
        """Check if the card effect should end the turn."""
        return self.extra.get("end_turn", False)


class EffectExecutor:
    """
    Executes card effects and manages combat state.

    This class serves as the central integration point between
    cards, effects, and combat state. It handles:

    1. Playing cards with all their effects
    2. Applying damage with stance multipliers
    3. Applying block with dexterity/frail
    4. Managing stance changes
    5. Triggering on-play hooks
    """

    # Stance damage multipliers
    STANCE_DAMAGE_MULT = {
        "Neutral": 1.0,
        "Calm": 1.0,
        "Wrath": 2.0,
        "Divinity": 3.0,
    }

    def __init__(self, state: CombatState):
        """
        Initialize the executor with combat state.

        Args:
            state: The CombatState to modify
        """
        self.state = state
        self._on_card_played_hooks: List[Callable[[EffectContext, Card], None]] = []
        self._on_damage_dealt_hooks: List[Callable[[EffectContext, int, EnemyCombatState], None]] = []
        self._on_stance_change_hooks: List[Callable[[EffectContext, str, str], None]] = []

    # =========================================================================
    # Main Card Execution
    # =========================================================================

    def play_card(
        self,
        card: Card,
        target_idx: int = -1,
        free: bool = False,
    ) -> EffectResult:
        """
        Play a card and execute all its effects.

        Args:
            card: The Card object to play
            target_idx: Index of target enemy (-1 for self/no target)
            free: If True, don't spend energy

        Returns:
            EffectResult with all effects that occurred
        """
        result = EffectResult(success=True)

        # Get target
        target = None
        if 0 <= target_idx < len(self.state.enemies):
            target = self.state.enemies[target_idx]
            if target.is_dead:
                target = None

        # Create effect context
        ctx = EffectContext(
            state=self.state,
            card=card,
            target=target,
            target_idx=target_idx,
            is_upgraded=card.upgraded,
            magic_number=card.magic_number if card.magic_number > 0 else 0,
        )

        # Spend energy
        if not free:
            cost = card.current_cost
            if cost > 0:
                if self.state.energy >= cost:
                    self.state.energy -= cost
                    ctx.energy_spent = cost
                    result.energy_spent = cost
                else:
                    result.success = False
                    result.errors.append(f"Not enough energy: need {cost}, have {self.state.energy}")
                    return result

        # Execute base damage
        if card.base_damage > 0:
            self._execute_damage(ctx, card, target, result)

        # Execute base block
        if card.base_block > 0:
            self._execute_block(ctx, card, result)

        # Execute stance change
        if card.enter_stance:
            stance_result = ctx.change_stance(card.enter_stance)
            result.stance_changed_to = card.enter_stance
            result.energy_gained += stance_result.get("energy_gained", 0)
            result.effects_executed.append(f"enter_stance_{card.enter_stance}")

        if card.exit_stance:
            stance_result = ctx.exit_stance()
            result.stance_changed_to = "Neutral"
            result.energy_gained += stance_result.get("energy_gained", 0)
            result.effects_executed.append("exit_stance")

        # Execute card-specific effects from the effects list
        for effect_name in card.effects:
            if execute_effect(effect_name, ctx):
                result.effects_executed.append(effect_name)
            else:
                # Try to handle unknown effect with fallback
                handled = self._handle_special_effect(effect_name, ctx, card, result)
                if handled:
                    result.effects_executed.append(effect_name)
                else:
                    result.errors.append(f"Unknown effect: {effect_name}")

        # Collect results from context
        result.damage_dealt += ctx.damage_dealt
        result.block_gained += ctx.block_gained
        result.energy_gained += ctx.energy_gained
        result.cards_drawn.extend(ctx.cards_drawn)
        result.cards_discarded.extend(ctx.cards_discarded)
        result.cards_exhausted.extend(ctx.cards_exhausted)
        result.statuses_applied.extend(ctx.statuses_applied)
        result.mantra_gained = ctx.mantra_gained
        result.scried_cards = ctx.scried_cards

        if ctx.should_end_turn():
            result.extra["end_turn"] = True

        # Trigger on-play hooks
        for hook in self._on_card_played_hooks:
            hook(ctx, card)

        # Update combat tracking
        self.state.cards_played_this_turn += 1
        if card.card_type.value == "ATTACK":
            self.state.attacks_played_this_turn += 1
            ctx.set_last_card_type("ATTACK")
        elif card.card_type.value == "SKILL":
            self.state.skills_played_this_turn += 1
            ctx.set_last_card_type("SKILL")
        elif card.card_type.value == "POWER":
            self.state.powers_played_this_turn += 1
            ctx.set_last_card_type("POWER")

        return result

    def _execute_damage(
        self,
        ctx: EffectContext,
        card: Card,
        target: Optional[EnemyCombatState],
        result: EffectResult,
    ) -> None:
        """Execute card's base damage."""
        base_damage = card.damage

        # Apply strength
        strength = self.state.player.statuses.get("Strength", 0)
        damage = base_damage + strength

        # Apply Vigor (first attack only)
        vigor = self.state.player.statuses.get("Vigor", 0)
        if vigor > 0:
            damage += vigor
            # Vigor is consumed after first attack
            if self.state.attacks_played_this_turn == 0:
                self.state.player.statuses["Vigor"] = 0

        # Apply Weak
        if self.state.player.is_weak:
            damage = int(damage * 0.75)

        # Apply Pen Nib
        if self.state.has_relic("Pen Nib"):
            counter = self.state.get_relic_counter("Pen Nib", 0)
            if counter >= 9:  # 10th attack
                damage *= 2
                self.state.set_relic_counter("Pen Nib", 0)
            else:
                self.state.set_relic_counter("Pen Nib", counter + 1)

        # Apply stance multiplier
        stance_mult = self.STANCE_DAMAGE_MULT.get(self.state.stance, 1.0)
        damage = int(damage * stance_mult)

        # Apply Wreath of Flame
        wreath = self.state.player.statuses.get("WreathOfFlame", 0)
        if wreath > 0:
            damage += wreath
            self.state.player.statuses["WreathOfFlame"] = 0

        # Check for multi-hit
        hits = 1
        if card.magic_number > 0 and "damage_x_times" in card.effects:
            hits = card.magic_number

        # Determine target(s)
        from ..content.cards import CardTarget
        if card.target == CardTarget.ALL_ENEMY:
            # Hit all enemies
            for enemy in ctx.living_enemies:
                self._deal_hits_to_enemy(ctx, enemy, damage, hits, result)
        elif card.target == CardTarget.ENEMY and target:
            # Single target
            self._deal_hits_to_enemy(ctx, target, damage, hits, result)

    def _deal_hits_to_enemy(
        self,
        ctx: EffectContext,
        enemy: EnemyCombatState,
        damage_per_hit: int,
        hits: int,
        result: EffectResult,
    ) -> None:
        """Deal multiple hits to an enemy."""
        for _ in range(hits):
            if enemy.is_dead:
                break

            # Apply Vulnerable
            actual_damage = damage_per_hit
            if enemy.is_vulnerable:
                actual_damage = int(actual_damage * 1.5)

            # Deal damage
            hp_dealt = ctx.deal_damage_to_enemy(enemy, actual_damage)

            # Trigger on-damage hooks
            for hook in self._on_damage_dealt_hooks:
                hook(ctx, hp_dealt, enemy)

    def _execute_block(
        self,
        ctx: EffectContext,
        card: Card,
        result: EffectResult,
    ) -> None:
        """Execute card's base block."""
        base_block = card.block

        # Apply Dexterity
        dexterity = self.state.player.statuses.get("Dexterity", 0)
        block = base_block + dexterity

        # Apply Frail
        if self.state.player.is_frail:
            block = int(block * 0.75)

        if block > 0:
            ctx.gain_block(block)

    # =========================================================================
    # Special Effect Handlers
    # =========================================================================

    # Effects that are no-ops or handled elsewhere (passive/tracked effects)
    _NOOP_EFFECTS = frozenset([
        "only_attack_in_hand", "cost_reduces_each_turn", "gain_damage_when_retained_4",
        "gains_block_when_retained", "on_stance_change_play_from_discard",
        "on_scry_play_from_discard", "unplayable", "damage_x_times",
        "choose_attack_from_any_class", "play_card_from_draw_twice", "discover_card",
        "put_card_on_bottom_of_draw_cost_0", "search_draw_for_skill", "search_draw_for_attack",
        "deal_50_damage_end_turn", "deal_damage_to_all_after_3_turns", "draw_2_put_1_on_top_of_draw",
        "add_x_random_colorless_cost_0", "put_attacks_from_draw_into_hand", "upgrade_all_cards_in_combat",
        "add_random_skills_to_draw_cost_0", "add_random_attacks_to_draw_cost_0",
        "add_random_colorless_to_hand", "add_random_colorless_each_turn", "auto_play_top_card_each_turn",
        "every_5_cards_deal_damage_to_all", "on_debuff_deal_damage", "gain_intangible_1",
        "lose_3_hp_gain_strength", "if_fatal_permanently_increase_damage", "cannot_be_removed",
        "returns_when_exhausted_or_removed", "limit_3_cards_per_turn", "lose_1_hp_when_other_card_played",
        "lose_3_max_hp_when_removed", "end_of_turn_add_copy_to_draw", "end_of_turn_lose_hp_equal_to_hand_size",
        "end_of_turn_gain_frail_1", "end_of_turn_gain_weak_1", "end_of_turn_take_2_damage",
        "end_of_turn_take_damage", "lose_1_energy_when_drawn", "shuffle_discard_into_draw",
        "reduce_hand_cost_to_1", "reduce_random_card_cost_to_0", "draw_if_no_attacks_in_hand",
        "apply_temp_strength_down", "gain_artifact", "gain_no_block_next_2_turns",
        "exhaust_up_to_x_cards", "if_fatal_gain_gold", "hits_x_times",
    ])

    def _handle_special_effect(
        self,
        effect_name: str,
        ctx: EffectContext,
        card: Card,
        result: EffectResult,
    ) -> bool:
        """Handle special effects that need card context."""
        # Fast path: no-op effects
        if effect_name in self._NOOP_EFFECTS:
            return True

        # Scry effects (scry_N pattern)
        if effect_name.startswith("scry_"):
            try:
                amount = int(effect_name.split("_")[1])
                ctx.scry(amount)
                return True
            except (IndexError, ValueError):
                pass

        # Dispatch table for most effects
        handler = self._EFFECT_HANDLERS.get(effect_name)
        if handler:
            handler(self, ctx, card, result)
            return True

        return False

    def _handle_conditional_last_card(self, ctx: EffectContext, card: Card, result: EffectResult, effect: str):
        """Handle effects that depend on the last card played."""
        last = ctx.get_last_card_type()
        if effect == "if_last_card_attack_gain_energy" and last == "ATTACK":
            ctx.gain_energy(1)
        elif effect == "if_last_card_attack_weak_1" and last == "ATTACK":
            ctx.apply_status_to_target("Weak", 1)
        elif effect == "if_last_card_skill_vulnerable_1" and last == "SKILL":
            ctx.apply_status_to_target("Vulnerable", 1)
        elif effect == "if_last_skill_draw_2" and last == "SKILL":
            ctx.draw_cards(2)

    # Effect handler dispatch table - maps effect name to (self, ctx, card, result) handler
    _EFFECT_HANDLERS = {
        # Conditional effects
        "if_last_card_attack_gain_energy": lambda s, c, cd, r: c.gain_energy(1) if c.get_last_card_type() == "ATTACK" else None,
        "if_last_card_attack_weak_1": lambda s, c, cd, r: c.apply_status_to_target("Weak", 1) if c.get_last_card_type() == "ATTACK" else None,
        "if_last_card_skill_vulnerable_1": lambda s, c, cd, r: c.apply_status_to_target("Vulnerable", 1) if c.get_last_card_type() == "SKILL" else None,
        "if_last_skill_draw_2": lambda s, c, cd, r: c.draw_cards(2) if c.get_last_card_type() == "SKILL" else None,
        "if_in_wrath_extra_block_6": lambda s, c, cd, r: c.gain_block(9 if c.is_upgraded else 6) if c.stance == "Wrath" else None,
        "if_enemy_attacking_enter_calm": lambda s, c, cd, r: c.change_stance("Calm") if c.is_enemy_attacking() else None,

        # Calm/Wrath conditionals
        "if_calm_draw_3_else_calm": lambda s, c, cd, r: c.draw_cards(4 if c.is_upgraded else 3) if c.stance == "Calm" else c.change_stance("Calm"),
        "if_wrath_gain_mantra_else_wrath": lambda s, c, cd, r: c.gain_mantra(5 if c.is_upgraded else 3) if c.stance == "Wrath" else c.change_stance("Wrath"),

        # Damage effects
        "damage_per_enemy": lambda s, c, cd, r: c.deal_damage_to_enemy(c.target, cd.damage * len(c.living_enemies)) if c.target else None,
        "damage_equals_draw_pile_size": lambda s, c, cd, r: c.deal_damage_to_enemy(c.target, len(c.draw_pile)) if c.target else None,
        "damage_plus_mantra_gained": lambda s, c, cd, r: c.deal_damage_to_enemy(c.target, c.extra_data.get("total_mantra_gained", 0)) if c.target else None,
        "damage_random_x_times": lambda s, c, cd, r: [c.deal_damage_to_random_enemy(cd.damage) for _ in range(cd.magic_number if cd.magic_number > 0 else 5)],

        # Block effects
        "gain_block_equal_unblocked_damage": lambda s, c, cd, r: (r.extra.__setitem__("wallop_block", c.damage_dealt), c.gain_block(c.damage_dealt)),
        "gain_block_per_card_in_hand": lambda s, c, cd, r: c.gain_block((cd.magic_number if cd.magic_number > 0 else 3) * len(c.hand)),

        # Card generation
        "add_insight_to_draw": lambda s, c, cd, r: c.add_card_to_draw_pile("Insight+" if c.is_upgraded else "Insight", "top"),
        "add_smite_to_hand": lambda s, c, cd, r: c.add_card_to_hand("Smite+" if c.is_upgraded else "Smite"),
        "add_safety_to_hand": lambda s, c, cd, r: c.add_card_to_hand("Safety+" if c.is_upgraded else "Safety"),
        "add_through_violence_to_draw": lambda s, c, cd, r: c.add_card_to_draw_pile("ThroughViolence+" if c.is_upgraded else "ThroughViolence", "top"),
        "shuffle_beta_into_draw": lambda s, c, cd, r: c.add_card_to_draw_pile("Beta+" if c.is_upgraded else "Beta", "random"),
        "shuffle_omega_into_draw": lambda s, c, cd, r: c.add_card_to_draw_pile("Omega", "random"),
        "add_expunger_to_hand": lambda s, c, cd, r: (c.add_card_to_hand("Expunger"), c.extra_data.__setitem__("expunger_x", c.energy_spent)),

        # Energy effects
        "gain_1_energy": lambda s, c, cd, r: c.gain_energy(2 if c.is_upgraded else 1),
        "end_turn": lambda s, c, cd, r: c.end_turn(),

        # Mantra effects
        "gain_mantra": lambda s, c, cd, r: c.gain_mantra(cd.magic_number if cd.magic_number > 0 else 2),
        "gain_mantra_add_insight": lambda s, c, cd, r: (c.gain_mantra(cd.magic_number if cd.magic_number > 0 else 3), c.add_card_to_draw_pile("Insight+" if c.is_upgraded else "Insight", "random")),
        "scry": lambda s, c, cd, r: c.scry(cd.magic_number if cd.magic_number > 0 else 3),

        # Status applications
        "apply_mark": lambda s, c, cd, r: c.apply_status_to_target("Mark", cd.magic_number if cd.magic_number > 0 else 8),
        "apply_block_return": lambda s, c, cd, r: c.apply_status_to_target("BlockReturn", cd.magic_number if cd.magic_number > 0 else 2),
        "apply_weak": lambda s, c, cd, r: c.apply_status_to_target("Weak", cd.magic_number if cd.magic_number > 0 else 2),
        "apply_vulnerable": lambda s, c, cd, r: c.apply_status_to_target("Vulnerable", cd.magic_number if cd.magic_number > 0 else 2),

        # Power card effects (apply statuses to self)
        "on_stance_change_gain_block": lambda s, c, cd, r: c.apply_status_to_player("MentalFortress", cd.magic_number if cd.magic_number > 0 else 4),
        "on_scry_gain_block": lambda s, c, cd, r: c.apply_status_to_player("Nirvana", cd.magic_number if cd.magic_number > 0 else 3),
        "on_wrath_draw": lambda s, c, cd, r: c.apply_status_to_player("Rushdown", cd.magic_number if cd.magic_number > 0 else 2),
        "if_calm_end_turn_gain_block": lambda s, c, cd, r: c.apply_status_to_player("LikeWater", cd.magic_number if cd.magic_number > 0 else 5),
        "gain_mantra_each_turn": lambda s, c, cd, r: c.apply_status_to_player("Devotion", cd.magic_number if cd.magic_number > 0 else 2),
        "retained_cards_cost_less": lambda s, c, cd, r: c.apply_status_to_player("Establishment", 1),
        "scry_each_turn": lambda s, c, cd, r: c.apply_status_to_player("Foresight", cd.magic_number if cd.magic_number > 0 else 3),
        "add_smite_each_turn": lambda s, c, cd, r: c.apply_status_to_player("BattleHymn", cd.magic_number if cd.magic_number > 0 else 1),
        "add_insight_end_turn": lambda s, c, cd, r: c.apply_status_to_player("Study", 1),
        "gain_energy_each_turn_stacking": lambda s, c, cd, r: c.apply_status_to_player("DevaForm", 1),
        "created_cards_upgraded": lambda s, c, cd, r: c.apply_status_to_player("MasterReality", 1),
        "next_attack_plus_damage": lambda s, c, cd, r: c.apply_status_to_player("WreathOfFlame", cd.magic_number if cd.magic_number > 0 else 5),
        "wrath_next_turn_draw_next_turn": lambda s, c, cd, r: c.apply_status_to_player("SimmeringFury", cd.magic_number if cd.magic_number > 0 else 2),
        "free_attack_next_turn": lambda s, c, cd, r: c.apply_status_to_player("FreeAttackPower", 1),
        "block_gain_applies_weak": lambda s, c, cd, r: c.apply_status_to_player("WaveOfTheHand", cd.magic_number if cd.magic_number > 0 else 1),
        "die_next_turn": lambda s, c, cd, r: c.apply_status_to_player("Blasphemy", 1),
        "deal_50_damage_end_turn_power": lambda s, c, cd, r: c.apply_status_to_player("Omega", 50),

        # Stance changes
        "enter_divinity": lambda s, c, cd, r: c.change_stance("Divinity"),
        "enter_calm": lambda s, c, cd, r: c.change_stance("Calm"),

        # Utility effects
        "draw_until_hand_full": lambda s, c, cd, r: c.draw_cards(max(0, 10 - len(c.hand))),
        "take_extra_turn": lambda s, c, cd, r: c.extra_data.__setitem__("extra_turn", True),
        "heal_magic_number": lambda s, c, cd, r: c.heal_player(cd.magic_number if cd.magic_number > 0 else 4),
        "if_enemy_hp_below_kill": lambda s, c, cd, r: setattr(c.target, 'hp', 0) if c.target and c.target.hp <= (cd.magic_number if cd.magic_number > 0 else 30) else None,
        "if_fatal_upgrade_random_card": lambda s, c, cd, r: c.extra_data.__setitem__("fatal_upgrade", True),
        "put_x_miracles_on_draw": lambda s, c, cd, r: c.apply_status_to_player("CollectMiracles", (c.energy_spent + 1) if c.is_upgraded else c.energy_spent),

        # Fasting
        "gain_strength_and_dex_lose_focus": lambda s, c, cd, r: (c.apply_status_to_player("Strength", 4 if c.is_upgraded else 3), c.apply_status_to_player("Dexterity", 4 if c.is_upgraded else 3), setattr(c.state, 'max_energy', max(0, c.state.max_energy - 1))),

        # Wish
        "choose_plated_armor_or_strength_or_gold": lambda s, c, cd, r: s._handle_wish(c, cd),

        # Trigger all marks
        "trigger_all_marks": lambda s, c, cd, r: [c.deal_damage_to_enemy(e, e.statuses.get("Mark", 0)) for e in c.living_enemies if e.statuses.get("Mark", 0) > 0],

        # Meditate
        "put_cards_from_discard_to_hand": lambda s, c, cd, r: [c.move_card_from_discard_to_hand(c.discard_pile[0]) for _ in range(min(cd.magic_number if cd.magic_number > 0 else 1, len(c.discard_pile))) if c.discard_pile and len(c.hand) < 10],
    }

    def _handle_wish(self, ctx: EffectContext, card: Card):
        """Handle Wish card choice."""
        choice = ctx.extra_data.get("wish_choice", 1)
        amount = 4 if ctx.is_upgraded else 3
        if choice == 0:
            ctx.apply_status_to_player("Plated Armor", amount)
        elif choice == 2:
            ctx.extra_data["gold_gained"] = 75 if ctx.is_upgraded else 50
        else:
            ctx.apply_status_to_player("Strength", amount)

    # =========================================================================
    # Hook Registration
    # =========================================================================

    def register_on_card_played(
        self, hook: Callable[[EffectContext, Card], None]
    ) -> None:
        """Register a hook called when any card is played."""
        self._on_card_played_hooks.append(hook)

    def register_on_damage_dealt(
        self, hook: Callable[[EffectContext, int, EnemyCombatState], None]
    ) -> None:
        """Register a hook called when damage is dealt."""
        self._on_damage_dealt_hooks.append(hook)

    def register_on_stance_change(
        self, hook: Callable[[EffectContext, str, str], None]
    ) -> None:
        """Register a hook called when stance changes."""
        self._on_stance_change_hooks.append(hook)

    # =========================================================================
    # Effect Execution Utilities
    # =========================================================================

    def execute_effect(
        self,
        effect_name: str,
        target_idx: int = -1,
        card: Optional[Card] = None,
    ) -> EffectResult:
        """
        Execute a single named effect.

        Args:
            effect_name: Effect string like "draw_2"
            target_idx: Target enemy index
            card: Optional card context

        Returns:
            EffectResult
        """
        result = EffectResult(success=True)

        target = None
        if 0 <= target_idx < len(self.state.enemies):
            target = self.state.enemies[target_idx]

        ctx = EffectContext(
            state=self.state,
            card=card,
            target=target,
            target_idx=target_idx,
            is_upgraded=card.upgraded if card else False,
            magic_number=card.magic_number if card and card.magic_number > 0 else 0,
        )

        if execute_effect(effect_name, ctx):
            result.effects_executed.append(effect_name)
        elif card and self._handle_special_effect(effect_name, ctx, card, result):
            result.effects_executed.append(effect_name)
        else:
            result.success = False
            result.errors.append(f"Unknown effect: {effect_name}")

        # Collect results
        result.damage_dealt = ctx.damage_dealt
        result.block_gained = ctx.block_gained
        result.energy_gained = ctx.energy_gained
        result.cards_drawn = ctx.cards_drawn
        result.cards_discarded = ctx.cards_discarded
        result.cards_exhausted = ctx.cards_exhausted
        result.statuses_applied = ctx.statuses_applied
        result.mantra_gained = ctx.mantra_gained
        result.scried_cards = ctx.scried_cards

        return result

    def apply_start_of_turn_effects(self) -> EffectResult:
        """Apply all start-of-turn effects."""
        result = EffectResult(success=True)

        ctx = EffectContext(state=self.state)

        # Foresight - scry at start of turn
        foresight = ctx.get_player_status("Foresight")
        if foresight > 0:
            ctx.scry(foresight)
            result.scried_cards = ctx.scried_cards
            result.effects_executed.append(f"foresight_scry_{foresight}")

        # Battle Hymn - add Smite to hand
        battle_hymn = ctx.get_player_status("BattleHymn")
        if battle_hymn > 0:
            for _ in range(battle_hymn):
                ctx.add_card_to_hand("Smite")
            result.effects_executed.append(f"battle_hymn_{battle_hymn}")

        # Deva Form - gain energy
        deva_form = ctx.get_player_status("DevaForm")
        if deva_form > 0:
            ctx.gain_energy(deva_form)
            # Increment for next turn
            ctx.apply_status_to_player("DevaForm", 1)
            result.energy_gained += deva_form
            result.effects_executed.append(f"deva_form_{deva_form}")

        # Devotion - gain mantra
        devotion = ctx.get_player_status("Devotion")
        if devotion > 0:
            mantra_result = ctx.gain_mantra(devotion)
            result.mantra_gained += devotion
            result.effects_executed.append(f"devotion_{devotion}")
            if mantra_result.get("divinity_triggered"):
                result.stance_changed_to = "Divinity"

        # Simmering Fury - enter Wrath and draw
        simmering = ctx.get_player_status("SimmeringFury")
        if simmering > 0:
            ctx.change_stance("Wrath")
            ctx.draw_cards(simmering)
            ctx.remove_status_from_player("SimmeringFury")
            result.stance_changed_to = "Wrath"
            result.effects_executed.append(f"simmering_fury_{simmering}")

        return result

    def apply_end_of_turn_effects(self) -> EffectResult:
        """Apply all end-of-turn effects."""
        result = EffectResult(success=True)

        ctx = EffectContext(state=self.state)

        # Like Water - gain block if in Calm
        if self.state.stance == "Calm":
            like_water = ctx.get_player_status("LikeWater")
            if like_water > 0:
                ctx.gain_block(like_water)
                result.block_gained += like_water
                result.effects_executed.append(f"like_water_{like_water}")

        # Divinity auto-exit
        if self.state.stance == "Divinity":
            ctx.change_stance("Neutral")
            result.stance_changed_to = "Neutral"
            result.effects_executed.append("divinity_exit")

        # Study - add Insight to draw
        study = ctx.get_player_status("Study")
        if study > 0:
            for _ in range(study):
                ctx.add_card_to_draw_pile("Insight", "random")
            result.effects_executed.append(f"study_{study}")

        # Blasphemy - die at end of turn
        if ctx.get_player_status("Blasphemy") > 0:
            self.state.player.hp = 0
            result.effects_executed.append("blasphemy_death")

        return result


# =============================================================================
# Factory Functions
# =============================================================================

def create_executor(state: CombatState) -> EffectExecutor:
    """Create a new EffectExecutor for a combat state."""
    return EffectExecutor(state)
