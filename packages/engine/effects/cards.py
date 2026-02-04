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

COMPLETE WATCHER CARD LIST:
===========================

ATTACKS:
--------
- Strike_P: Basic attack (6/8 damage)
- Eruption: 9 damage, enter Wrath (2 cost, upgraded 1 cost)
- BowlingBash: 7/10 damage x number of enemies
- Conclude: 12/16 damage ALL, end turn
- Consecrate: 5/8 damage ALL
- CutThroughFate: 7/9 damage, Scry 2/3, draw 1
- EmptyFist: 9/14 damage, exit stance
- FlurryOfBlows: 4/6 damage, auto-plays from discard on stance change
- FlyingSleeves: 4/6 damage x2, Retain
- FollowUp: 7/11 damage, +1 energy if last card was Attack
- JustLucky: 3/4 damage, 2/3 block, Scry 1/2
- ReachHeaven: 10/15 damage, add Through Violence to draw
- SashWhip: 8/10 damage, Weak 1/2 if last card was Attack
- SignatureMove: 30/40 damage, only playable if only Attack in hand
- Tantrum: 3 damage x 3/4, enter Wrath, shuffle into draw
- TalkToTheHand: 5/7 damage, apply Block Return 2/3, Exhaust
- Wallop: 9/14 damage, gain Block = unblocked damage dealt
- Weave: 4/6 damage, auto-plays from discard on Scry
- WheelKick: 15/20 damage, draw 2
- WindmillStrike: 7/10 damage, Retain, +4 damage each turn retained
- FearNoEvil: 8/11 damage, enter Calm if enemy attacking
- SandsOfTime: 20/26 damage, Retain, cost -1 each turn retained
- CarveReality: 6/10 damage, add Smite to hand
- Brilliance: 12/16 damage + total Mantra gained this combat
- Ragnarok: 5/6 damage x5/6 to random enemies
- LessonLearned: 10/13 damage, Exhaust, upgrade random card if fatal
- Judgement: Kill enemy if HP <= 30/40

SKILLS:
-------
- Defend_P: 5/8 Block
- Vigilance: 8/12 Block, enter Calm
- Halt: 3/4 Block, +6/9 Block in Wrath
- Tranquility: Enter Calm, Retain, Exhaust
- Crescendo: Enter Wrath, Retain, Exhaust (upgraded: not Exhaust)
- EmptyBody: 7/10 Block, exit stance
- EmptyMind: Draw 2/3, exit stance
- Evaluate: 6/10 Block, put Insight on draw
- InnerPeace: Draw 3/4 if Calm, else enter Calm
- Protect: 12/16 Block, Retain
- ThirdEye: 7/9 Block, Scry 3/5
- Prostrate: 4 Block, gain 2/3 Mantra
- Collect: X cost, put X Miracles on draw, Exhaust
- DeceiveReality: 4/7 Block, add Safety to hand
- Indignation: Enter Wrath, or gain 3/5 Mantra if in Wrath
- Meditate: Put 1/2 cards from discard to hand with Retain, enter Calm, end turn
- Perseverance: 5/7 Block, Retain, +2/+3 Block when retained
- Pray: Gain 3/4 Mantra, shuffle Insight into draw
- Sanctity: 6/9 Block, draw 2/3 if last card was Skill
- Swivel: 8/11 Block, next Attack costs 0
- WaveOfTheHand: Apply Weak 1/2 whenever you gain Block this turn
- SimmeringFury: Next turn enter Wrath and draw 2/3
- Worship: Gain 5/8 Mantra, Retain
- WreathOfFlame: Next Attack deals +5/+8 damage
- Alpha: Shuffle Beta into draw, Exhaust, Innate
- Blasphemy: Enter Divinity, die at start of next turn, Retain
- ConjureBlade: X cost, add Expunger (X hits) to hand
- Omniscience: Choose card from draw, play it twice, Exhaust
- Scrawl: Draw until 10 cards, Exhaust
- SpiritShield: Gain 3/4 Block per card in hand
- Vault: Take extra turn, Exhaust
- Wish: Choose: 3/4 Plated Armor, or 3/4 Strength, or 50/75 Gold

POWERS:
-------
- BattleHymn: Add 1 Smite to hand each turn (upgraded: Innate)
- Establishment: Retained cards cost 1 less (upgraded: Innate)
- LikeWater: If in Calm at end of turn, gain 5/7 Block
- MentalFortress: Gain 4/6 Block when changing stance
- Nirvana: Gain 3/4 Block when Scrying
- Rushdown: Draw 2 when entering Wrath (upgraded: cost 0)
- Study: Shuffle 1 Insight into draw at end of turn
- Foresight: Scry 3/4 at start of turn
- DevaForm: Gain 1 Energy at start of each turn (stacks)
- Devotion: Gain 3/4 Mantra at start of turn
- Fasting: Gain 3/4 Strength and Dexterity, lose 1 Focus
- MasterReality: Created cards are Upgraded

SPECIAL/GENERATED CARDS:
------------------------
- Miracle: Retain, Exhaust, gain 1/2 Energy
- Insight: Retain, Exhaust, draw 2/3
- Smite: Retain, Exhaust, 12/16 damage
- Safety: Retain, Exhaust, 12/16 Block
- ThroughViolence: Retain, Exhaust, 20/30 damage
- Expunger: Deal 9 damage X times
- Beta: Shuffle Omega into draw, Exhaust
- Omega: At end of turn deal 50 damage to ALL enemies
"""

from __future__ import annotations

from typing import TYPE_CHECKING, List, Optional
import random

from .registry import (
    effect, effect_simple, effect_custom, EffectContext
)

if TYPE_CHECKING:
    from ..state.combat import EnemyCombatState


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


@effect_simple("gain_1_energy")
def gain_1_energy(ctx: EffectContext) -> None:
    """Miracle: Gain 1 energy (2 if upgraded). Retain. Exhaust."""
    energy_gain = 2 if ctx.is_upgraded else 1
    ctx.gain_energy(energy_gain)


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
        amount = 2
        ctx.draw_cards(amount)


@effect_simple("if_in_wrath_extra_block_6")
def if_in_wrath_extra_block(ctx: EffectContext) -> None:
    """If in Wrath, gain extra block (Halt)."""
    if ctx.stance == "Wrath":
        extra = 10 if ctx.is_upgraded else 6
        ctx.gain_block(extra)


@effect_simple("if_calm_draw_else_calm")
def if_calm_draw_else_calm(ctx: EffectContext) -> None:
    """If in Calm draw 3/4, else enter Calm (Inner Peace)."""
    if ctx.stance == "Calm":
        amount = 4 if ctx.is_upgraded else 3
        ctx.draw_cards(amount)
    else:
        ctx.change_stance("Calm")


# Alias for backwards compatibility
@effect_simple("if_calm_draw_3_else_calm")
def _if_calm_draw_else_calm_alias(ctx: EffectContext) -> None:
    """Alias for if_calm_draw_else_calm."""
    if_calm_draw_else_calm(ctx)


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


@effect_simple("cost_0_in_wrath")
def cost_0_in_wrath(ctx: EffectContext) -> None:
    """Scrawl costs 0 in Wrath stance. This is a marker effect; cost logic is handled externally."""
    pass


def get_card_cost_modifier(card_id: str, stance: str) -> Optional[int]:
    """
    Get cost override for a card based on current stance or state.

    Args:
        card_id: The card ID
        stance: Current stance name

    Returns:
        Override cost if applicable, None otherwise
    """
    base_id = card_id.rstrip("+")
    if base_id == "Scrawl" and stance == "Wrath":
        return 0
    return None


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
    Choose: Plated Armor (0), Strength (1), or Gold (2) (Wish).

    Choice is determined by extra_data["wish_choice"]:
        0 = Plated Armor (3/4)
        1 = Strength (3/4) [default]
        2 = Gold (50/75)
    """
    choice = ctx.extra_data.get("wish_choice", 1)
    amount = 4 if ctx.is_upgraded else 3

    if choice == 0:
        ctx.apply_status_to_player("PlatedArmor", amount)
    elif choice == 2:
        gold = 75 if ctx.is_upgraded else 50
        ctx.extra_data["gold_gained"] = gold
    else:
        # Default: Strength (best combat value)
        ctx.apply_status_to_player("Strength", amount)


# =============================================================================
# Draw Pile Manipulation
# =============================================================================

@effect_simple("shuffle_discard_into_draw")
def shuffle_discard_into_draw(ctx: EffectContext) -> None:
    """Shuffle discard pile into draw pile (Deep Breath)."""
    ctx._shuffle_discard_into_draw()


# =============================================================================
# COMPLETE WATCHER ATTACK CARD EFFECTS
# =============================================================================

@effect_simple("damage_twice")
def damage_twice(ctx: EffectContext) -> None:
    """
    Deal damage twice (Flying Sleeves).

    The base damage is calculated by EffectExecutor, this marks
    the card as hitting twice.
    """
    # Damage execution is handled by EffectExecutor with magic_number
    # This effect just signals multi-hit behavior
    pass


@effect_simple("if_last_card_attack_weak")
def if_last_card_attack_weak(ctx: EffectContext) -> None:
    """Apply Weak if last card played was an Attack (Sash Whip)."""
    if ctx.get_last_card_type() == "ATTACK":
        amount = 2 if ctx.is_upgraded else 1
        ctx.apply_status_to_target("Weak", amount)


@effect_simple("if_last_card_skill_vulnerable")
def if_last_card_skill_vulnerable(ctx: EffectContext) -> None:
    """Apply Vulnerable if last card played was a Skill (Crush Joints)."""
    if ctx.get_last_card_type() == "SKILL":
        amount = 2 if ctx.is_upgraded else 1
        ctx.apply_status_to_target("Vulnerable", amount)


@effect_simple("shuffle_self_into_draw")
def shuffle_self_into_draw(ctx: EffectContext) -> None:
    """After playing, shuffle this card into draw pile instead of discard."""
    combat_state = ctx.state if hasattr(ctx, 'state') else ctx
    combat_state._shuffle_played_card = True


@effect_simple("only_attack_in_hand")
def only_attack_in_hand(ctx: EffectContext) -> None:
    """
    Signature Move - can only be played if only Attack in hand.

    This is a playability check, handled elsewhere. Effect is passive.
    """
    pass


@effect_simple("gain_damage_when_retained_4")
def gain_damage_when_retained(ctx: EffectContext) -> None:
    """
    Windmill Strike - gains +4 damage each turn retained.

    This is tracked by the combat system when cards are retained.
    The damage bonus is stored in extra_data.
    """
    bonus = ctx.extra_data.get("windmill_bonus", 0)
    if bonus > 0 and ctx.target:
        ctx.deal_damage_to_target(bonus)


@effect_simple("cost_reduces_each_turn")
def cost_reduces_each_turn(ctx: EffectContext) -> None:
    """
    Sands of Time - cost reduces by 1 each turn while retained.

    This is handled by the retain system.
    """
    pass


@effect_simple("gains_block_when_retained")
def gains_block_when_retained(ctx: EffectContext) -> None:
    """
    Perseverance - gains block when retained.

    The block bonus is tracked in extra_data.
    """
    bonus = ctx.extra_data.get("perseverance_bonus", 0)
    if bonus > 0:
        ctx.gain_block(bonus)


@effect_simple("if_fatal_gain_gold")
def if_fatal_gain_gold(ctx: EffectContext) -> None:
    """
    Hand of Greed - gain gold if this kills (not applicable in combat sim).

    In combat simulation, we track that the effect triggered.
    """
    ctx.extra_data["fatal_gold"] = True
    amount = 25 if ctx.is_upgraded else 20
    ctx.extra_data["gold_if_fatal"] = amount


