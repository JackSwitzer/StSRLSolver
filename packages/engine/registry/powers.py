"""
Power Trigger Implementations.

This module contains power trigger handlers using the registry pattern.
Powers are buffs/debuffs that trigger at various points during combat.

Organized by trigger hook for easier maintenance.
"""

from __future__ import annotations

from . import power_trigger, PowerContext


# =============================================================================
# AT_START_OF_TURN Triggers (before draw)
# =============================================================================

@power_trigger("atStartOfTurn", power="Poison")
def poison_start(ctx: PowerContext) -> None:
    """Poison: Deal damage and decrement at start of turn."""
    if ctx.owner and ctx.amount > 0:
        # Deal HP_LOSS damage (ignores block)
        ctx.owner.hp -= ctx.amount
        if ctx.owner.hp < 0:
            ctx.owner.hp = 0
        # Decrement
        ctx.owner.statuses["Poison"] = ctx.amount - 1
        if ctx.owner.statuses["Poison"] <= 0:
            del ctx.owner.statuses["Poison"]


@power_trigger("atStartOfTurn", power="Regeneration")
def regeneration_start(ctx: PowerContext) -> None:
    """Regeneration: Heal at start of turn, then decrement."""
    if ctx.owner == ctx.player:
        heal = min(ctx.amount, ctx.player.max_hp - ctx.player.hp)
        ctx.player.hp += heal
        # Decrement
        ctx.player.statuses["Regeneration"] = ctx.amount - 1
        if ctx.player.statuses["Regeneration"] <= 0:
            del ctx.player.statuses["Regeneration"]


@power_trigger("atStartOfTurn", power="Choked")
def choked_start(ctx: PowerContext) -> None:
    """Choke: Remove at start of turn."""
    if "Choked" in ctx.player.statuses:
        del ctx.player.statuses["Choked"]


@power_trigger("atStartOfTurn", power="NextTurnBlock")
def next_turn_block_start(ctx: PowerContext) -> None:
    """Next Turn Block: Gain block at start of turn."""
    if ctx.amount > 0:
        ctx.gain_block(ctx.amount)
        del ctx.player.statuses["NextTurnBlock"]


# =============================================================================
# AT_END_OF_TURN_PRE_END_TURN_CARDS Triggers (before discarding)
# =============================================================================

@power_trigger("atEndOfTurnPreEndTurnCards", power="Metallicize")
def metallicize_end(ctx: PowerContext) -> None:
    """Metallicize: Gain block at end of turn."""
    ctx.gain_block(ctx.amount)


@power_trigger("atEndOfTurnPreEndTurnCards", power="Plated Armor")
def plated_armor_end(ctx: PowerContext) -> None:
    """Plated Armor: Gain block at end of turn."""
    ctx.gain_block(ctx.amount)


@power_trigger("atEndOfTurnPreEndTurnCards", power="LikeWater")
def like_water_end(ctx: PowerContext) -> None:
    """Like Water: Gain block if in Calm."""
    if ctx.state.stance == "Calm":
        ctx.gain_block(ctx.amount)


# =============================================================================
# AT_END_OF_TURN Triggers
# =============================================================================

@power_trigger("atEndOfTurn", power="Constricted")
def constricted_end(ctx: PowerContext) -> None:
    """Constricted: Take damage at end of turn."""
    if ctx.owner == ctx.player:
        ctx.player.hp -= ctx.amount
        if ctx.player.hp < 0:
            ctx.player.hp = 0


@power_trigger("atEndOfTurn", power="Combust")
def combust_end(ctx: PowerContext) -> None:
    """Combust: Deal 5 damage to all enemies, lose 1 HP."""
    for enemy in ctx.living_enemies:
        blocked = min(enemy.block, 5)
        enemy.block -= blocked
        enemy.hp -= (5 - blocked)
        if enemy.hp < 0:
            enemy.hp = 0
    ctx.player.hp -= 1


