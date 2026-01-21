"""
Watcher Card Effect Implementations.

This module registers all Watcher card effects using the effect registry.
Effects are implemented as pure functions that modify the EffectContext.

The effects are organized by category:
- Basic effects (draw, block, damage)
- Stance effects
- Mantra effects
- Scry effects
- Card manipulation effects
- Status application effects
- Special card effects
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from .registry import (
    effect, effect_simple, effect_custom, EffectContext
)

if TYPE_CHECKING:
    pass


# =============================================================================
# Basic Effects
# =============================================================================

@effect_simple("draw_cards")
def draw_cards_effect(ctx: EffectContext) -> None:
    """Draw cards based on card's magic number."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 1
    ctx.draw_cards(amount)


@effect("draw")
def draw_effect(ctx: EffectContext, amount: int) -> None:
    """Draw specific number of cards."""
    ctx.draw_cards(amount)


@effect_simple("draw_1")
def draw_1(ctx: EffectContext) -> None:
    """Draw 1 card."""
    ctx.draw_cards(1)


@effect_simple("draw_2")
def draw_2(ctx: EffectContext) -> None:
    """Draw 2 cards."""
    ctx.draw_cards(2)


@effect_simple("draw_3")
def draw_3(ctx: EffectContext) -> None:
    """Draw 3 cards."""
    ctx.draw_cards(3)


@effect("gain_energy")
def gain_energy_effect(ctx: EffectContext, amount: int) -> None:
    """Gain energy."""
    ctx.gain_energy(amount)


@effect("gain_block")
def gain_block_effect(ctx: EffectContext, amount: int) -> None:
    """Gain block."""
    ctx.gain_block(amount)


@effect_simple("gain_block_2")
def gain_block_2(ctx: EffectContext) -> None:
    """Gain 2 block (Just Lucky)."""
    amount = 3 if ctx.is_upgraded else 2
    ctx.gain_block(amount)


# =============================================================================
# Stance Effects
# =============================================================================

@effect_simple("enter_wrath")
def enter_wrath(ctx: EffectContext) -> None:
    """Enter Wrath stance."""
    ctx.change_stance("Wrath")


@effect_simple("enter_calm")
def enter_calm(ctx: EffectContext) -> None:
    """Enter Calm stance."""
    ctx.change_stance("Calm")


@effect_simple("enter_divinity")
def enter_divinity(ctx: EffectContext) -> None:
    """Enter Divinity stance."""
    ctx.change_stance("Divinity")


@effect_simple("exit_stance")
def exit_stance(ctx: EffectContext) -> None:
    """Exit to Neutral stance."""
    ctx.exit_stance()


# =============================================================================
# Mantra Effects
# =============================================================================

@effect_simple("gain_mantra")
def gain_mantra_effect(ctx: EffectContext) -> None:
    """Gain mantra based on card's magic number."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 2
    ctx.gain_mantra(amount)


@effect("mantra")
def mantra_with_amount(ctx: EffectContext, amount: int) -> None:
    """Gain specific amount of mantra."""
    ctx.gain_mantra(amount)


@effect_simple("gain_mantra_2")
def gain_mantra_2(ctx: EffectContext) -> None:
    """Gain 2 mantra (Prostrate base)."""
    amount = 3 if ctx.is_upgraded else 2
    ctx.gain_mantra(amount)


@effect_simple("gain_mantra_3")
def gain_mantra_3(ctx: EffectContext) -> None:
    """Gain 3 mantra (Pray base)."""
    amount = 4 if ctx.is_upgraded else 3
    ctx.gain_mantra(amount)


@effect_simple("gain_mantra_5")
def gain_mantra_5(ctx: EffectContext) -> None:
    """Gain 5 mantra (Worship base)."""
    amount = 8 if ctx.is_upgraded else 5
    ctx.gain_mantra(amount)


# =============================================================================
# Scry Effects
# =============================================================================

@effect_simple("scry")
def scry_effect(ctx: EffectContext) -> None:
    """Scry using card's magic number."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 1
    ctx.scry(amount)


@effect("scry_n")
def scry_n_effect(ctx: EffectContext, amount: int) -> None:
    """Scry X cards - look at top X cards and discard any."""
    ctx.scry(amount)