# =============================================================================
# ADDITIONAL WATCHER SKILL EFFECTS
# =============================================================================

@effect_simple("on_draw_add_miracles_and_exhaust")
def deus_ex_machina_effect(ctx: EffectContext) -> None:
    """
    Deus Ex Machina - when drawn, add 2/3 Miracles and exhaust.

    This is a triggered effect handled by the draw system.
    """
    amount = 3 if ctx.is_upgraded else 2
    for _ in range(amount):
        card_id = "Miracle+" if ctx.state.player.statuses.get("MasterReality", 0) else "Miracle"
        ctx.add_card_to_hand(card_id)
    # Exhaust is handled by card flags


@effect_simple("choose_attack_from_any_class")
def foreign_influence_effect(ctx: EffectContext) -> None:
    """
    Foreign Influence - choose attack from any class to add to hand.

    In simulation, we add a random strong attack.
    """
    # In simulation, add a generic strong card
    attacks = ["Smite", "PommelStrike", "Cleave"]
    card_id = random.choice(attacks)
    ctx.add_card_to_hand(card_id)
    if ctx.is_upgraded:
        ctx.add_card_to_hand(random.choice(attacks))


@effect_simple("play_card_from_draw_twice")
def omniscience_effect(ctx: EffectContext) -> None:
    """
    Omniscience - choose card from draw pile, play it twice.

    In simulation, plays top non-Attack card from draw twice.
    """
    ctx.extra_data["omniscience_active"] = True
    # The actual card selection and double-play is handled by the executor


@effect_simple("retain_card")
def retain_card_effect(ctx: EffectContext) -> None:
    """Mark selected cards as retained (Meditate)."""
    ctx.extra_data["grant_retain"] = True


# =============================================================================
# WATCHER POWER EFFECTS - COMPLETE
# =============================================================================

@effect_simple("gain_strength_and_dex_lose_focus")
def fasting_effect(ctx: EffectContext) -> None:
    """
    Fasting - gain Strength and Dexterity (Watcher has no Focus).

    Base: +3 Str, +3 Dex
    Upgraded: +4 Str, +4 Dex
    """
    amount = 4 if ctx.is_upgraded else 3
    ctx.apply_status_to_player("Strength", amount)
    ctx.apply_status_to_player("Dexterity", amount)
    # Watcher doesn't have Focus, so no loss


# =============================================================================
# HAND OF GREED (Missing Attack)
# =============================================================================

@effect_simple("hand_of_greed")
def hand_of_greed_effect(ctx: EffectContext) -> None:
    """
    Hand of Greed - deal damage, gain gold if fatal.

    20/25 damage, gain 20/25 gold on kill.
    """
    # Damage is handled by base_damage
    # Gold tracking for kills
    ctx.extra_data["hand_of_greed_gold"] = 25 if ctx.is_upgraded else 20


# =============================================================================
# COMPLETE CARD EFFECT REGISTRY FOR ALL WATCHER CARDS
# =============================================================================

# This maps card IDs to their effects for lookup
WATCHER_CARD_EFFECTS = {
    # === BASIC ATTACKS ===
    "Strike_P": [],  # Just damage
    "Eruption": ["enter_wrath"],

    # === COMMON ATTACKS ===
    "BowlingBash": ["damage_per_enemy"],
    "CutThroughFate": ["scry", "draw_1"],
    "EmptyFist": ["exit_stance"],  # exit_stance is on Card
    "FlurryOfBlows": ["on_stance_change_play_from_discard"],
    "FlyingSleeves": ["damage_twice"],
    "FollowUp": ["if_last_card_attack_gain_energy"],
    "JustLucky": ["scry", "gain_block"],
    "SashWhip": ["if_last_card_attack_weak"],
    "Consecrate": [],  # Just AoE damage
    "CrushJoints": ["if_last_card_skill_vulnerable"],

    # === UNCOMMON ATTACKS ===
    "Tantrum": ["damage_x_times", "enter_wrath", "shuffle_self_into_draw"],
    "FearNoEvil": ["if_enemy_attacking_enter_calm"],
    "ReachHeaven": ["add_through_violence_to_draw"],
    "SandsOfTime": ["cost_reduces_each_turn"],
    "SignatureMove": ["only_attack_in_hand"],
    "TalkToTheHand": ["apply_block_return"],
    "Wallop": ["gain_block_equal_unblocked_damage"],
    "Weave": ["on_scry_play_from_discard"],
    "WheelKick": ["draw_2"],
    "WindmillStrike": ["gain_damage_when_retained_4"],
    "Conclude": ["end_turn"],
    "CarveReality": ["add_smite_to_hand"],

    # === RARE ATTACKS ===
    "Brilliance": ["damage_plus_mantra_gained"],
    "Judgement": ["if_enemy_hp_below_kill"],
    "LessonLearned": ["if_fatal_upgrade_random_card"],
    "Ragnarok": ["damage_random_x_times"],

    # === BASIC SKILLS ===
    "Defend_P": [],  # Just block
    "Vigilance": ["enter_calm"],

    # === COMMON SKILLS ===
    "ClearTheMind": ["enter_calm"],  # Tranquility
    "Crescendo": ["enter_wrath"],
    "EmptyBody": ["exit_stance"],
    "EmptyMind": ["draw_cards", "exit_stance"],
    "Evaluate": ["add_insight_to_draw"],
    "Halt": ["if_in_wrath_extra_block_6"],
    "InnerPeace": ["if_calm_draw_else_calm"],
    "PathToVictory": ["apply_mark", "trigger_all_marks"],  # Pressure Points
    "Protect": [],  # Just block + retain
    "ThirdEye": ["scry"],
    "Prostrate": ["gain_mantra"],

    # === UNCOMMON SKILLS ===
    "Collect": ["put_x_miracles_on_draw"],
    "DeceiveReality": ["add_safety_to_hand"],
    "Indignation": ["if_wrath_gain_mantra_else_wrath"],
    "Meditate": ["put_cards_from_discard_to_hand", "enter_calm", "end_turn"],
    "Perseverance": ["gains_block_when_retained"],
    "Pray": ["gain_mantra_add_insight"],
    "Sanctity": ["if_last_skill_draw_2"],
    "Swivel": ["free_attack_next_turn"],
    "Vengeance": ["wrath_next_turn_draw_next_turn"],  # Simmering Fury
    "WaveOfTheHand": ["block_gain_applies_weak"],
    "Worship": ["gain_mantra"],
    "WreathOfFlame": ["next_attack_plus_damage"],

    # === RARE SKILLS ===
    "Alpha": ["shuffle_beta_into_draw"],
    "Blasphemy": ["enter_divinity", "die_next_turn"],
    "ConjureBlade": ["add_expunger_to_hand"],
    "DeusExMachina": ["on_draw_add_miracles_and_exhaust"],
    "ForeignInfluence": ["choose_attack_from_any_class"],
    "Omniscience": ["play_card_from_draw_twice"],
    "Scrawl": ["draw_until_hand_full", "cost_0_in_wrath"],
    "SpiritShield": ["gain_block_per_card_in_hand"],
    "Vault": ["take_extra_turn"],
    "Wish": ["choose_plated_armor_or_strength_or_gold"],

    # === POWERS ===
    "BattleHymn": ["add_smite_each_turn"],
    "Establishment": ["retained_cards_cost_less"],
    "LikeWater": ["if_calm_end_turn_gain_block"],
    "MentalFortress": ["on_stance_change_gain_block"],
    "Nirvana": ["on_scry_gain_block"],
    "Adaptation": ["on_wrath_draw"],  # Rushdown
    "Study": ["add_insight_end_turn"],
    "Wireheading": ["scry_each_turn"],  # Foresight
    "DevaForm": ["gain_energy_each_turn_stacking"],
    "Devotion": ["gain_mantra_each_turn"],
    "Fasting2": ["gain_strength_and_dex_lose_focus"],  # Fasting
    "MasterReality": ["created_cards_upgraded"],

    # === SPECIAL CARDS ===
    "Miracle": ["gain_1_energy"],
    "Insight": ["draw_cards"],
    "Smite": [],  # Just damage
    "Safety": [],  # Just block
    "ThroughViolence": [],  # Just damage
    "Expunger": ["hits_x_times"],
    "Beta": ["shuffle_omega_into_draw"],
    "Omega": ["deal_50_damage_end_turn"],
}


def get_card_effects(card_id: str) -> List[str]:
    """
    Get the effect names for a card.

    Args:
        card_id: The card ID (e.g., "Strike_P", "Eruption")

    Returns:
        List of effect names for the card
    """
    # Strip upgrade suffix if present
    base_id = card_id.rstrip("+")
    return WATCHER_CARD_EFFECTS.get(base_id, [])


# =============================================================================
# EFFECT EXECUTION HELPERS
# =============================================================================

def execute_card_effects(ctx: EffectContext, effects: List[str]) -> None:
    """
    Execute a list of effects in order.

    Args:
        ctx: The effect context
        effects: List of effect names to execute
    """
    from .registry import execute_effect

    for effect_name in effects:
        execute_effect(effect_name, ctx)


# =============================================================================
# STANCE CHANGE TRIGGERS
# =============================================================================

def trigger_on_stance_change(ctx: EffectContext, old_stance: str, new_stance: str) -> None:
    """
    Trigger all on-stance-change effects.

    Args:
        ctx: Effect context
        old_stance: Previous stance
        new_stance: New stance
    """
    # Mental Fortress - gain block on stance change
    mental_fortress = ctx.get_player_status("MentalFortress")
    if mental_fortress > 0:
        ctx.gain_block(mental_fortress)

    # Rushdown - draw when entering Wrath
    if new_stance == "Wrath":
        rushdown = ctx.get_player_status("Rushdown")
        if rushdown > 0:
            ctx.draw_cards(rushdown)

    # Flurry of Blows - move from discard to hand
    flurries = [c for c in ctx.state.discard_pile if c.startswith("FlurryOfBlows")]
    for f in flurries:
        if len(ctx.state.hand) < 10:
            ctx.state.discard_pile.remove(f)
            ctx.state.hand.append(f)


def trigger_on_scry(ctx: EffectContext, cards_scried: List[str]) -> None:
    """
    Trigger all on-scry effects.

    Args:
        ctx: Effect context
        cards_scried: List of card IDs that were scried
    """
    # Nirvana - gain flat block once per scry action (not per card)
    nirvana = ctx.get_player_status("Nirvana")
    if nirvana > 0:
        ctx.gain_block(nirvana)

    # Weave - move from discard to hand
    weaves = [c for c in ctx.state.discard_pile if c.startswith("Weave")]
    for w in weaves:
        if len(ctx.state.hand) < 10:
            ctx.state.discard_pile.remove(w)
            ctx.state.hand.append(w)


# =============================================================================
# START/END OF TURN EFFECTS
# =============================================================================