@power_trigger("atEndOfTurn", power="Ritual")
def ritual_end(ctx: PowerContext) -> None:
    """Ritual: Gain Strength at end of turn."""
    ctx.apply_power_to_player("Strength", ctx.amount)


@power_trigger("atEndOfTurn", power="LoseStrength")
def lose_strength_end(ctx: PowerContext) -> None:
    """Lose Strength: Remove temporary strength at end of turn."""
    if ctx.amount > 0:
        current = ctx.player.statuses.get("Strength", 0)
        ctx.player.statuses["Strength"] = current - ctx.amount
        del ctx.player.statuses["LoseStrength"]


@power_trigger("atEndOfTurn", power="LoseDexterity")
def lose_dexterity_end(ctx: PowerContext) -> None:
    """Lose Dexterity: Remove temporary dexterity at end of turn."""
    if ctx.amount > 0:
        current = ctx.player.statuses.get("Dexterity", 0)
        ctx.player.statuses["Dexterity"] = current - ctx.amount
        del ctx.player.statuses["LoseDexterity"]


@power_trigger("atEndOfTurn", power="Intangible")
def intangible_end(ctx: PowerContext) -> None:
    """Intangible: Decrement at end of turn."""
    if ctx.amount > 0:
        ctx.player.statuses["Intangible"] = ctx.amount - 1
        if ctx.player.statuses["Intangible"] <= 0:
            del ctx.player.statuses["Intangible"]


# =============================================================================
# AT_END_OF_ROUND Triggers (after all turns)
# =============================================================================

@power_trigger("atEndOfRound", power="Weakened")
def weak_end_round(ctx: PowerContext) -> None:
    """Weak: Decrement at end of round."""
    if ctx.owner and "Weakened" in ctx.owner.statuses:
        ctx.owner.statuses["Weakened"] -= 1
        if ctx.owner.statuses["Weakened"] <= 0:
            del ctx.owner.statuses["Weakened"]


@power_trigger("atEndOfRound", power="Vulnerable")
def vulnerable_end_round(ctx: PowerContext) -> None:
    """Vulnerable: Decrement at end of round."""
    if ctx.owner and "Vulnerable" in ctx.owner.statuses:
        ctx.owner.statuses["Vulnerable"] -= 1
        if ctx.owner.statuses["Vulnerable"] <= 0:
            del ctx.owner.statuses["Vulnerable"]


@power_trigger("atEndOfRound", power="Frail")
def frail_end_round(ctx: PowerContext) -> None:
    """Frail: Decrement at end of round."""
    if ctx.owner and "Frail" in ctx.owner.statuses:
        ctx.owner.statuses["Frail"] -= 1
        if ctx.owner.statuses["Frail"] <= 0:
            del ctx.owner.statuses["Frail"]


# =============================================================================
# ON_USE_CARD Triggers
# =============================================================================

@power_trigger("onUseCard", power="Vigor")
def vigor_on_use(ctx: PowerContext) -> None:
    """Vigor: Consumed when first attack is played (handled in damage calc)."""
    # Vigor is handled in damage calculation, just mark for removal after attack
    pass


@power_trigger("onUseCard", power="AfterImage")
def after_image_on_use(ctx: PowerContext) -> None:
    """After Image: Gain 1 Block when playing a card."""
    ctx.gain_block(ctx.amount)


@power_trigger("onUseCard", power="Choked")
def choked_on_use(ctx: PowerContext) -> None:
    """Choke: Lose HP when playing a card."""
    ctx.player.hp -= ctx.amount
    if ctx.player.hp < 0:
        ctx.player.hp = 0


@power_trigger("onUseCard", power="Duplication")
def duplication_on_use(ctx: PowerContext) -> None:
    """Duplication: Mark that card should be played again."""
    # Actual duplication is handled by combat engine
    if ctx.amount > 0:
        ctx.player.statuses["Duplication"] = ctx.amount - 1
        if ctx.player.statuses["Duplication"] <= 0:
            del ctx.player.statuses["Duplication"]