@effect_simple("scry_1")
def scry_1(ctx: EffectContext) -> None:
    """Scry 1 card (Just Lucky)."""
    amount = 2 if ctx.is_upgraded else 1
    ctx.scry(amount)


@effect_simple("scry_2")
def scry_2(ctx: EffectContext) -> None:
    """Scry 2 cards (Cut Through Fate base)."""
    amount = 3 if ctx.is_upgraded else 2
    ctx.scry(amount)


@effect_simple("scry_3")
def scry_3(ctx: EffectContext) -> None:
    """Scry 3 cards (Third Eye base)."""
    amount = 5 if ctx.is_upgraded else 3
    ctx.scry(amount)


# =============================================================================
# Status Application Effects
# =============================================================================

@effect("apply_weak")
def apply_weak_effect(ctx: EffectContext, amount: int) -> None:
    """Apply Weak to target."""
    ctx.apply_status_to_target("Weak", amount)


@effect("apply_vulnerable")
def apply_vulnerable_effect(ctx: EffectContext, amount: int) -> None:
    """Apply Vulnerable to target."""
    ctx.apply_status_to_target("Vulnerable", amount)


@effect("apply_strength")
def apply_strength_effect(ctx: EffectContext, amount: int) -> None:
    """Apply Strength to player."""
    ctx.apply_status_to_player("Strength", amount)


@effect("apply_dexterity")
def apply_dexterity_effect(ctx: EffectContext, amount: int) -> None:
    """Apply Dexterity to player."""
    ctx.apply_status_to_player("Dexterity", amount)


@effect_simple("apply_weak_1")
def apply_weak_1(ctx: EffectContext) -> None:
    """Apply 1 Weak to target (Sash Whip conditional)."""
    amount = 2 if ctx.is_upgraded else 1
    ctx.apply_status_to_target("Weak", amount)


@effect_simple("apply_vulnerable_1")
def apply_vulnerable_1(ctx: EffectContext) -> None:
    """Apply 1 Vulnerable to target (Crush Joints conditional)."""
    amount = 2 if ctx.is_upgraded else 1
    ctx.apply_status_to_target("Vulnerable", amount)


# =============================================================================
# Card Generation Effects
# =============================================================================

@effect_simple("add_miracle_to_hand")
def add_miracle_to_hand(ctx: EffectContext) -> None:
    """Add a Miracle to hand (Pure Water relic)."""
    card_id = "Miracle+" if ctx.is_upgraded else "Miracle"
    ctx.add_card_to_hand(card_id)


@effect_simple("add_insight_to_draw")
def add_insight_to_draw(ctx: EffectContext) -> None:
    """Add an Insight to top of draw pile (Evaluate)."""
    card_id = "Insight+" if ctx.is_upgraded else "Insight"
    ctx.add_card_to_draw_pile(card_id, "top")


@effect_simple("add_insight_to_draw_random")
def add_insight_to_draw_random(ctx: EffectContext) -> None:
    """Add an Insight to draw pile at random position (Pray)."""
    card_id = "Insight+" if ctx.is_upgraded else "Insight"
    ctx.add_card_to_draw_pile(card_id, "random")


@effect_simple("add_smite_to_hand")
def add_smite_to_hand(ctx: EffectContext) -> None:
    """Add a Smite to hand (Carve Reality)."""
    card_id = "Smite+" if ctx.is_upgraded else "Smite"
    ctx.add_card_to_hand(card_id)


@effect_simple("add_safety_to_hand")
def add_safety_to_hand(ctx: EffectContext) -> None:
    """Add a Safety to hand (Deceive Reality)."""
    card_id = "Safety+" if ctx.is_upgraded else "Safety"
    ctx.add_card_to_hand(card_id)


@effect_simple("add_through_violence_to_draw")
def add_through_violence_to_draw(ctx: EffectContext) -> None:
    """Add Through Violence to top of draw pile (Reach Heaven)."""
    card_id = "ThroughViolence+" if ctx.is_upgraded else "ThroughViolence"
    ctx.add_card_to_draw_pile(card_id, "top")


@effect_simple("shuffle_beta_into_draw")
def shuffle_beta_into_draw(ctx: EffectContext) -> None:
    """Shuffle Beta into draw pile (Alpha)."""
    card_id = "Beta+" if ctx.is_upgraded else "Beta"
    ctx.add_card_to_draw_pile(card_id, "random")