def apply_start_of_turn_effects(ctx: EffectContext) -> dict:
    """
    Apply all start-of-turn triggered effects.

    Returns:
        Dict with effects that triggered and their results
    """
    result = {
        "effects": [],
        "energy_gained": 0,
        "cards_drawn": 0,
        "mantra_gained": 0,
        "block_gained": 0,
        "stance_changed": None,
    }

    # Foresight - Scry at start of turn
    foresight = ctx.get_player_status("Foresight")
    if foresight > 0:
        ctx.scry(foresight)
        result["effects"].append(f"foresight_scry_{foresight}")

    # Battle Hymn - Add Smite(s) to hand
    battle_hymn = ctx.get_player_status("BattleHymn")
    if battle_hymn > 0:
        for _ in range(battle_hymn):
            # Check Master Reality for upgrade
            upgraded = ctx.get_player_status("MasterReality") > 0
            card_id = "Smite+" if upgraded else "Smite"
            ctx.add_card_to_hand(card_id)
        result["effects"].append(f"battle_hymn_{battle_hymn}")

    # Deva Form - Gain energy (stacking)
    deva_form = ctx.get_player_status("DevaForm")
    if deva_form > 0:
        ctx.gain_energy(deva_form)
        result["energy_gained"] += deva_form
        # Increment for next turn
        ctx.apply_status_to_player("DevaForm", 1)
        result["effects"].append(f"deva_form_{deva_form}")

    # Devotion - Gain mantra
    devotion = ctx.get_player_status("Devotion")
    if devotion > 0:
        mantra_result = ctx.gain_mantra(devotion)
        result["mantra_gained"] += devotion
        result["effects"].append(f"devotion_{devotion}")
        if mantra_result.get("divinity_triggered"):
            result["stance_changed"] = "Divinity"
            # Trigger stance change effects (Mental Fortress, Rushdown, Flurry of Blows)
            old_stance = mantra_result.get("old_stance", "Neutral")
            trigger_on_stance_change(ctx, old_stance, "Divinity")

    # Simmering Fury - Enter Wrath and draw
    simmering = ctx.get_player_status("SimmeringFury")
    if simmering > 0:
        ctx.change_stance("Wrath")
        ctx.draw_cards(simmering)
        ctx.remove_status_from_player("SimmeringFury")
        result["stance_changed"] = "Wrath"
        result["cards_drawn"] += simmering
        result["effects"].append(f"simmering_fury_{simmering}")

    return result


def apply_end_of_turn_effects(ctx: EffectContext) -> dict:
    """
    Apply all end-of-turn triggered effects.

    Returns:
        Dict with effects that triggered and their results
    """
    result = {
        "effects": [],
        "damage_dealt": 0,
        "block_gained": 0,
        "stance_changed": None,
        "player_died": False,
    }

    # Like Water - Gain block if in Calm
    if ctx.stance == "Calm":
        like_water = ctx.get_player_status("LikeWater")
        if like_water > 0:
            ctx.gain_block(like_water)
            result["block_gained"] += like_water
            result["effects"].append(f"like_water_{like_water}")

    # Divinity auto-exit
    if ctx.stance == "Divinity":
        ctx.change_stance("Neutral")
        result["stance_changed"] = "Neutral"
        result["effects"].append("divinity_exit")

    # Study - Add Insight to draw pile
    study = ctx.get_player_status("Study")
    if study > 0:
        for _ in range(study):
            upgraded = ctx.get_player_status("MasterReality") > 0
            card_id = "Insight+" if upgraded else "Insight"
            ctx.add_card_to_draw_pile(card_id, "random")
        result["effects"].append(f"study_{study}")

    # Omega - Deal 50 damage to all enemies
    omega = ctx.get_player_status("Omega")
    if omega > 0:
        for enemy in ctx.living_enemies:
            dmg = ctx.deal_damage_to_enemy(enemy, omega)
            result["damage_dealt"] += dmg
        result["effects"].append(f"omega_{omega}")

    # Blasphemy - Die at end of turn (the turn AFTER playing it)
    blasphemy = ctx.get_player_status("Blasphemy")
    if blasphemy > 0:
        # Decrement counter, die when it reaches 0
        new_val = blasphemy - 1
        if new_val <= 0:
            ctx.state.player.hp = 0
            result["player_died"] = True
            result["effects"].append("blasphemy_death")
            ctx.remove_status_from_player("Blasphemy")
        else:
            ctx.state.player.statuses["Blasphemy"] = new_val

    # Handle retained cards bonuses
    _process_retained_cards(ctx, result)

    return result


def _process_retained_cards(ctx: EffectContext, result: dict) -> None:
    """Process end-of-turn effects for retained cards."""

    # Track cards that will be retained
    retained_cards = []

    for card_id in ctx.state.hand:
        base_id = card_id.rstrip("+")

        # Check if card has retain
        # In actual implementation, this would check card data
        retain_cards = {
            "Miracle", "FlyingSleeves", "Protect", "Worship",
            "WindmillStrike", "SandsOfTime", "Perseverance",
            "Tranquility", "Crescendo", "Blasphemy",
            "Insight", "Smite", "Safety", "ThroughViolence"
        }

        if base_id in retain_cards or card_id in retain_cards:
            retained_cards.append(card_id)

    # Apply Establishment cost reduction
    establishment = ctx.get_player_status("Establishment")
    if establishment > 0 and retained_cards:
        # Reduce cost of retained cards
        for card_id in retained_cards:
            current_cost = ctx.state.card_costs.get(card_id, 1)
            new_cost = max(0, current_cost - establishment)
            ctx.state.card_costs[card_id] = new_cost
        result["effects"].append(f"establishment_{len(retained_cards)}_cards")

    # Windmill Strike damage bonus
    for card_id in retained_cards:
        if card_id.startswith("WindmillStrike"):
            bonus = ctx.extra_data.get(f"windmill_bonus_{card_id}", 0)
            ctx.extra_data[f"windmill_bonus_{card_id}"] = bonus + 4

    # Sands of Time cost reduction
    for card_id in retained_cards:
        if card_id.startswith("SandsOfTime"):
            current = ctx.state.card_costs.get(card_id, 4)
            ctx.state.card_costs[card_id] = max(0, current - 1)

    # Perseverance block bonus
    for card_id in retained_cards:
        if card_id.startswith("Perseverance"):
            bonus = ctx.extra_data.get(f"perseverance_bonus_{card_id}", 0)
            upgrade_bonus = 3 if card_id.endswith("+") else 2
            ctx.extra_data[f"perseverance_bonus_{card_id}"] = bonus + upgrade_bonus


# =============================================================================
# MANTRA SYSTEM
# =============================================================================

def gain_mantra_and_check_divinity(ctx: EffectContext, amount: int) -> dict:
    """
    Gain mantra and check for Divinity trigger.

    Args:
        ctx: Effect context
        amount: Amount of mantra to gain

    Returns:
        Dict with mantra gained and whether Divinity triggered
    """
    result = {
        "mantra_gained": amount,
        "total_mantra": 0,
        "divinity_triggered": False,
        "energy_gained": 0,
    }

    # Track total mantra gained this combat
    total_combat_mantra = ctx.extra_data.get("total_mantra_gained", 0)
    ctx.extra_data["total_mantra_gained"] = total_combat_mantra + amount

    # Add to current mantra
    current = ctx.get_player_status("Mantra")
    new_total = current + amount

    if new_total >= 10:
        # Enter Divinity
        remainder = new_total - 10
        ctx.state.player.statuses["Mantra"] = remainder
        result["total_mantra"] = remainder
        result["divinity_triggered"] = True

        # Trigger stance change
        stance_result = ctx.change_stance("Divinity")
        result["energy_gained"] = stance_result.get("energy_gained", 0)
    else:
        ctx.state.player.statuses["Mantra"] = new_total
        result["total_mantra"] = new_total

    return result


# =============================================================================
# SCRY SYSTEM
# =============================================================================

def perform_scry(ctx: EffectContext, amount: int, discard_indices: List[int] = None) -> List[str]:
    """
    Perform scry action - look at top cards, choose which to discard.

    Args:
        ctx: Effect context
        amount: Number of cards to scry
        discard_indices: Indices of scried cards to discard (0-indexed).
                        If None, AI chooses (default: keep all).

    Returns:
        List of card IDs that were scried
    """
    scried = []

    # Look at top N cards
    for _ in range(amount):
        if not ctx.state.draw_pile:
            break
        card = ctx.state.draw_pile.pop()
        scried.append(card)

    # Discard selected cards
    if discard_indices:
        for i in sorted(discard_indices, reverse=True):
            if 0 <= i < len(scried):
                card = scried.pop(i)
                ctx.state.discard_pile.append(card)

    # Put remaining cards back on top (in reverse order)
    for card in reversed(scried):
        ctx.state.draw_pile.append(card)

    # Trigger on-scry effects
    all_scried = scried.copy()
    trigger_on_scry(ctx, all_scried)

    return all_scried


# =============================================================================
# CARD VALIDATION
# =============================================================================

def can_play_card(ctx: EffectContext, card_id: str) -> tuple:
    """
    Check if a card can be played.

    Args:
        ctx: Effect context
        card_id: Card to check

    Returns:
        Tuple of (can_play: bool, reason: str)
    """
    base_id = card_id.rstrip("+")

    # Signature Move - only if only attack in hand
    if base_id == "SignatureMove":
        attack_count = sum(
            1 for c in ctx.state.hand
            if _is_attack_card(c)
        )
        if attack_count > 1:
            return (False, "Can only play when only Attack in hand")

    # Clash - only if only attacks in hand (Ironclad but similar pattern)
    if base_id == "Clash":
        non_attacks = sum(
            1 for c in ctx.state.hand
            if not _is_attack_card(c)
        )
        if non_attacks > 0:
            return (False, "Can only play when only Attacks in hand")

    return (True, "")


def _is_attack_card(card_id: str) -> bool:
    """Check if a card is an Attack type."""
    # List of known attack card IDs
    attack_ids = {
        # Watcher
        "Strike_P", "Eruption", "BowlingBash", "CutThroughFate",
        "EmptyFist", "FlurryOfBlows", "FlyingSleeves", "FollowUp",
        "JustLucky", "SashWhip", "Consecrate", "CrushJoints",
        "Tantrum", "FearNoEvil", "ReachHeaven", "SandsOfTime",
        "SignatureMove", "TalkToTheHand", "Wallop", "Weave",
        "WheelKick", "WindmillStrike", "Conclude", "CarveReality",
        "Brilliance", "Judgement", "LessonLearned", "Ragnarok",
        "Smite", "ThroughViolence", "Expunger",
        # Ironclad
        "Strike_R", "Bash", "Anger", "Body Slam", "Clash", "Cleave",
        "Clothesline", "Headbutt", "Heavy Blade", "Iron Wave",
        "Perfected Strike", "Pommel Strike", "Sword Boomerang",
        "Thunderclap", "Twin Strike", "Wild Strike",
        "Blood for Blood", "Carnage", "Dropkick", "Hemokinesis",
        "Pummel", "Rampage", "Reckless Charge", "Searing Blow",
        "Sever Soul", "Uppercut", "Whirlwind",
        "Bludgeon", "Feed", "Fiend Fire", "Immolate", "Reaper",
        "Strike_R", "Bash", "Anger", "Cleave", "Clothesline",
        # Silent
        "Strike_G", "Neutralize", "Bane", "Dagger Spray", "Dagger Throw",
        "Flying Knee", "Poisoned Stab", "Quick Slash", "Slice",
        "Underhanded Strike", "Sucker Punch", "All Out Attack", "Backstab",
        "Choke", "Dash", "Endless Agony", "Eviscerate", "Finisher",
        "Flechettes", "Heel Hook", "Masterful Stab", "Predator",
        "Riddle With Holes", "Skewer", "Die Die Die", "Glass Knife",
        "Grand Finale", "Unload", "Shiv",
    }
    base_id = card_id.rstrip("+")
    return base_id in attack_ids


# =============================================================================
# Register all effects on module load
# =============================================================================

def _ensure_effects_registered():
    """Ensure all effects are registered. Called on module import."""
    # All effects are registered via decorators when this module is imported
    pass


