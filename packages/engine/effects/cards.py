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
        "Strike_P", "Eruption", "BowlingBash", "CutThroughFate",
        "EmptyFist", "FlurryOfBlows", "FlyingSleeves", "FollowUp",
        "JustLucky", "SashWhip", "Consecrate", "CrushJoints",
        "Tantrum", "FearNoEvil", "ReachHeaven", "SandsOfTime",
        "SignatureMove", "TalkToTheHand", "Wallop", "Weave",
        "WheelKick", "WindmillStrike", "Conclude", "CarveReality",
        "Brilliance", "Judgement", "LessonLearned", "Ragnarok",
        "Smite", "ThroughViolence", "Expunger",
        # Ironclad
        "Strike_R", "Bash", "Anger", "Cleave", "Clothesline",
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