@effect_simple("shuffle_omega_into_draw")
def shuffle_omega_into_draw(ctx: EffectContext) -> None:
    """Shuffle Omega into draw pile (Beta)."""
    ctx.add_card_to_draw_pile("Omega", "random")


# =============================================================================
# Conditional Effects
# =============================================================================

@effect_simple("if_last_card_attack_gain_energy")
def if_last_card_attack_gain_energy(ctx: EffectContext) -> None:
    """If last card was Attack, gain 1 energy (Follow-Up)."""
    if ctx.get_last_card_type() == "ATTACK":
        ctx.gain_energy(1)


@effect_simple("if_last_card_attack_weak_1")
def if_last_card_attack_weak_1(ctx: EffectContext) -> None:
    """If last card was Attack, apply Weak (Sash Whip)."""
    if ctx.get_last_card_type() == "ATTACK":
        amount = 2 if ctx.is_upgraded else 1
        ctx.apply_status_to_target("Weak", amount)


@effect_simple("if_last_card_skill_vulnerable_1")
def if_last_card_skill_vulnerable_1(ctx: EffectContext) -> None:
    """If last card was Skill, apply Vulnerable (Crush Joints)."""
    if ctx.get_last_card_type() == "SKILL":
        amount = 2 if ctx.is_upgraded else 1
        ctx.apply_status_to_target("Vulnerable", amount)


@effect_simple("if_last_skill_draw_2")
def if_last_skill_draw_2(ctx: EffectContext) -> None:
    """If last card was Skill, draw 2 (Sanctity)."""
    if ctx.get_last_card_type() == "SKILL":
        amount = 3 if ctx.is_upgraded else 2
        ctx.draw_cards(amount)


@effect_simple("if_in_wrath_extra_block_6")
def if_in_wrath_extra_block(ctx: EffectContext) -> None:
    """If in Wrath, gain extra block (Halt)."""
    if ctx.stance == "Wrath":
        extra = 9 if ctx.is_upgraded else 6
        ctx.gain_block(extra)


@effect_simple("if_calm_draw_3_else_calm")
def if_calm_draw_else_calm(ctx: EffectContext) -> None:
    """If in Calm draw 3, else enter Calm (Inner Peace)."""
    if ctx.stance == "Calm":
        amount = 4 if ctx.is_upgraded else 3
        ctx.draw_cards(amount)
    else:
        ctx.change_stance("Calm")


@effect_simple("if_wrath_gain_mantra_else_wrath")
def if_wrath_gain_mantra_else_wrath(ctx: EffectContext) -> None:
    """If in Wrath gain mantra, else enter Wrath (Indignation)."""
    if ctx.stance == "Wrath":
        amount = 5 if ctx.is_upgraded else 3
        ctx.gain_mantra(amount)
    else:
        ctx.change_stance("Wrath")


@effect_simple("if_enemy_attacking_enter_calm")
def if_enemy_attacking_enter_calm(ctx: EffectContext) -> None:
    """If enemy is attacking, enter Calm (Fear No Evil)."""
    if ctx.is_enemy_attacking():
        ctx.change_stance("Calm")


@effect_simple("if_enemy_hp_below_kill")
def if_enemy_hp_below_kill(ctx: EffectContext) -> None:
    """Kill enemy if HP below threshold (Judgement)."""
    threshold = 40 if ctx.is_upgraded else 30
    if ctx.target and ctx.target.hp <= threshold:
        ctx.target.hp = 0


# =============================================================================
# Special Damage Effects
# =============================================================================

@effect_simple("damage_per_enemy")
def damage_per_enemy(ctx: EffectContext) -> None:
    """
    Deal damage times number of enemies (Bowling Bash).
    The base damage is already calculated, this multiplies by enemy count.
    """
    # Note: This is handled specially in EffectExecutor
    pass


@effect_simple("damage_x_times")
def damage_x_times(ctx: EffectContext) -> None:
    """
    Deal damage X times (Flying Sleeves, Tantrum).
    Handled in EffectExecutor's damage calculation.
    """
    pass


@effect_simple("damage_random_x_times")
def damage_random_x_times(ctx: EffectContext) -> None:
    """Deal damage to random enemies X times (Ragnarok)."""
    if ctx.card:
        damage = ctx.card.damage
        hits = ctx.card.magic_number if ctx.card.magic_number > 0 else 5
        for _ in range(hits):
            ctx.deal_damage_to_random_enemy(damage)