# Auto-register on import
_ensure_effects_registered()


# =============================================================================
# IRONCLAD CARD EFFECTS
# =============================================================================

# -----------------------------------------------------------------------------
# Simple Stat Modifications
# -----------------------------------------------------------------------------

@effect_simple("gain_strength")
def gain_strength_effect(ctx: EffectContext) -> None:
    """Inflame - Gain Strength permanently."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 2
    ctx.apply_status_to_player("Strength", amount)


@effect_simple("gain_temp_strength")
def gain_temp_strength(ctx: EffectContext) -> None:
    """Flex - Gain temporary Strength (lost at end of turn)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 2
    ctx.apply_status_to_player("Strength", amount)
    ctx.apply_status_to_player("LoseStrength", amount)


@effect_simple("reduce_enemy_strength")
def reduce_enemy_strength(ctx: EffectContext) -> None:
    """Disarm - Reduce enemy Strength permanently."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 2
    if ctx.target:
        current = ctx.target.statuses.get("Strength", 0)
        ctx.target.statuses["Strength"] = current - amount


@effect_simple("double_strength")
def double_strength(ctx: EffectContext) -> None:
    """Limit Break - Double current Strength (Java: doubles any non-zero strength including negative)."""
    current = ctx.state.player.statuses.get("Strength", 0)
    if current != 0:
        # Java: hasPower("Strength") check + apply strAmt more
        # This effectively doubles the strength (positive or negative)
        ctx.apply_status_to_player("Strength", current)


@effect_simple("double_block")
def double_block(ctx: EffectContext) -> None:
    """Entrench - Double current Block."""
    current = ctx.state.player.block
    if current > 0:
        ctx.gain_block(current)


# -----------------------------------------------------------------------------
# Energy Effects
# -----------------------------------------------------------------------------

@effect_simple("gain_2_energy")
def gain_2_energy(ctx: EffectContext) -> None:
    """Seeing Red - Gain 2 energy."""
    ctx.gain_energy(2)


@effect_simple("lose_hp_gain_energy")
def lose_hp_gain_energy(ctx: EffectContext) -> None:
    """Bloodletting - Lose 3 HP, gain 2/3 energy."""
    ctx.state.player.hp -= 3
    if ctx.state.player.hp < 0:
        ctx.state.player.hp = 0
    energy_gain = ctx.magic_number if ctx.magic_number > 0 else 2
    ctx.gain_energy(energy_gain)


@effect_simple("lose_hp_gain_energy_draw")
def lose_hp_gain_energy_draw(ctx: EffectContext) -> None:
    """Offering - Lose 6 HP, gain 2 energy, draw 3/5 cards."""
    ctx.state.player.hp -= 6
    if ctx.state.player.hp < 0:
        ctx.state.player.hp = 0
    ctx.gain_energy(2)
    draw_amount = ctx.magic_number if ctx.magic_number > 0 else 3
    ctx.draw_cards(draw_amount)


# -----------------------------------------------------------------------------
# HP Loss / Self-Damage Effects
# -----------------------------------------------------------------------------

@effect_simple("lose_hp")
def lose_hp_effect(ctx: EffectContext) -> None:
    """Hemokinesis - Lose HP (2 HP)."""
    hp_loss = ctx.magic_number if ctx.magic_number > 0 else 2
    ctx.state.player.hp -= hp_loss
    if ctx.state.player.hp < 0:
        ctx.state.player.hp = 0


# -----------------------------------------------------------------------------
# Card Generation / Manipulation
# -----------------------------------------------------------------------------

@effect_simple("add_copy_to_discard")
def add_copy_to_discard(ctx: EffectContext) -> None:
    """Anger - Add a copy of this card to discard pile."""
    if ctx.card:
        card_id = ctx.card.id
        if ctx.is_upgraded:
            card_id = card_id + "+"
        ctx.add_card_to_discard(card_id)


@effect_simple("shuffle_wound_into_draw")
def shuffle_wound_into_draw(ctx: EffectContext) -> None:
    """Wild Strike - Shuffle a Wound into draw pile."""
    ctx.add_card_to_draw_pile("Wound", "random")


@effect_simple("shuffle_dazed_into_draw")
def shuffle_dazed_into_draw(ctx: EffectContext) -> None:
    """Reckless Charge - Shuffle a Dazed into draw pile."""
    ctx.add_card_to_draw_pile("Dazed", "random")


@effect_simple("add_wounds_to_hand")
def add_wounds_to_hand(ctx: EffectContext) -> None:
    """Power Through - Add 2 Wounds to hand."""
    for _ in range(2):
        ctx.add_card_to_hand("Wound")


@effect_simple("add_burn_to_discard")
def add_burn_to_discard(ctx: EffectContext) -> None:
    """Immolate - Add a Burn to discard pile."""
    ctx.add_card_to_discard("Burn")


@effect_simple("add_random_attack_cost_0")
def add_random_attack_cost_0(ctx: EffectContext) -> None:
    """Infernal Blade - Add a random Attack that costs 0 this turn."""
    from ..content.cards import ALL_CARDS, CardType, CardColor
    attacks = [
        cid for cid, c in ALL_CARDS.items()
        if c.card_type == CardType.ATTACK and c.color == CardColor.RED
        and c.rarity.value not in ["BASIC", "SPECIAL", "CURSE"]
    ]
    if attacks:
        card_id = random.choice(attacks)
        ctx.add_card_to_hand(card_id)
        # Mark card as cost 0 this turn
        if not hasattr(ctx.state, "card_costs"):
            ctx.state.card_costs = {}
        ctx.state.card_costs[card_id] = 0


@effect_simple("put_card_from_discard_on_draw")
def put_card_from_discard_on_draw(ctx: EffectContext) -> None:
    """Headbutt - Put a card from discard on top of draw pile (requires selection)."""
    # In simulation, move first card from discard to top of draw
    if ctx.state.discard_pile:
        card = ctx.state.discard_pile[0]
        ctx.state.discard_pile.remove(card)
        ctx.state.draw_pile.append(card)


@effect_simple("return_exhausted_card_to_hand")
def return_exhausted_card_to_hand(ctx: EffectContext) -> None:
    """Exhume - Return a card from exhaust pile to hand (requires selection)."""
    if ctx.state.exhaust_pile and len(ctx.state.hand) < 10:
        # In simulation, return first non-Exhume card
        for card in ctx.state.exhaust_pile:
            if not card.startswith("Exhume"):
                ctx.state.exhaust_pile.remove(card)
                ctx.state.hand.append(card)
                break


@effect_simple("copy_attack_or_power")
def copy_attack_or_power(ctx: EffectContext) -> None:
    """Dual Wield - Create 1/2 copies of an Attack or Power in hand (requires selection)."""
    copies = ctx.magic_number if ctx.magic_number > 0 else 1
    # In simulation, copy first Attack or Power
    from ..content.cards import ALL_CARDS, CardType
    for card_id in ctx.state.hand:
        base_id = card_id.rstrip("+")
        card_def = ALL_CARDS.get(base_id)
        if card_def and card_def.card_type in [CardType.ATTACK, CardType.POWER]:
            for _ in range(copies):
                if len(ctx.state.hand) < 10:
                    ctx.add_card_to_hand(card_id)
            break


# -----------------------------------------------------------------------------
# Draw Effects
# -----------------------------------------------------------------------------

@effect_simple("draw_then_no_draw")
def draw_then_no_draw(ctx: EffectContext) -> None:
    """Battle Trance - Draw 3/4 cards, cannot draw more this turn."""
    draw_amount = ctx.magic_number if ctx.magic_number > 0 else 3
    ctx.draw_cards(draw_amount)
    ctx.apply_status_to_player("NoDraw", 1)


@effect_simple("exhaust_to_draw")
def exhaust_to_draw(ctx: EffectContext) -> None:
    """Burning Pact - Exhaust 1 card, draw 2/3 (requires card selection)."""
    # In simulation, exhaust first non-essential card and draw
    if ctx.state.hand:
        # Find first non-power card to exhaust
        for i, card_id in enumerate(ctx.state.hand):
            ctx.exhaust_hand_idx(i)
            break
    draw_amount = ctx.magic_number if ctx.magic_number > 0 else 2
    ctx.draw_cards(draw_amount)


@effect_simple("draw_then_put_on_draw")
def draw_then_put_on_draw(ctx: EffectContext) -> None:
    """Warcry - Draw 1/2, put card from hand on top of draw (requires selection)."""
    draw_amount = ctx.magic_number if ctx.magic_number > 0 else 1
    ctx.draw_cards(draw_amount)
    # In simulation, put last card on draw
    if ctx.state.hand:
        card = ctx.state.hand.pop()
        ctx.state.draw_pile.append(card)


# -----------------------------------------------------------------------------
# Exhaust-Related Effects
# -----------------------------------------------------------------------------

@effect_simple("exhaust_random_card")
def exhaust_random_card(ctx: EffectContext) -> None:
    """True Grit (base) - Exhaust a random card from hand."""
    if ctx.state.hand:
        card = random.choice(ctx.state.hand)
        ctx.exhaust_card(card)


@effect_simple("exhaust_non_attacks_gain_block")
def exhaust_non_attacks_gain_block(ctx: EffectContext) -> None:
    """Second Wind - Exhaust all non-Attack cards, gain block per card."""
    from ..content.cards import ALL_CARDS, CardType
    block_per = ctx.magic_number if ctx.magic_number > 0 else 5
    cards_to_exhaust = []

    for card_id in ctx.state.hand[:]:  # Copy to avoid modifying during iteration
        base_id = card_id.rstrip("+")
        card_def = ALL_CARDS.get(base_id)
        if card_def and card_def.card_type != CardType.ATTACK:
            cards_to_exhaust.append(card_id)

    for card_id in cards_to_exhaust:
        ctx.exhaust_card(card_id)
        ctx.gain_block(block_per)


@effect_simple("exhaust_all_non_attacks")
def exhaust_all_non_attacks(ctx: EffectContext) -> None:
    """Sever Soul - Exhaust all non-Attack cards in hand."""
    from ..content.cards import ALL_CARDS, CardType
    cards_to_exhaust = []

    for card_id in ctx.state.hand[:]:
        base_id = card_id.rstrip("+")
        card_def = ALL_CARDS.get(base_id)
        if card_def and card_def.card_type != CardType.ATTACK:
            cards_to_exhaust.append(card_id)

    for card_id in cards_to_exhaust:
        ctx.exhaust_card(card_id)


@effect_simple("exhaust_hand_damage_per_card")
def exhaust_hand_damage_per_card(ctx: EffectContext) -> None:
    """Fiend Fire - Exhaust all cards in hand, deal damage per card."""
    if ctx.card and ctx.target:
        damage = ctx.card.damage
        count = 0

        # Count cards to exhaust (all except this card)
        for card_id in ctx.state.hand[:]:
            if card_id != ctx.card.id:
                count += 1

        # Exhaust all other cards
        cards_to_exhaust = [c for c in ctx.state.hand if c != ctx.card.id]
        for card_id in cards_to_exhaust:
            ctx.exhaust_card(card_id)

        # Deal damage for each exhausted card
        for _ in range(count):
            ctx.deal_damage_to_enemy(ctx.target, damage)


# -----------------------------------------------------------------------------
# Conditional Damage Effects
# -----------------------------------------------------------------------------

@effect_simple("damage_equals_block")
def damage_equals_block(ctx: EffectContext) -> None:
    """Body Slam - Deal damage equal to current Block (Java: uses calculateCardDamage pipeline)."""
    # Java sets baseDamage = p.currentBlock, then calls calculateCardDamage(m)
    # which applies Strength, Weak, Vulnerable, Stance modifiers
    base_damage = ctx.state.player.block
    if ctx.target and base_damage >= 0:
        ctx.deal_card_damage_to_enemy(ctx.target, base_damage)


@effect_simple("damage_per_strike")
def damage_per_strike(ctx: EffectContext) -> None:
    """Perfected Strike - Bonus damage per Strike card in deck."""
    bonus_per = ctx.magic_number if ctx.magic_number > 0 else 2
    strike_count = 0

    # Count Strikes in all piles
    for pile in [ctx.state.hand, ctx.state.draw_pile, ctx.state.discard_pile, ctx.state.exhaust_pile]:
        for card_id in pile:
            if "Strike" in card_id:
                strike_count += 1

    bonus = strike_count * bonus_per
    if ctx.target and bonus > 0:
        ctx.deal_damage_to_enemy(ctx.target, bonus)


@effect_simple("strength_multiplier")
def strength_multiplier(ctx: EffectContext) -> None:
    """Heavy Blade - Strength affects this card 3/5 times instead of 1."""
    # Handled in damage calculation (executor/combat engine).
    pass


@effect_simple("increase_damage_on_use")
def increase_damage_on_use(ctx: EffectContext) -> None:
    """Rampage - Increase base damage by 5/8 each time played."""
    increase = ctx.magic_number if ctx.magic_number > 0 else 5
    if not hasattr(ctx.state, 'rampage_bonus'):
        ctx.state.rampage_bonus = {}
    card_key = ctx.card.id if ctx.card else "Rampage"
    current = ctx.state.rampage_bonus.get(card_key, 0)
    # Deal the bonus damage
    if ctx.target and current > 0:
        ctx.deal_damage_to_enemy(ctx.target, current)
    # Increase for next time
    ctx.state.rampage_bonus[card_key] = current + increase


@effect_simple("random_enemy_x_times")
def random_enemy_x_times(ctx: EffectContext) -> None:
    """Sword Boomerang - Deal damage to random enemy X times."""
    if ctx.card:
        damage = ctx.card.damage
        hits = ctx.magic_number if ctx.magic_number > 0 else 3
        for _ in range(hits):
            ctx.deal_damage_to_random_enemy(damage)


@effect_simple("if_vulnerable_draw_and_energy")
def if_vulnerable_draw_and_energy(ctx: EffectContext) -> None:
    """Dropkick - If enemy is Vulnerable, draw 1 and gain 1 energy."""
    if ctx.target and ctx.target.statuses.get("Vulnerable", 0) > 0:
        ctx.draw_cards(1)
        ctx.gain_energy(1)


@effect_simple("damage_all_heal_unblocked")
def damage_all_heal_unblocked(ctx: EffectContext) -> None:
    """Reaper - Deal damage to all enemies, heal for unblocked damage."""
    if ctx.card:
        damage = ctx.card.damage
        total_hp_damage = 0

        for enemy in ctx.living_enemies:
            # Calculate unblocked damage
            blocked = min(enemy.block, damage)
            hp_damage = damage - blocked
            enemy.block -= blocked
            enemy.hp -= hp_damage
            if enemy.hp < 0:
                enemy.hp = 0
            total_hp_damage += hp_damage

        # Heal for total unblocked damage
        if total_hp_damage > 0:
            ctx.heal_player(total_hp_damage)


@effect_simple("if_fatal_gain_max_hp")
def if_fatal_gain_max_hp(ctx: EffectContext) -> None:
    """Feed - If this kills, gain 3/4 max HP."""
    max_hp_gain = ctx.magic_number if ctx.magic_number > 0 else 3
    ctx.extra_data["fatal_max_hp"] = max_hp_gain
    # Actual max HP gain happens in combat engine when kill confirmed


# -----------------------------------------------------------------------------
# Status Application Effects
# -----------------------------------------------------------------------------

@effect_simple("apply_weak_all")
def apply_weak_all(ctx: EffectContext) -> None:
    """Intimidate - Apply Weak to ALL enemies."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 1
    ctx.apply_status_to_all_enemies("Weak", amount)


