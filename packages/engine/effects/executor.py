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

    def _handle_special_effect(
        self,
        effect_name: str,
        ctx: EffectContext,
        card: Card,
        result: EffectResult,
    ) -> bool:
        """
        Handle special effects that need card context.

        These are effects that can't be handled by the generic registry
        because they need access to card-specific information.
        """
        # Conditional effects based on last card type
        if effect_name == "if_last_card_attack_gain_energy":
            if ctx.get_last_card_type() == "ATTACK":
                ctx.gain_energy(1)
            return True

        if effect_name == "if_last_card_attack_weak_1":
            if ctx.get_last_card_type() == "ATTACK":
                ctx.apply_status_to_target("Weak", 1)
            return True

        if effect_name == "if_last_card_skill_vulnerable_1":
            if ctx.get_last_card_type() == "SKILL":
                ctx.apply_status_to_target("Vulnerable", 1)
            return True

        if effect_name == "if_last_skill_draw_2":
            if ctx.get_last_card_type() == "SKILL":
                ctx.draw_cards(2)
            return True

        # Wrath-conditional effects
        if effect_name == "if_in_wrath_extra_block_6":
            if ctx.stance == "Wrath":
                ctx.gain_block(6 if not ctx.is_upgraded else 9)
            return True

        # Calm conditional
        if effect_name == "if_calm_draw_3_else_calm":
            if ctx.stance == "Calm":
                amount = 4 if ctx.is_upgraded else 3
                ctx.draw_cards(amount)
            else:
                ctx.change_stance("Calm")
            return True

        # Wrath/Mantra conditional
        if effect_name == "if_wrath_gain_mantra_else_wrath":
            if ctx.stance == "Wrath":
                amount = 5 if ctx.is_upgraded else 3
                ctx.gain_mantra(amount)
            else:
                ctx.change_stance("Wrath")
            return True

        # Enemy attacking conditional
        if effect_name == "if_enemy_attacking_enter_calm":
            if ctx.is_enemy_attacking():
                ctx.change_stance("Calm")
            return True

        # Damage per enemy
        if effect_name == "damage_per_enemy":
            # Bowling Bash - damage equals number of enemies
            num_enemies = len(ctx.living_enemies)
            base_damage = card.damage * num_enemies
            if ctx.target:
                ctx.deal_damage_to_enemy(ctx.target, base_damage)
            return True

        # Damage equal to draw pile
        if effect_name == "damage_equals_draw_pile_size":
            damage = len(ctx.draw_pile)
            if ctx.target:
                ctx.deal_damage_to_enemy(ctx.target, damage)
            return True

        # Gain block equal to unblocked damage
        if effect_name == "gain_block_equal_unblocked_damage":
            # Wallop - gain block equal to unblocked damage dealt
            # This is tracked during damage calculation
            result.extra["wallop_block"] = ctx.damage_dealt
            ctx.gain_block(ctx.damage_dealt)
            return True

        # Gain block per card in hand
        if effect_name == "gain_block_per_card_in_hand":
            per_card = card.magic_number if card.magic_number > 0 else 3
            block = per_card * len(ctx.hand)
            ctx.gain_block(block)
            return True

        # End turn effect
        if effect_name == "end_turn":
            ctx.end_turn()
            return True

        # Only attack in hand
        if effect_name == "only_attack_in_hand":
            # Signature Move - check this is only attack
            # For now just execute, validation happens elsewhere
            return True

        # Retain-related effects
        if effect_name == "cost_reduces_each_turn":
            # Sands of Time - cost reduces while retained
            return True

        if effect_name == "gain_damage_when_retained_4":
            # Windmill Strike - tracked separately
            return True

        if effect_name == "gains_block_when_retained":
            # Perseverance - tracked separately
            return True

        # Card generation effects
        if effect_name == "add_insight_to_draw":
            card_id = "Insight+" if ctx.is_upgraded else "Insight"
            ctx.add_card_to_draw_pile(card_id, "top")
            return True

        if effect_name == "add_smite_to_hand":
            card_id = "Smite+" if ctx.is_upgraded else "Smite"
            ctx.add_card_to_hand(card_id)
            return True

        if effect_name == "add_safety_to_hand":
            card_id = "Safety+" if ctx.is_upgraded else "Safety"
            ctx.add_card_to_hand(card_id)
            return True

        if effect_name == "add_through_violence_to_draw":
            card_id = "ThroughViolence+" if ctx.is_upgraded else "ThroughViolence"
            ctx.add_card_to_draw_pile(card_id, "top")
            return True

        # Miracle energy gain
        if effect_name == "gain_1_energy":
            amount = 2 if ctx.is_upgraded else 1
            ctx.gain_energy(amount)
            return True

        # Mark (Pressure Points)
        if effect_name == "apply_mark":
            amount = card.magic_number if card.magic_number > 0 else 8
            ctx.apply_status_to_target("Mark", amount)
            return True

        if effect_name == "trigger_all_marks":
            # Deal damage to all enemies equal to their Mark
            for enemy in ctx.living_enemies:
                mark = enemy.statuses.get("Mark", 0)
                if mark > 0:
                    ctx.deal_damage_to_enemy(enemy, mark)
            return True

        # Block return (Talk to the Hand)
        if effect_name == "apply_block_return":
            amount = card.magic_number if card.magic_number > 0 else 2
            ctx.apply_status_to_target("BlockReturn", amount)
            return True

        # Mantra effects
        if effect_name == "gain_mantra":
            amount = card.magic_number if card.magic_number > 0 else 2
            ctx.gain_mantra(amount)
            return True

        if effect_name == "gain_mantra_add_insight":
            # Pray
            amount = card.magic_number if card.magic_number > 0 else 3
            ctx.gain_mantra(amount)
            card_id = "Insight+" if ctx.is_upgraded else "Insight"
            ctx.add_card_to_draw_pile(card_id, "random")
            return True

        # Scry effects
        if effect_name == "scry":
            amount = card.magic_number if card.magic_number > 0 else 3
            ctx.scry(amount)
            return True

        if effect_name.startswith("scry_"):
            # Parse scry_N
            try:
                amount = int(effect_name.split("_")[1])
                ctx.scry(amount)
                return True
            except (IndexError, ValueError):
                pass

        # Power effects (these apply statuses to self)
        if effect_name == "on_stance_change_gain_block":
            amount = card.magic_number if card.magic_number > 0 else 4
            ctx.apply_status_to_player("MentalFortress", amount)
            return True

        if effect_name == "on_scry_gain_block":
            amount = card.magic_number if card.magic_number > 0 else 3
            ctx.apply_status_to_player("Nirvana", amount)
            return True

        if effect_name == "on_wrath_draw":
            amount = card.magic_number if card.magic_number > 0 else 2
            ctx.apply_status_to_player("Rushdown", amount)
            return True

        if effect_name == "if_calm_end_turn_gain_block":
            amount = card.magic_number if card.magic_number > 0 else 5
            ctx.apply_status_to_player("LikeWater", amount)
            return True

        if effect_name == "gain_mantra_each_turn":
            amount = card.magic_number if card.magic_number > 0 else 2
            ctx.apply_status_to_player("Devotion", amount)
            return True

        if effect_name == "retained_cards_cost_less":
            ctx.apply_status_to_player("Establishment", 1)
            return True

        if effect_name == "scry_each_turn":
            amount = card.magic_number if card.magic_number > 0 else 3
            ctx.apply_status_to_player("Foresight", amount)
            return True

        if effect_name == "add_smite_each_turn":
            amount = card.magic_number if card.magic_number > 0 else 1
            ctx.apply_status_to_player("BattleHymn", amount)
            return True

        if effect_name == "add_insight_end_turn":
            ctx.apply_status_to_player("Study", 1)
            return True

        if effect_name == "gain_energy_each_turn_stacking":
            ctx.apply_status_to_player("DevaForm", 1)
            return True

        if effect_name == "created_cards_upgraded":
            ctx.apply_status_to_player("MasterReality", 1)
            return True

        if effect_name == "next_attack_plus_damage":
            amount = card.magic_number if card.magic_number > 0 else 5
            ctx.apply_status_to_player("WreathOfFlame", amount)
            return True

        # Special cards
        if effect_name == "enter_divinity":
            ctx.change_stance("Divinity")
            return True

        if effect_name == "die_next_turn":
            ctx.apply_status_to_player("Blasphemy", 1)
            return True

        if effect_name == "shuffle_beta_into_draw":
            card_id = "Beta+" if ctx.is_upgraded else "Beta"
            ctx.add_card_to_draw_pile(card_id, "random")
            return True

        if effect_name == "shuffle_omega_into_draw":
            card_id = "Omega"  # Omega doesn't upgrade
            ctx.add_card_to_draw_pile(card_id, "random")
            return True

        if effect_name == "draw_until_hand_full":
            cards_to_draw = 10 - len(ctx.hand)
            if cards_to_draw > 0:
                ctx.draw_cards(cards_to_draw)
            return True

        if effect_name == "take_extra_turn":
            ctx.extra_data["extra_turn"] = True
            return True

        # Meditate
        if effect_name == "put_cards_from_discard_to_hand":
            # In simulation, just move some cards
            amount = card.magic_number if card.magic_number > 0 else 1
            for _ in range(amount):
                if ctx.discard_pile and len(ctx.hand) < 10:
                    # Move first card from discard (in real game player chooses)
                    card_id = ctx.discard_pile[0]
                    ctx.move_card_from_discard_to_hand(card_id)
            return True

        if effect_name == "enter_calm":
            ctx.change_stance("Calm")
            return True

        # Damage plus mantra gained
        if effect_name == "damage_plus_mantra_gained":
            # Brilliance - damage increased by total mantra gained this combat
            # This would need tracking at combat level
            total_mantra = ctx.extra_data.get("total_mantra_gained", 0)
            extra_damage = total_mantra
            if ctx.target:
                ctx.deal_damage_to_enemy(ctx.target, extra_damage)
            return True

        # Judgment - kill if HP below threshold
        if effect_name == "if_enemy_hp_below_kill":
            threshold = card.magic_number if card.magic_number > 0 else 30
            if ctx.target and ctx.target.hp <= threshold:
                ctx.target.hp = 0
            return True

        # Lesson Learned - upgrade random card on kill
        if effect_name == "if_fatal_upgrade_random_card":
            ctx.extra_data["fatal_upgrade"] = True
            return True

        # Ragnarok - damage random enemies
        if effect_name == "damage_random_x_times":
            damage = card.damage
            hits = card.magic_number if card.magic_number > 0 else 5
            for _ in range(hits):
                ctx.deal_damage_to_random_enemy(damage)
            return True

        # Status applications
        if effect_name == "apply_weak":
            amount = card.magic_number if card.magic_number > 0 else 2
            ctx.apply_status_to_target("Weak", amount)
            return True

        if effect_name == "apply_vulnerable":
            amount = card.magic_number if card.magic_number > 0 else 2
            ctx.apply_status_to_target("Vulnerable", amount)
            return True

        # Heal effects
        if effect_name == "heal_magic_number":
            amount = card.magic_number if card.magic_number > 0 else 4
            ctx.heal_player(amount)
            return True

        # Simmering Fury
        if effect_name == "wrath_next_turn_draw_next_turn":
            # Apply a status that triggers next turn
            draw_amount = card.magic_number if card.magic_number > 0 else 2
            ctx.apply_status_to_player("SimmeringFury", draw_amount)
            return True

        # Meditate special
        if effect_name == "free_attack_next_turn":
            ctx.apply_status_to_player("FreeAttackPower", 1)
            return True

        # Wave of the Hand
        if effect_name == "block_gain_applies_weak":
            amount = card.magic_number if card.magic_number > 0 else 1
            ctx.apply_status_to_player("WaveOfTheHand", amount)
            return True

        # On-trigger effects (passive, handled by combat loop)
        if effect_name in [
            "on_stance_change_play_from_discard",
            "on_scry_play_from_discard",
        ]:
            return True

        # Unplayable effects (for curses/status)
        if effect_name == "unplayable":
            return True

        # Effects that need external systems
        if effect_name in [
            "choose_plated_armor_or_strength_or_gold",
            "choose_attack_from_any_class",
            "play_card_from_draw_twice",
            "put_x_miracles_on_draw",
            "add_expunger_to_hand",
            "discover_card",
            "put_card_on_bottom_of_draw_cost_0",
            "search_draw_for_skill",
            "search_draw_for_attack",
            "deal_50_damage_end_turn",
            "deal_damage_to_all_after_3_turns",
            "draw_2_put_1_on_top_of_draw",
            "add_x_random_colorless_cost_0",
            "put_attacks_from_draw_into_hand",
            "upgrade_all_cards_in_combat",
            "add_random_skills_to_draw_cost_0",
            "add_random_attacks_to_draw_cost_0",
            "add_random_colorless_to_hand",
            "add_random_colorless_each_turn",
            "auto_play_top_card_each_turn",
            "every_5_cards_deal_damage_to_all",
            "on_debuff_deal_damage",
            "gain_intangible_1",
            "lose_3_hp_gain_strength",
            "if_fatal_permanently_increase_damage",
            "cannot_be_removed",
            "returns_when_exhausted_or_removed",
            "limit_3_cards_per_turn",
            "lose_1_hp_when_other_card_played",
            "lose_3_max_hp_when_removed",
            "end_of_turn_add_copy_to_draw",
            "end_of_turn_lose_hp_equal_to_hand_size",
            "end_of_turn_gain_frail_1",
            "end_of_turn_gain_weak_1",
            "end_of_turn_take_2_damage",
            "end_of_turn_take_damage",
            "lose_1_energy_when_drawn",
            "shuffle_discard_into_draw",
            "reduce_hand_cost_to_1",
            "reduce_random_card_cost_to_0",
            "draw_if_no_attacks_in_hand",
            "apply_temp_strength_down",
            "gain_artifact",
            "gain_no_block_next_2_turns",
            "exhaust_up_to_x_cards",
            "if_fatal_gain_gold",
            "gain_strength_and_dex_lose_focus",
            "hits_x_times",
        ]:
            # These need more complex handling
            return True

        return False

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