@effect_simple("damage_plus_mantra_gained")
def damage_plus_mantra_gained(ctx: EffectContext) -> None:
    """Deal extra damage equal to mantra gained (Brilliance)."""
    # Total mantra tracked in combat state
    total_mantra = ctx.extra_data.get("total_mantra_gained", 0)
    if ctx.target and total_mantra > 0:
        ctx.deal_damage_to_enemy(ctx.target, total_mantra)


@effect_simple("damage_equals_draw_pile_size")
def damage_equals_draw_pile(ctx: EffectContext) -> None:
    """Deal damage equal to draw pile size (Mind Blast)."""
    damage = len(ctx.draw_pile)
    if ctx.target:
        ctx.deal_damage_to_enemy(ctx.target, damage)


@effect_simple("gain_block_equal_unblocked_damage")
def gain_block_equal_unblocked(ctx: EffectContext) -> None:
    """Gain block equal to unblocked damage dealt (Wallop)."""
    # This is tracked during damage application
    # The damage_dealt on ctx is the HP damage after block
    ctx.gain_block(ctx.damage_dealt)


@effect_simple("gain_block_per_card_in_hand")
def gain_block_per_card(ctx: EffectContext) -> None:
    """Gain block for each card in hand (Spirit Shield)."""
    per_card = ctx.magic_number if ctx.magic_number > 0 else 3
    block = per_card * len(ctx.hand)
    ctx.gain_block(block)


# =============================================================================
# Pressure Points (Mark)
# =============================================================================

@effect_simple("apply_mark")
def apply_mark(ctx: EffectContext) -> None:
    """Apply Mark to target (Pressure Points)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 8
    ctx.apply_status_to_target("Mark", amount)


@effect_simple("trigger_all_marks")
def trigger_all_marks(ctx: EffectContext) -> None:
    """Deal damage to all enemies equal to their Mark."""
    for enemy in ctx.living_enemies:
        mark = enemy.statuses.get("Mark", 0)
        if mark > 0:
            ctx.deal_damage_to_enemy(enemy, mark)


# =============================================================================
# Talk to the Hand (Block Return)
# =============================================================================

@effect_simple("apply_block_return")
def apply_block_return(ctx: EffectContext) -> None:
    """Apply Block Return to target (Talk to the Hand)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 2
    ctx.apply_status_to_target("BlockReturn", amount)


# =============================================================================
# Turn Control
# =============================================================================

@effect_simple("end_turn")
def end_turn_effect(ctx: EffectContext) -> None:
    """End the player's turn (Conclude, Meditate)."""
    ctx.end_turn()


@effect_simple("take_extra_turn")
def take_extra_turn(ctx: EffectContext) -> None:
    """Take an extra turn after this one (Vault)."""
    ctx.extra_data["extra_turn"] = True


# =============================================================================
# Blasphemy
# =============================================================================

@effect_simple("die_next_turn")
def die_next_turn(ctx: EffectContext) -> None:
    """Set Blasphemy - die at end of next turn."""
    ctx.apply_status_to_player("Blasphemy", 1)


# =============================================================================
# Special Scrawl
# =============================================================================

@effect_simple("draw_until_hand_full")
def draw_until_hand_full(ctx: EffectContext) -> None:
    """Draw until hand has 10 cards (Scrawl)."""
    cards_needed = 10 - len(ctx.hand)
    if cards_needed > 0:
        ctx.draw_cards(cards_needed)


# =============================================================================
# Meditate
# =============================================================================

@effect_simple("put_cards_from_discard_to_hand")
def put_cards_from_discard_to_hand(ctx: EffectContext) -> None:
    """Put cards from discard into hand (Meditate)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 1
    for _ in range(amount):
        if ctx.discard_pile and len(ctx.hand) < 10:
            # In simulation, move first card (player would choose)
            card = ctx.discard_pile[0]
            ctx.move_card_from_discard_to_hand(card)


# =============================================================================
# Power Effects (apply power status to self)
# =============================================================================

@effect_simple("on_stance_change_gain_block")
def mental_fortress_power(ctx: EffectContext) -> None:
    """Mental Fortress - Gain block on stance change."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 4
    ctx.apply_status_to_player("MentalFortress", amount)