@effect_simple("apply_vulnerable_1_all")
def apply_vulnerable_1_all(ctx: EffectContext) -> None:
    """Thunderclap - Apply 1 Vulnerable to ALL enemies."""
    ctx.apply_status_to_all_enemies("Vulnerable", 1)


@effect_simple("apply_weak_and_vulnerable")
def apply_weak_and_vulnerable(ctx: EffectContext) -> None:
    """Uppercut - Apply Weak and Vulnerable to target."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 1
    ctx.apply_status_to_target("Weak", amount)
    ctx.apply_status_to_target("Vulnerable", amount)


@effect_simple("apply_weak_and_vulnerable_all")
def apply_weak_and_vulnerable_all(ctx: EffectContext) -> None:
    """Shockwave - Apply Weak and Vulnerable to ALL enemies."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 3
    ctx.apply_status_to_all_enemies("Weak", amount)
    ctx.apply_status_to_all_enemies("Vulnerable", amount)


@effect_simple("gain_strength_if_enemy_attacking")
def gain_strength_if_enemy_attacking(ctx: EffectContext) -> None:
    """Spot Weakness - Gain Strength if enemy is attacking."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 3
    if ctx.is_enemy_attacking():
        ctx.apply_status_to_player("Strength", amount)


# -----------------------------------------------------------------------------
# X-Cost Effects
# -----------------------------------------------------------------------------

@effect_simple("damage_all_x_times")
def damage_all_x_times(ctx: EffectContext) -> None:
    """Whirlwind - Deal damage to ALL enemies X times (X = energy spent)."""
    if ctx.card:
        damage = ctx.card.damage
        # X is energy spent (stored in context)
        x = ctx.energy_spent if hasattr(ctx, 'energy_spent') else ctx.state.energy
        for _ in range(x):
            ctx.deal_damage_to_all_enemies(damage)


# -----------------------------------------------------------------------------
# Power Card Effects (apply as statuses that trigger)
# -----------------------------------------------------------------------------

@effect_simple("block_not_lost")
def block_not_lost(ctx: EffectContext) -> None:
    """Barricade - Block is not removed at start of turn."""
    ctx.apply_status_to_player("Barricade", 1)


@effect_simple("gain_vulnerable_gain_energy_per_turn")
def gain_vulnerable_gain_energy_per_turn(ctx: EffectContext) -> None:
    """Berserk - Gain 2/1 Vulnerable, gain 1 energy each turn."""
    vuln_amount = ctx.magic_number if ctx.magic_number > 0 else 2
    ctx.apply_status_to_player("Vulnerable", vuln_amount)
    ctx.apply_status_to_player("Berserk", 1)


@effect_simple("start_turn_lose_hp_draw")
def start_turn_lose_hp_draw(ctx: EffectContext) -> None:
    """Brutality - At start of turn, lose 1 HP and draw 1 card."""
    ctx.apply_status_to_player("Brutality", 1)


@effect_simple("skills_cost_0_exhaust")
def skills_cost_0_exhaust(ctx: EffectContext) -> None:
    """Corruption - Skills cost 0 but Exhaust."""
    ctx.apply_status_to_player("Corruption", 1)


@effect_simple("draw_on_exhaust")
def draw_on_exhaust(ctx: EffectContext) -> None:
    """Dark Embrace - Draw 1 card whenever a card is exhausted."""
    ctx.apply_status_to_player("DarkEmbrace", 1)


@effect_simple("draw_on_status")
def draw_on_status(ctx: EffectContext) -> None:
    """Evolve - Draw 1/2 cards whenever you draw a Status."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 1
    ctx.apply_status_to_player("Evolve", amount)


@effect_simple("block_on_exhaust")
def block_on_exhaust(ctx: EffectContext) -> None:
    """Feel No Pain - Gain 3/4 Block whenever a card is exhausted."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 3
    ctx.apply_status_to_player("FeelNoPain", amount)


@effect_simple("damage_on_status_curse")
def damage_on_status_curse(ctx: EffectContext) -> None:
    """Fire Breathing - Deal 6/10 damage to ALL enemies when drawing Status/Curse."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 6
    ctx.apply_status_to_player("FireBreathing", amount)


@effect_simple("when_attacked_deal_damage")
def when_attacked_deal_damage(ctx: EffectContext) -> None:
    """Flame Barrier - When attacked, deal 4/6 damage back."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 4
    ctx.apply_status_to_player("FlameBarrier", amount)


@effect_simple("gain_strength_each_turn")
def gain_strength_each_turn(ctx: EffectContext) -> None:
    """Demon Form - Gain 2/3 Strength at start of each turn."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 2
    ctx.apply_status_to_player("DemonForm", amount)


@effect_simple("end_turn_gain_block")
def end_turn_gain_block(ctx: EffectContext) -> None:
    """Metallicize - Gain 3/4 Block at end of each turn."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 3
    ctx.apply_status_to_player("Metallicize", amount)


@effect_simple("end_turn_damage_all_lose_hp")
def end_turn_damage_all_lose_hp(ctx: EffectContext) -> None:
    """Combust - At end of turn, lose 1 HP and deal 5/7 to all enemies."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 5
    ctx.apply_status_to_player("Combust", amount)


@effect_simple("gain_strength_on_hp_loss")
def gain_strength_on_hp_loss(ctx: EffectContext) -> None:
    """Rupture - Gain 1/2 Strength when losing HP from a card."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 1
    ctx.apply_status_to_player("Rupture", amount)


@effect_simple("damage_random_on_block")
def damage_random_on_block(ctx: EffectContext) -> None:
    """Juggernaut - Deal 5/7 damage to random enemy when gaining Block."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 5
    ctx.apply_status_to_player("Juggernaut", amount)


@effect_simple("gain_block_per_attack")
def gain_block_per_attack(ctx: EffectContext) -> None:
    """Rage - Gain 3/5 Block for each Attack played this turn."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 3
    ctx.apply_status_to_player("Rage", amount)


@effect_simple("play_attacks_twice")
def play_attacks_twice(ctx: EffectContext) -> None:
    """Double Tap - Next 1/2 Attacks are played twice."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 1
    ctx.apply_status_to_player("DoubleTap", amount)


@effect_simple("play_top_card")
def play_top_card(ctx: EffectContext) -> None:
    """Havoc - Play the top card of draw pile and Exhaust it."""
    if not ctx.state.draw_pile:
        return

    card_id = ctx.state.draw_pile.pop()
    from ..content.cards import get_card, normalize_card_id, CardTarget
    base_id, upgraded = normalize_card_id(card_id)
    try:
        card = get_card(base_id, upgraded=upgraded)
    except Exception:
        ctx.state.exhaust_pile.append(card_id)
        ctx.cards_exhausted.append(card_id)
        return

    if card.cost != -2 and "unplayable" not in card.effects:
        from .executor import EffectExecutor
        target_idx = -1
        if card.target == CardTarget.ENEMY:
            living_indices = [
                i for i, enemy in enumerate(ctx.state.enemies) if not enemy.is_dead
            ]
            if living_indices:
                target_idx = random.choice(living_indices)
        executor = EffectExecutor(ctx.state)
        executor.play_card(card, target_idx=target_idx, free=True)

    ctx.state.exhaust_pile.append(card_id)
    ctx.cards_exhausted.append(card_id)


@effect_simple("gain_energy_on_exhaust_2_3")
def gain_energy_on_exhaust_2_3(ctx: EffectContext) -> None:
    """Sentinel - If exhausted, gain 2/3 energy."""
    # Handled when a Sentinel card is exhausted.
    pass