# =============================================================================
# ON_EXHAUST Triggers
# =============================================================================

@power_trigger("onExhaust", power="DarkEmbrace")
def dark_embrace_exhaust(ctx: PowerContext) -> None:
    """Dark Embrace: Draw 1 card when exhausting a card."""
    ctx.draw_cards(ctx.amount)


@power_trigger("onExhaust", power="FeelNoPain")
def feel_no_pain_exhaust(ctx: PowerContext) -> None:
    """Feel No Pain: Gain Block when exhausting a card."""
    ctx.gain_block(ctx.amount)


# =============================================================================
# ON_CHANGE_STANCE Triggers (Watcher)
# =============================================================================

@power_trigger("onChangeStance", power="MentalFortress")
def mental_fortress_stance(ctx: PowerContext) -> None:
    """Mental Fortress: Gain Block on stance change."""
    ctx.gain_block(ctx.amount)


@power_trigger("onChangeStance", power="Rushdown")
def rushdown_stance(ctx: PowerContext) -> None:
    """Rushdown: Draw cards when entering Wrath."""
    new_stance = ctx.trigger_data.get("new_stance", "")
    if new_stance == "Wrath":
        ctx.draw_cards(ctx.amount)


# =============================================================================
# ON_SCRY Triggers
# =============================================================================

@power_trigger("onScry", power="Nirvana")
def nirvana_scry(ctx: PowerContext) -> None:
    """Nirvana: Gain Block when Scrying."""
    cards_scried = ctx.trigger_data.get("cards_scried", 1)
    ctx.gain_block(ctx.amount * cards_scried)


# =============================================================================
# MODIFY_BLOCK Triggers
# =============================================================================

@power_trigger("modifyBlock", power="Dexterity")
def dexterity_modify_block(ctx: PowerContext) -> int:
    """Dexterity: Add to block from cards."""
    base_block = ctx.trigger_data.get("value", 0)
    return base_block + ctx.amount


@power_trigger("modifyBlock", power="Frail", priority=10)
def frail_modify_block(ctx: PowerContext) -> int:
    """Frail: Reduce block from cards by 25%."""
    base_block = ctx.trigger_data.get("value", 0)
    # Apply after dexterity
    return int(base_block * 0.75)


# =============================================================================
# AT_DAMAGE_GIVE Triggers
# =============================================================================

@power_trigger("atDamageGive", power="Strength")
def strength_damage_give(ctx: PowerContext) -> int:
    """Strength: Add to damage dealt."""
    base_damage = ctx.trigger_data.get("value", 0)
    return base_damage + ctx.amount


@power_trigger("atDamageGive", power="Vigor")
def vigor_damage_give(ctx: PowerContext) -> int:
    """Vigor: Add to first attack's damage."""
    base_damage = ctx.trigger_data.get("value", 0)
    # Vigor is consumed after first attack
    return base_damage + ctx.amount


@power_trigger("atDamageGive", power="Weakened", priority=99)
def weak_damage_give(ctx: PowerContext) -> int:
    """Weak: Reduce damage dealt by 25%."""
    base_damage = ctx.trigger_data.get("value", 0)
    return int(base_damage * 0.75)


# =============================================================================
# AT_DAMAGE_RECEIVE Triggers
# =============================================================================

@power_trigger("atDamageReceive", power="Vulnerable")
def vulnerable_damage_receive(ctx: PowerContext) -> int:
    """Vulnerable: Take 50% more damage."""
    base_damage = ctx.trigger_data.get("value", 0)
    return int(base_damage * 1.5)


@power_trigger("atDamageFinalReceive", power="Intangible", priority=1)
def intangible_damage_final(ctx: PowerContext) -> int:
    """Intangible: Reduce all damage to 1."""
    return 1