@effect_simple("on_scry_gain_block")
def nirvana_power(ctx: EffectContext) -> None:
    """Nirvana - Gain block when scrying."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 3
    ctx.apply_status_to_player("Nirvana", amount)


@effect_simple("on_wrath_draw")
def rushdown_power(ctx: EffectContext) -> None:
    """Rushdown - Draw when entering Wrath."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 2
    ctx.apply_status_to_player("Rushdown", amount)


@effect_simple("if_calm_end_turn_gain_block")
def like_water_power(ctx: EffectContext) -> None:
    """Like Water - Gain block at end of turn if in Calm."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 5
    ctx.apply_status_to_player("LikeWater", amount)


@effect_simple("gain_mantra_each_turn")
def devotion_power(ctx: EffectContext) -> None:
    """Devotion - Gain mantra each turn."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 2
    ctx.apply_status_to_player("Devotion", amount)


@effect_simple("retained_cards_cost_less")
def establishment_power(ctx: EffectContext) -> None:
    """Establishment - Retained cards cost 1 less."""
    ctx.apply_status_to_player("Establishment", 1)


@effect_simple("scry_each_turn")
def foresight_power(ctx: EffectContext) -> None:
    """Foresight - Scry at start of each turn."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 3
    ctx.apply_status_to_player("Foresight", amount)


@effect_simple("add_smite_each_turn")
def battle_hymn_power(ctx: EffectContext) -> None:
    """Battle Hymn - Add Smite to hand each turn."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 1
    ctx.apply_status_to_player("BattleHymn", amount)


@effect_simple("add_insight_end_turn")
def study_power(ctx: EffectContext) -> None:
    """Study - Add Insight to draw pile at end of turn."""
    ctx.apply_status_to_player("Study", 1)


@effect_simple("gain_energy_each_turn_stacking")
def deva_form_power(ctx: EffectContext) -> None:
    """Deva Form - Gain energy each turn, increasing."""
    ctx.apply_status_to_player("DevaForm", 1)


@effect_simple("created_cards_upgraded")
def master_reality_power(ctx: EffectContext) -> None:
    """Master Reality - Created cards are upgraded."""
    ctx.apply_status_to_player("MasterReality", 1)


@effect_simple("next_attack_plus_damage")
def wreath_of_flame_effect(ctx: EffectContext) -> None:
    """Wreath of Flame - Next attack deals extra damage."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 5
    ctx.apply_status_to_player("WreathOfFlame", amount)


@effect_simple("wrath_next_turn_draw_next_turn")
def simmering_fury_effect(ctx: EffectContext) -> None:
    """Simmering Fury - Next turn enter Wrath and draw."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 2
    ctx.apply_status_to_player("SimmeringFury", amount)


@effect_simple("free_attack_next_turn")
def swivel_effect(ctx: EffectContext) -> None:
    """Swivel - Next attack costs 0."""
    ctx.apply_status_to_player("FreeAttackPower", 1)


@effect_simple("block_gain_applies_weak")
def wave_of_the_hand_effect(ctx: EffectContext) -> None:
    """Wave of the Hand - Block gain applies Weak."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 1
    ctx.apply_status_to_player("WaveOfTheHand", amount)


# =============================================================================
# Pray (Combined Effect)
# =============================================================================

@effect_simple("gain_mantra_add_insight")
def pray_effect(ctx: EffectContext) -> None:
    """Pray - Gain mantra and add Insight to draw pile."""
    mantra = ctx.magic_number if ctx.magic_number > 0 else 3
    ctx.gain_mantra(mantra)
    card_id = "Insight+" if ctx.is_upgraded else "Insight"
    ctx.add_card_to_draw_pile(card_id, "random")


# =============================================================================
# Heal Effects
# =============================================================================

@effect("heal")
def heal_effect(ctx: EffectContext, amount: int) -> None:
    """Heal the player."""
    ctx.heal_player(amount)


@effect_simple("heal_magic_number")
def heal_magic_number(ctx: EffectContext) -> None:
    """Heal by magic number (Bandage Up, Bite)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 4
    ctx.heal_player(amount)


# =============================================================================
# On-Trigger Effects (passive, tracked by combat system)
# =============================================================================