# -----------------------------------------------------------------------------
# Special/Misc Effects
# -----------------------------------------------------------------------------

@effect_simple("only_attacks_in_hand")
def only_attacks_in_hand(ctx: EffectContext) -> None:
    """Clash - Can only be played if only Attacks in hand (playability check)."""
    # Handled in can_play_card
    pass


@effect_simple("cost_reduces_when_damaged")
def cost_reduces_when_damaged(ctx: EffectContext) -> None:
    """Blood for Blood - Cost reduces by 1 each time you take damage."""
    # Tracked in combat state
    pass


@effect_simple("can_upgrade_unlimited")
def can_upgrade_unlimited(ctx: EffectContext) -> None:
    """Searing Blow - Can be upgraded multiple times."""
    # This affects upgrade logic, not combat effect
    pass


@effect_simple("upgrade_card_in_hand")
def upgrade_card_in_hand(ctx: EffectContext) -> None:
    """Armaments - Upgrade a card in hand (upgraded: all cards)."""
    if ctx.is_upgraded:
        # Upgrade all cards in hand
        new_hand = []
        for card_id in ctx.state.hand:
            if not card_id.endswith("+"):
                new_hand.append(card_id + "+")
            else:
                new_hand.append(card_id)
        ctx.state.hand = new_hand
    else:
        # Upgrade one card (first upgradeable)
        for i, card_id in enumerate(ctx.state.hand):
            if not card_id.endswith("+"):
                ctx.state.hand[i] = card_id + "+"
                break


# =============================================================================
# IRONCLAD CARD EFFECTS MAPPING
# =============================================================================

IRONCLAD_CARD_EFFECTS = {
    # === BASIC ===
    "Strike_R": [],
    "Defend_R": [],
    "Bash": ["apply_vulnerable"],

    # === COMMON ATTACKS ===
    "Anger": ["add_copy_to_discard"],
    "Body Slam": ["damage_equals_block"],
    "Clash": ["only_attacks_in_hand"],
    "Cleave": [],  # AoE damage only
    "Clothesline": ["apply_weak"],
    "Headbutt": ["put_card_from_discard_on_draw"],
    "Heavy Blade": ["strength_multiplier"],
    "Iron Wave": [],  # Damage + block built in
    "Perfected Strike": ["damage_per_strike"],
    "Pommel Strike": ["draw_cards"],
    "Sword Boomerang": ["random_enemy_x_times"],
    "Thunderclap": ["apply_vulnerable_1_all"],
    "Twin Strike": ["damage_x_times"],
    "Wild Strike": ["shuffle_wound_into_draw"],

    # === COMMON SKILLS ===
    "Armaments": ["upgrade_card_in_hand"],
    "Flex": ["gain_temp_strength"],
    "Havoc": ["play_top_card"],
    "Shrug It Off": ["draw_1"],
    "True Grit": ["exhaust_random_card"],
    "Warcry": ["draw_then_put_on_draw"],

    # === UNCOMMON ATTACKS ===
    "Blood for Blood": ["cost_reduces_when_damaged"],
    "Carnage": [],  # Just ethereal damage
    "Dropkick": ["if_vulnerable_draw_and_energy"],
    "Hemokinesis": ["lose_hp"],
    "Pummel": ["damage_x_times"],
    "Rampage": ["increase_damage_on_use"],
    "Reckless Charge": ["shuffle_dazed_into_draw"],
    "Searing Blow": ["can_upgrade_unlimited"],
    "Sever Soul": ["exhaust_all_non_attacks"],
    "Uppercut": ["apply_weak_and_vulnerable"],
    "Whirlwind": ["damage_all_x_times"],

    # === UNCOMMON SKILLS ===
    "Battle Trance": ["draw_then_no_draw"],
    "Bloodletting": ["lose_hp_gain_energy"],
    "Burning Pact": ["exhaust_to_draw"],
    "Disarm": ["reduce_enemy_strength"],
    "Dual Wield": ["copy_attack_or_power"],
    "Entrench": ["double_block"],
    "Flame Barrier": ["when_attacked_deal_damage"],
    "Ghostly Armor": [],  # Ethereal block only
    "Infernal Blade": ["add_random_attack_cost_0"],
    "Intimidate": ["apply_weak_all"],
    "Power Through": ["add_wounds_to_hand"],
    "Rage": ["gain_block_per_attack"],
    "Second Wind": ["exhaust_non_attacks_gain_block"],
    "Seeing Red": ["gain_2_energy"],
    "Sentinel": ["gain_energy_on_exhaust_2_3"],
    "Shockwave": ["apply_weak_and_vulnerable_all"],
    "Spot Weakness": ["gain_strength_if_enemy_attacking"],

    # === UNCOMMON POWERS ===
    "Combust": ["end_turn_damage_all_lose_hp"],
    "Dark Embrace": ["draw_on_exhaust"],
    "Evolve": ["draw_on_status"],
    "Feel No Pain": ["block_on_exhaust"],
    "Fire Breathing": ["damage_on_status_curse"],
    "Inflame": ["gain_strength"],
    "Metallicize": ["end_turn_gain_block"],
    "Rupture": ["gain_strength_on_hp_loss"],

    # === RARE ATTACKS ===
    "Bludgeon": [],  # Big damage only
    "Feed": ["if_fatal_gain_max_hp"],
    "Fiend Fire": ["exhaust_hand_damage_per_card"],
    "Immolate": ["add_burn_to_discard"],
    "Reaper": ["damage_all_heal_unblocked"],

    # === RARE SKILLS ===
    "Double Tap": ["play_attacks_twice"],
    "Exhume": ["return_exhausted_card_to_hand"],
    "Impervious": [],  # Big block only
    "Limit Break": ["double_strength"],
    "Offering": ["lose_hp_gain_energy_draw"],

    # === RARE POWERS ===
    "Barricade": ["block_not_lost"],
    "Berserk": ["gain_vulnerable_gain_energy_per_turn"],
    "Brutality": ["start_turn_lose_hp_draw"],
    "Corruption": ["skills_cost_0_exhaust"],
    "Demon Form": ["gain_strength_each_turn"],
    "Juggernaut": ["damage_random_on_block"],
}
# SILENT CARD EFFECTS
# =============================================================================

# -----------------------------------------------------------------------------
# Poison Effects
# -----------------------------------------------------------------------------

@effect_simple("apply_poison")
def apply_poison(ctx: EffectContext) -> None:
    """Apply Poison to target (Deadly Poison, Poisoned Stab, etc.)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 5
    ctx.apply_status_to_target("Poison", amount)


@effect_simple("double_poison")
def double_poison(ctx: EffectContext) -> None:
    """Double (or triple if upgraded) the target's Poison (Catalyst)."""
    if ctx.target:
        current_poison = ctx.target.statuses.get("Poison", 0)
        if current_poison > 0:
            multiplier = 3 if ctx.is_upgraded else 2
            new_poison = current_poison * (multiplier - 1)  # Add the difference
            ctx.apply_status_to_target("Poison", new_poison)


@effect_simple("apply_poison_all")
def apply_poison_all(ctx: EffectContext) -> None:
    """Apply Poison to all enemies (Crippling Poison)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 4
    ctx.apply_status_to_all_enemies("Poison", amount)


@effect_simple("apply_weak_2_all")
def apply_weak_2_all(ctx: EffectContext) -> None:
    """Apply Weak to all enemies (Crippling Poison)."""
    ctx.apply_status_to_all_enemies("Weak", 2)


@effect_simple("apply_poison_random_3_times")
def apply_poison_random_3_times(ctx: EffectContext) -> None:
    """Apply Poison to random enemies 3 times (Bouncing Flask).

    Uses deterministic RNG based on combat state for reproducibility.
    In Java, this uses AbstractDungeon.cardRandomRng.
    """
    amount = ctx.magic_number if ctx.magic_number > 0 else 3
    living = ctx.living_enemies
    if living:
        # Use deterministic selection based on card_rng_state for reproducibility
        # This ensures the same seed produces the same targeting
        seed0, seed1 = ctx.state.card_rng_state
        for i in range(3):
            # Simple deterministic index based on RNG state, turn, and bounce number
            idx = (seed0 + seed1 + ctx.state.turn * 7 + i * 13) % len(living)
            target = living[idx]
            ctx.apply_status_to_enemy(target, "Poison", amount)


@effect_simple("apply_poison_all_each_turn")
def apply_poison_all_each_turn(ctx: EffectContext) -> None:
    """Noxious Fumes - Apply Poison to all enemies at start of turn (power)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 2
    ctx.apply_status_to_player("NoxiousFumes", amount)


@effect_simple("apply_corpse_explosion")
def apply_corpse_explosion(ctx: EffectContext) -> None:
    """Apply Corpse Explosion to target (Corpse Explosion)."""
    ctx.apply_status_to_target("CorpseExplosion", 1)


# -----------------------------------------------------------------------------
# Shiv Effects
# -----------------------------------------------------------------------------

@effect_simple("add_shivs_to_hand")
def add_shivs_to_hand(ctx: EffectContext) -> None:
    """Add Shivs to hand (Blade Dance, Cloak and Dagger)."""
    count = ctx.magic_number if ctx.magic_number > 0 else 3
    # Check Master Reality for upgrades
    upgraded = ctx.get_player_status("MasterReality") > 0
    # Check Accuracy for shiv damage bonus (applied via power, not here)
    for _ in range(count):
        card_id = "Shiv+" if upgraded else "Shiv"
        ctx.add_card_to_hand(card_id)


@effect_simple("add_shiv_each_turn")
def add_shiv_each_turn(ctx: EffectContext) -> None:
    """Infinite Blades - Add a Shiv to hand at start of each turn (power)."""
    ctx.apply_status_to_player("InfiniteBlades", 1)