# =============================================================================
# ON_ATTACKED_TO_CHANGE_DAMAGE Triggers
# =============================================================================

@power_trigger("onAttackedToChangeDamage", power="Buffer")
def buffer_change_damage(ctx: PowerContext) -> int:
    """Buffer: Prevent damage and decrement."""
    damage = ctx.trigger_data.get("value", 0)
    if damage > 0 and ctx.amount > 0:
        ctx.player.statuses["Buffer"] = ctx.amount - 1
        if ctx.player.statuses["Buffer"] <= 0:
            del ctx.player.statuses["Buffer"]
        return 0  # Prevent damage
    return damage


# =============================================================================
# WAS_HP_LOST Triggers
# =============================================================================

@power_trigger("wasHPLost", power="Rupture")
def rupture_hp_lost(ctx: PowerContext) -> None:
    """Rupture: Gain Strength when losing HP from cards."""
    # Only triggers from card HP loss, not enemy attacks
    source = ctx.trigger_data.get("source", "")
    if source == "card":
        ctx.apply_power_to_player("Strength", ctx.amount)


@power_trigger("wasHPLost", power="Plated Armor")
def plated_armor_hp_lost(ctx: PowerContext) -> None:
    """Plated Armor: Lose 1 stack when taking unblocked damage."""
    if ctx.trigger_data.get("unblocked", False):
        current = ctx.player.statuses.get("Plated Armor", 0)
        if current > 1:
            ctx.player.statuses["Plated Armor"] = current - 1
        else:
            del ctx.player.statuses["Plated Armor"]


# =============================================================================
# ON_ATTACK Triggers
# =============================================================================

@power_trigger("onAttack", power="Envenom")
def envenom_on_attack(ctx: PowerContext) -> None:
    """Envenom: Apply Poison on unblocked attack damage."""
    target = ctx.trigger_data.get("target")
    if target and ctx.trigger_data.get("unblocked_damage", 0) > 0:
        ctx.apply_power(target, "Poison", ctx.amount)


@power_trigger("onAttack", power="Thorns")
def thorns_on_attacked(ctx: PowerContext) -> None:
    """Thorns: Deal damage back when attacked."""
    attacker = ctx.trigger_data.get("attacker")
    if attacker and hasattr(attacker, 'hp'):
        attacker.hp -= ctx.amount
        if attacker.hp < 0:
            attacker.hp = 0


# =============================================================================
# ON_GAIN_BLOCK Triggers
# =============================================================================

@power_trigger("onGainBlock", power="Juggernaut")
def juggernaut_gain_block(ctx: PowerContext) -> None:
    """Juggernaut: Deal damage to random enemy when gaining block."""
    import random
    if ctx.living_enemies:
        target = random.choice(ctx.living_enemies)
        blocked = min(target.block, ctx.amount)
        target.block -= blocked
        target.hp -= (ctx.amount - blocked)
        if target.hp < 0:
            target.hp = 0


@power_trigger("onGainBlock", power="WaveOfTheHand")
def wave_of_hand_gain_block(ctx: PowerContext) -> None:
    """Wave of the Hand: Apply Weak to all enemies when gaining block."""
    ctx.apply_power_to_all_enemies("Weakened", ctx.amount)


# =============================================================================
# ENERGY_RECHARGE Triggers
# =============================================================================

@power_trigger("onEnergyRecharge", power="DevaForm")
def deva_form_energy(ctx: PowerContext) -> None:
    """Deva Form: Gain energy at start of turn (increases each turn)."""
    ctx.gain_energy(ctx.amount)
    # Increment for next turn
    ctx.player.statuses["DevaForm"] = ctx.amount + 1


@power_trigger("onEnergyRecharge", power="Energized")
def energized_energy(ctx: PowerContext) -> None:
    """Energized: Gain energy next turn, then remove."""
    ctx.gain_energy(ctx.amount)
    del ctx.player.statuses["Energized"]