@effect_simple("on_stance_change_play_from_discard")
def flurry_of_blows_passive(ctx: EffectContext) -> None:
    """Flurry of Blows - Auto-play from discard on stance change."""
    # This is handled by the stance change system
    pass


@effect_simple("on_scry_play_from_discard")
def weave_passive(ctx: EffectContext) -> None:
    """Weave - Move to hand from discard on scry."""
    # This is handled by the scry system
    pass


# =============================================================================
# Unplayable/Curse Effects
# =============================================================================

@effect_simple("unplayable")
def unplayable(ctx: EffectContext) -> None:
    """This card cannot be played."""
    pass


# =============================================================================
# Lesson Learned
# =============================================================================

@effect_simple("if_fatal_upgrade_random_card")
def if_fatal_upgrade(ctx: EffectContext) -> None:
    """If this kills, upgrade a random card (Lesson Learned)."""
    ctx.extra_data["fatal_upgrade"] = True


# =============================================================================
# Collect (X-cost)
# =============================================================================

@effect_simple("put_x_miracles_on_draw")
def collect_effect(ctx: EffectContext) -> None:
    """Put X Miracles on top of draw pile (Collect)."""
    # X is the energy spent
    x = ctx.energy_spent if hasattr(ctx, 'energy_spent') else ctx.energy
    for _ in range(x):
        card_id = "Miracle+" if ctx.is_upgraded else "Miracle"
        ctx.add_card_to_draw_pile(card_id, "top")


# =============================================================================
# Conjure Blade (X-cost)
# =============================================================================

@effect_simple("add_expunger_to_hand")
def conjure_blade_effect(ctx: EffectContext) -> None:
    """Add Expunger to hand with X hits (Conjure Blade)."""
    x = ctx.extra_data.get("x_cost", 1)
    ctx.add_card_to_hand("Expunger")
    # Store the X value for Expunger's hits
    ctx.extra_data["expunger_hits"] = x


@effect_simple("hits_x_times")
def hits_x_times(ctx: EffectContext) -> None:
    """Hit X times (Expunger)."""
    # X is stored from Conjure Blade
    hits = ctx.extra_data.get("expunger_hits", 1)
    if ctx.card:
        damage = ctx.card.damage
        for _ in range(hits):
            ctx.deal_damage_to_target(damage)


# =============================================================================
# Omega (Power)
# =============================================================================

@effect_simple("deal_50_damage_end_turn")
def omega_power(ctx: EffectContext) -> None:
    """Omega - Deal 50 damage to all enemies at end of turn."""
    ctx.apply_status_to_player("Omega", 50)


# =============================================================================
# Artifact
# =============================================================================

@effect_simple("gain_artifact")
def gain_artifact(ctx: EffectContext) -> None:
    """Gain Artifact (Panacea)."""
    amount = 2 if ctx.is_upgraded else 1
    ctx.apply_status_to_player("Artifact", amount)


# =============================================================================
# Temporary Strength Down
# =============================================================================

@effect_simple("apply_temp_strength_down")
def apply_temp_strength_down(ctx: EffectContext) -> None:
    """Apply temporary Strength down (Dark Shackles)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 9
    ctx.apply_status_to_target("TempStrengthDown", amount)
    # This is removed at end of turn


# =============================================================================
# Wish
# =============================================================================

@effect_simple("choose_plated_armor_or_strength_or_gold")
def wish_effect(ctx: EffectContext) -> None:
    """
    Choose: Plated Armor, Strength, or Gold (Wish).

    In simulation, we'll default to Strength for combat value.
    """
    # Default to Strength in simulation
    amount = 4 if ctx.is_upgraded else 3
    ctx.apply_status_to_player("Strength", amount)


# =============================================================================
# Draw Pile Manipulation
# =============================================================================

@effect_simple("shuffle_discard_into_draw")
def shuffle_discard_into_draw(ctx: EffectContext) -> None:
    """Shuffle discard pile into draw pile (Deep Breath)."""
    ctx._shuffle_discard_into_draw()


# =============================================================================
# Register all effects on module load
# =============================================================================

def _ensure_effects_registered():
    """Ensure all effects are registered. Called on module import."""
    # All effects are registered via decorators when this module is imported
    pass


# Auto-register on import
_ensure_effects_registered()