@effect_simple("shivs_deal_more_damage")
def shivs_deal_more_damage(ctx: EffectContext) -> None:
    """Accuracy - Shivs deal extra damage (power)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 4
    ctx.apply_status_to_player("Accuracy", amount)


@effect_simple("add_shivs_equal_to_discarded")
def add_shivs_equal_to_discarded(ctx: EffectContext) -> None:
    """Add Shivs equal to cards discarded (Storm of Steel)."""
    # Cards are discarded first, count is stored in extra_data
    count = ctx.extra_data.get("cards_discarded_count", len(ctx.hand))
    upgraded = ctx.is_upgraded or ctx.get_player_status("MasterReality") > 0
    for _ in range(count):
        card_id = "Shiv+" if upgraded else "Shiv"
        ctx.add_card_to_hand(card_id)


# -----------------------------------------------------------------------------
# Discard Effects
# -----------------------------------------------------------------------------

@effect_simple("discard_1")
def discard_1(ctx: EffectContext) -> None:
    """Discard 1 card (Survivor, Dagger Throw, Acrobatics)."""
    # In simulation, discard first non-essential card
    # In actual game, this requires selection
    if ctx.hand:
        # Mark that discard selection is needed
        ctx.extra_data["discard_selection_needed"] = 1
        # For simulation, discard the last card
        card = ctx.hand[-1]
        ctx.discard_card(card)


@effect_simple("discard_x")
def discard_x(ctx: EffectContext) -> None:
    """Discard X cards (Prepared, Concentrate)."""
    count = ctx.magic_number if ctx.magic_number > 0 else 1
    ctx.extra_data["discard_selection_needed"] = count
    # For simulation, discard from end of hand
    for _ in range(min(count, len(ctx.hand))):
        if ctx.hand:
            card = ctx.hand[-1]
            ctx.discard_card(card)


@effect_simple("discard_random_1")
def discard_random_1(ctx: EffectContext) -> None:
    """Discard a random card (All-Out Attack)."""
    if ctx.hand:
        card = random.choice(ctx.hand)
        ctx.discard_card(card)


@effect_simple("discard_hand")
def discard_hand(ctx: EffectContext) -> None:
    """Discard entire hand (Storm of Steel, Calculated Gamble)."""
    count = len(ctx.hand)
    ctx.extra_data["cards_discarded_count"] = count
    for card in ctx.hand.copy():
        ctx.discard_card(card)


@effect_simple("discard_hand_draw_same")
def discard_hand_draw_same(ctx: EffectContext) -> None:
    """Discard hand and draw same number (Calculated Gamble)."""
    count = len(ctx.hand)
    # Discard all cards
    for card in ctx.hand.copy():
        ctx.discard_card(card)
    # Draw same number
    ctx.draw_cards(count)


@effect_simple("discard_non_attacks")
def discard_non_attacks(ctx: EffectContext) -> None:
    """Discard all non-Attack cards (Unload)."""
    for card_id in ctx.hand.copy():
        if not _is_attack_card(card_id):
            ctx.discard_card(card_id)


# -----------------------------------------------------------------------------
# Discard Trigger Effects (Reflex, Tactician)
# -----------------------------------------------------------------------------

@effect_simple("when_discarded_draw")
def when_discarded_draw(ctx: EffectContext) -> None:
    """When discarded, draw cards (Reflex)."""
    # This is a passive effect handled by the discard system
    # The power marker is set here for tracking
    pass


@effect_simple("when_discarded_gain_energy")
def when_discarded_gain_energy(ctx: EffectContext) -> None:
    """When discarded, gain energy (Tactician)."""
    # This is a passive effect handled by the discard system
    pass


@effect_simple("cost_reduces_per_discard")
def cost_reduces_per_discard(ctx: EffectContext) -> None:
    """Cost reduces by 1 for each card discarded this turn (Eviscerate)."""
    # This is tracked by the combat system
    pass


@effect_simple("refund_2_energy_if_discarded_this_turn")
def refund_2_energy_if_discarded_this_turn(ctx: EffectContext) -> None:
    """Refund 2 energy if a card was discarded this turn (Sneaky Strike)."""
    discarded_this_turn = getattr(ctx.state, "discarded_this_turn", 0)
    if discarded_this_turn > 0:
        ctx.gain_energy(2)


# -----------------------------------------------------------------------------
# Draw Effects
# -----------------------------------------------------------------------------

@effect_simple("draw_x")
def draw_x(ctx: EffectContext) -> None:
    """Draw X cards based on magic number (Acrobatics, Prepared)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 3
    ctx.draw_cards(amount)


@effect_simple("draw_to_x_cards")
def draw_to_x_cards(ctx: EffectContext) -> None:
    """Draw until you have X cards in hand (Expertise)."""
    target_hand_size = ctx.magic_number if ctx.magic_number > 0 else 6
    cards_to_draw = max(0, target_hand_size - len(ctx.hand))
    if cards_to_draw > 0:
        ctx.draw_cards(cards_to_draw)


@effect_simple("draw_2_next_turn")
def draw_2_next_turn(ctx: EffectContext) -> None:
    """Draw 2 cards next turn (Predator)."""
    ctx.apply_status_to_player("NextTurnDraw", 2)


@effect_simple("draw_x_next_turn")
def draw_x_next_turn(ctx: EffectContext) -> None:
    """Draw X cards next turn (Doppelganger)."""
    x = ctx.energy_spent if hasattr(ctx, 'energy_spent') and ctx.energy_spent > 0 else ctx.energy
    bonus = 1 if ctx.is_upgraded else 0
    ctx.apply_status_to_player("NextTurnDraw", x + bonus)


@effect_simple("draw_1_discard_1_each_turn")
def draw_1_discard_1_each_turn(ctx: EffectContext) -> None:
    """Tools of the Trade - Draw 1, discard 1 at start of each turn (power)."""
    ctx.apply_status_to_player("ToolsOfTheTrade", 1)


# -----------------------------------------------------------------------------
# Block Effects
# -----------------------------------------------------------------------------

@effect_simple("block_next_turn")
def block_next_turn(ctx: EffectContext) -> None:
    """Gain block at start of next turn (Dodge and Roll)."""
    # The block amount is the card's base block
    amount = ctx.card.block if ctx.card else 4
    ctx.apply_status_to_player("NextTurnBlock", amount)


@effect_simple("block_not_removed_next_turn")
def block_not_removed_next_turn(ctx: EffectContext) -> None:
    """Block is not removed at start of next turn (Blur)."""
    ctx.apply_status_to_player("Blur", 1)


@effect_simple("gain_1_block_per_card_played")
def gain_1_block_per_card_played(ctx: EffectContext) -> None:
    """After Image - Gain 1 block when playing any card (power)."""
    ctx.apply_status_to_player("AfterImage", 1)


@effect_simple("if_skill_drawn_gain_block")
def if_skill_drawn_gain_block(ctx: EffectContext) -> None:
    """If the card drawn was a Skill, gain block (Escape Plan)."""
    # The draw happens first, then we check what was drawn
    if ctx.cards_drawn:
        last_drawn = ctx.cards_drawn[-1]
        if _is_skill_card(last_drawn):
            # Gain the card's block again
            amount = ctx.card.block if ctx.card else 3
            ctx.gain_block(amount)


# -----------------------------------------------------------------------------
# Energy Effects
# -----------------------------------------------------------------------------

@effect_simple("gain_energy_next_turn")
def gain_energy_next_turn(ctx: EffectContext) -> None:
    """Gain energy next turn (Outmaneuver)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 2
    ctx.apply_status_to_player("NextTurnEnergy", amount)


@effect_simple("gain_energy_next_turn_1")
def gain_energy_next_turn_1(ctx: EffectContext) -> None:
    """Gain 1 energy next turn (Flying Knee)."""
    ctx.apply_status_to_player("NextTurnEnergy", 1)


@effect_simple("gain_x_energy_next_turn")
def gain_x_energy_next_turn(ctx: EffectContext) -> None:
    """Gain X energy next turn (Doppelganger)."""
    x = ctx.energy_spent if hasattr(ctx, 'energy_spent') and ctx.energy_spent > 0 else ctx.energy
    bonus = 1 if ctx.is_upgraded else 0
    ctx.apply_status_to_player("NextTurnEnergy", x + bonus)


@effect_simple("gain_energy_2")
def gain_energy_2(ctx: EffectContext) -> None:
    """Gain 2 energy (Concentrate)."""
    ctx.gain_energy(2)


# -----------------------------------------------------------------------------
# Damage Effects
# -----------------------------------------------------------------------------

@effect_simple("double_damage_if_poisoned")
def double_damage_if_poisoned(ctx: EffectContext) -> None:
    """Deal double damage if target is poisoned (Bane)."""
    if ctx.target and ctx.target.statuses.get("Poison", 0) > 0:
        # Deal additional damage equal to base damage
        damage = ctx.card.damage if ctx.card else 7
        ctx.deal_damage_to_target(damage)


@effect_simple("damage_all_x_times")
def damage_all_x_times(ctx: EffectContext) -> None:
    """Deal damage to all enemies X times (Dagger Spray)."""
    hits = ctx.magic_number if ctx.magic_number > 0 else 2
    damage = ctx.card.damage if ctx.card else 4
    for _ in range(hits):
        for enemy in ctx.living_enemies:
            ctx.deal_damage_to_enemy(enemy, damage)


@effect_simple("damage_per_attack_this_turn")
def damage_per_attack_this_turn(ctx: EffectContext) -> None:
    """Deal damage for each attack played this turn (Finisher)."""
    attacks_played = ctx.state.attacks_played_this_turn
    damage = ctx.card.damage if ctx.card else 6
    for _ in range(attacks_played):
        ctx.deal_damage_to_target(damage)


@effect_simple("damage_per_skill_in_hand")
def damage_per_skill_in_hand(ctx: EffectContext) -> None:
    """Deal damage for each skill in hand (Flechettes)."""
    skill_count = sum(1 for c in ctx.hand if _is_skill_card(c))
    damage = ctx.card.damage if ctx.card else 4
    for _ in range(skill_count):
        ctx.deal_damage_to_target(damage)


@effect_simple("damage_x_times_energy")
def damage_x_times_energy(ctx: EffectContext) -> None:
    """Deal damage X times where X is energy spent (Skewer)."""
    x = ctx.energy_spent if hasattr(ctx, 'energy_spent') and ctx.energy_spent > 0 else ctx.energy
    damage = ctx.card.damage if ctx.card else 7
    for _ in range(x):
        ctx.deal_damage_to_target(damage)


@effect_simple("reduce_damage_by_2")
def reduce_damage_by_2(ctx: EffectContext) -> None:
    """Reduce card's damage by 2 after playing (Glass Knife)."""
    # This is tracked by the combat system for the card instance
    ctx.extra_data["reduce_damage_this_combat"] = 2


@effect_simple("deal_damage_per_card_played")
def deal_damage_per_card_played(ctx: EffectContext) -> None:
    """A Thousand Cuts - Deal damage to all enemies per card played (power)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 1
    ctx.apply_status_to_player("ThousandCuts", amount)


# -----------------------------------------------------------------------------
# Conditional Effects
# -----------------------------------------------------------------------------

@effect_simple("if_target_weak_gain_energy_draw")
def if_target_weak_gain_energy_draw(ctx: EffectContext) -> None:
    """If target is Weak, gain energy and draw (Heel Hook)."""
    if ctx.target and ctx.target.statuses.get("Weak", 0) > 0:
        ctx.gain_energy(1)
        ctx.draw_cards(1)


@effect_simple("cost_increases_when_damaged")
def cost_increases_when_damaged(ctx: EffectContext) -> None:
    """Cost increases by 1 each time you take damage (Masterful Stab)."""
    # This is tracked by the combat system
    pass


@effect_simple("only_playable_if_draw_pile_empty")
def only_playable_if_draw_pile_empty(ctx: EffectContext) -> None:
    """Grand Finale - only playable if draw pile is empty."""
    # This is a playability check, handled in can_play_card
    pass


# -----------------------------------------------------------------------------
# Power Effects
# -----------------------------------------------------------------------------

@effect_simple("gain_dexterity")
def gain_dexterity(ctx: EffectContext) -> None:
    """Gain Dexterity (Footwork)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 2
    ctx.apply_status_to_player("Dexterity", amount)


@effect_simple("gain_thorns")
def gain_thorns(ctx: EffectContext) -> None:
    """Gain Thorns (Caltrops)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 3
    ctx.apply_status_to_player("Thorns", amount)


@effect_simple("retain_cards_each_turn")
def retain_cards_each_turn(ctx: EffectContext) -> None:
    """Well-Laid Plans - Retain cards at end of turn (power)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 1
    ctx.apply_status_to_player("WellLaidPlans", amount)


@effect_simple("attacks_apply_poison")
def attacks_apply_poison(ctx: EffectContext) -> None:
    """Envenom - Attacks apply Poison (power)."""
    ctx.apply_status_to_player("Envenom", 1)


@effect_simple("gain_intangible")
def gain_intangible(ctx: EffectContext) -> None:
    """Gain Intangible (Wraith Form)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 2
    ctx.apply_status_to_player("Intangible", amount)


@effect_simple("lose_1_dexterity_each_turn")
def lose_1_dexterity_each_turn(ctx: EffectContext) -> None:
    """Lose 1 Dexterity at end of each turn (Wraith Form)."""
    ctx.apply_status_to_player("WraithFormPower", 1)


@effect_simple("double_damage_next_turn")
def double_damage_next_turn(ctx: EffectContext) -> None:
    """Double damage dealt next turn (Phantasmal Killer)."""
    ctx.apply_status_to_player("PhantasmalKiller", 1)


# -----------------------------------------------------------------------------
# Strength Reduction Effects
# -----------------------------------------------------------------------------

@effect_simple("reduce_strength_all_enemies")
def reduce_strength_all_enemies(ctx: EffectContext) -> None:
    """Reduce Strength of all enemies (Piercing Wail)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 6
    for enemy in ctx.living_enemies:
        current = enemy.statuses.get("Strength", 0)
        enemy.statuses["Strength"] = current - amount
        # Also track temporary strength loss so it returns at end of turn
        current_loss = enemy.statuses.get("TempStrengthLoss", 0)
        enemy.statuses["TempStrengthLoss"] = current_loss + amount


@effect_simple("apply_choke")
def apply_choke(ctx: EffectContext) -> None:
    """Apply Choke to target (Choke)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 3
    ctx.apply_status_to_target("Choked", amount)


# -----------------------------------------------------------------------------
# X-Cost Effects
# -----------------------------------------------------------------------------

@effect_simple("apply_weak_x")
def apply_weak_x(ctx: EffectContext) -> None:
    """Apply X Weak to target (Malaise)."""
    x = ctx.energy_spent if hasattr(ctx, 'energy_spent') and ctx.energy_spent > 0 else ctx.energy
    bonus = 1 if ctx.is_upgraded else 0
    ctx.apply_status_to_target("Weak", x + bonus)


@effect_simple("apply_strength_down_x")
def apply_strength_down_x(ctx: EffectContext) -> None:
    """Apply X permanent Strength down to target (Malaise)."""
    x = ctx.energy_spent if hasattr(ctx, 'energy_spent') and ctx.energy_spent > 0 else ctx.energy
    bonus = 1 if ctx.is_upgraded else 0
    if ctx.target:
        current = ctx.target.statuses.get("Strength", 0)
        ctx.target.statuses["Strength"] = current - (x + bonus)


# -----------------------------------------------------------------------------
# Special Card Effects
# -----------------------------------------------------------------------------

@effect_simple("double_next_skills")
def double_next_skills(ctx: EffectContext) -> None:
    """Next X skills are played twice (Burst)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 1
    ctx.apply_status_to_player("Burst", amount)


@effect_simple("copy_to_hand_when_drawn")
def copy_to_hand_when_drawn(ctx: EffectContext) -> None:
    """Copy to hand when drawn (Endless Agony)."""
    # This is a triggered effect handled by the draw system
    pass


@effect_simple("copy_card_to_hand_next_turn")
def copy_card_to_hand_next_turn(ctx: EffectContext) -> None:
    """Copy a card to hand at start of next turn (Nightmare)."""
    # This requires card selection
    # Store that Nightmare was played
    copies = ctx.magic_number if ctx.magic_number > 0 else 3
    ctx.extra_data["nightmare_copies"] = copies
    ctx.extra_data["nightmare_selection_needed"] = True


@effect_simple("put_card_on_draw_pile_cost_0")
def put_card_on_draw_pile_cost_0(ctx: EffectContext) -> None:
    """Put a card from hand on top of draw pile; it costs 0 (Setup)."""
    ctx.extra_data["setup_selection_needed"] = True


@effect_simple("add_random_skill_cost_0")
def add_random_skill_cost_0(ctx: EffectContext) -> None:
    """Add a random Skill to hand that costs 0 (Distraction)."""
    from ..content.cards import ALL_CARDS, CardType, CardColor
    skills = [
        cid for cid, c in ALL_CARDS.items()
        if c.card_type == CardType.SKILL and c.color == CardColor.GREEN
        and c.rarity.value not in ["SPECIAL", "CURSE", "STATUS"]
    ]
    if skills:
        card_id = random.choice(skills)
        ctx.add_card_to_hand(card_id)
        # Mark card as cost 0 this turn
        ctx.state.card_costs = getattr(ctx.state, 'card_costs', {})
        ctx.state.card_costs[card_id] = 0


@effect_simple("no_draw_this_turn")
def no_draw_this_turn(ctx: EffectContext) -> None:
    """Cannot draw cards for rest of turn (Bullet Time)."""
    ctx.apply_status_to_player("NoDraw", 1)


@effect_simple("cards_cost_0_this_turn")
def cards_cost_0_this_turn(ctx: EffectContext) -> None:
    """All cards cost 0 for rest of turn (Bullet Time)."""
    ctx.apply_status_to_player("ZeroCostCards", 1)


@effect_simple("obtain_random_potion")
def obtain_random_potion(ctx: EffectContext) -> None:
    """Obtain a random potion (Alchemize)."""
    ctx.extra_data["obtain_potion"] = True


# -----------------------------------------------------------------------------
# Helper Functions
# -----------------------------------------------------------------------------

def _is_skill_card(card_id: str) -> bool:
    """Check if a card is a Skill type."""
    skill_ids = {
        "Defend_P", "Defend_G", "Defend_R", "Defend_B",
        "Vigilance", "ClearTheMind", "Crescendo", "EmptyBody", "EmptyMind",
        "Evaluate", "Halt", "InnerPeace", "PathToVictory", "Protect",
        "ThirdEye", "Prostrate", "Collect", "DeceiveReality", "Indignation",
        "Meditate", "Perseverance", "Pray", "Sanctity", "Swivel",
        "WaveOfTheHand", "Worship", "WreathOfFlame", "Alpha", "Blasphemy",
        "ConjureBlade", "Omniscience", "Scrawl", "SpiritShield", "Vault", "Wish",
        # Silent skills
        "Survivor", "Acrobatics", "Backflip", "Blade Dance", "Cloak And Dagger",
        "Deadly Poison", "Deflect", "Dodge and Roll", "Outmaneuver",
        "PiercingWail", "Prepared", "Blur", "Bouncing Flask",
        "Calculated Gamble", "Catalyst", "Concentrate", "Crippling Poison",
        "Distraction", "Escape Plan", "Expertise", "Leg Sweep", "Reflex",
        "Setup", "Tactician", "Terror", "Adrenaline", "Venomology",
        "Bullet Time", "Burst", "Corpse Explosion", "Doppelganger",
        "Malaise", "Night Terror", "Phantasmal Killer", "Storm of Steel",
        # Special
        "Miracle", "Insight", "Safety",
    }
    base_id = card_id.rstrip("+")
    return base_id in skill_ids


# =============================================================================
# SILENT CARD EFFECTS REGISTRY
# =============================================================================

SILENT_CARD_EFFECTS = {
    # === BASIC ===
    "Strike_G": [],  # Just damage
    "Defend_G": [],  # Just block
    "Neutralize": ["apply_weak"],
    "Survivor": ["discard_1"],

    # === COMMON ATTACKS ===
    "Bane": ["double_damage_if_poisoned"],
    "Dagger Spray": ["damage_all_x_times"],
    "Dagger Throw": ["draw_1", "discard_1"],
    "Flying Knee": ["gain_energy_next_turn_1"],
    "Poisoned Stab": ["apply_poison"],
    "Quick Slash": ["draw_1"],
    "Slice": [],  # Just damage
    "Underhanded Strike": ["refund_2_energy_if_discarded_this_turn"],
    "Sucker Punch": ["apply_weak"],

    # === COMMON SKILLS ===
    "Acrobatics": ["draw_x", "discard_1"],
    "Backflip": ["draw_2"],
    "Blade Dance": ["add_shivs_to_hand"],
    "Cloak And Dagger": ["add_shivs_to_hand"],
    "Deadly Poison": ["apply_poison"],
    "Deflect": [],  # Just block
    "Dodge and Roll": ["block_next_turn"],
    "Outmaneuver": ["gain_energy_next_turn"],
    "PiercingWail": ["reduce_strength_all_enemies"],
    "Prepared": ["draw_x", "discard_x"],

    # === UNCOMMON ATTACKS ===
    "All Out Attack": ["discard_random_1"],
    "Backstab": [],  # Just damage
    "Choke": ["apply_choke"],
    "Dash": [],  # Damage + block handled by base stats
    "Endless Agony": ["copy_to_hand_when_drawn"],
    "Eviscerate": ["cost_reduces_per_discard", "damage_x_times"],
    "Finisher": ["damage_per_attack_this_turn"],
    "Flechettes": ["damage_per_skill_in_hand"],
    "Heel Hook": ["if_target_weak_gain_energy_draw"],
    "Masterful Stab": ["cost_increases_when_damaged"],
    "Predator": ["draw_2_next_turn"],
    "Riddle With Holes": ["damage_x_times"],
    "Skewer": ["damage_x_times_energy"],

    # === UNCOMMON SKILLS ===
    "Blur": ["block_not_removed_next_turn"],
    "Bouncing Flask": ["apply_poison_random_3_times"],
    "Calculated Gamble": ["discard_hand_draw_same"],
    "Catalyst": ["double_poison"],
    "Concentrate": ["discard_x", "gain_energy_2"],
    "Crippling Poison": ["apply_poison_all", "apply_weak_2_all"],
    "Distraction": ["add_random_skill_cost_0"],
    "Escape Plan": ["draw_1", "if_skill_drawn_gain_block"],
    "Expertise": ["draw_to_x_cards"],
    "Leg Sweep": ["apply_weak"],
    "Reflex": ["unplayable", "when_discarded_draw"],
    "Setup": ["put_card_on_draw_pile_cost_0"],
    "Tactician": ["unplayable", "when_discarded_gain_energy"],
    "Terror": ["apply_vulnerable"],

    # === UNCOMMON POWERS ===
    "Accuracy": ["shivs_deal_more_damage"],
    "Caltrops": ["gain_thorns"],
    "Footwork": ["gain_dexterity"],
    "Infinite Blades": ["add_shiv_each_turn"],
    "Noxious Fumes": ["apply_poison_all_each_turn"],
    "Well Laid Plans": ["retain_cards_each_turn"],

    # === RARE ATTACKS ===
    "Die Die Die": [],  # Just AoE damage + exhaust
    "Glass Knife": ["damage_x_times", "reduce_damage_by_2"],
    "Grand Finale": ["only_playable_if_draw_pile_empty"],
    "Unload": ["discard_non_attacks"],

    # === RARE SKILLS ===
    "Adrenaline": ["gain_energy", "draw_2"],
    "Venomology": ["obtain_random_potion"],  # Alchemize
    "Bullet Time": ["no_draw_this_turn", "cards_cost_0_this_turn"],
    "Burst": ["double_next_skills"],
    "Corpse Explosion": ["apply_poison", "apply_corpse_explosion"],
    "Doppelganger": ["draw_x_next_turn", "gain_x_energy_next_turn"],
    "Malaise": ["apply_weak_x", "apply_strength_down_x"],
    "Night Terror": ["copy_card_to_hand_next_turn"],  # Nightmare
    "Phantasmal Killer": ["double_damage_next_turn"],
    "Storm of Steel": ["discard_hand", "add_shivs_equal_to_discarded"],

    # === RARE POWERS ===
    "After Image": ["gain_1_block_per_card_played"],
    "A Thousand Cuts": ["deal_damage_per_card_played"],
    "Envenom": ["attacks_apply_poison"],
    "Tools of the Trade": ["draw_1_discard_1_each_turn"],
    "Wraith Form v2": ["gain_intangible", "lose_1_dexterity_each_turn"],

    # === SPECIAL ===
    "Shiv": [],  # Just damage + exhaust
}


