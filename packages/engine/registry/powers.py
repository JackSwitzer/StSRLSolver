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
        damage = ctx.amount

        # For player, apply Intangible and Tungsten Rod
        if ctx.owner == ctx.player:
            # Intangible caps HP loss to 1
            if ctx.player.statuses.get("Intangible", 0) > 0 and damage > 1:
                damage = 1
            # Tungsten Rod reduces HP loss by 1
            if ctx.state.has_relic("Tungsten Rod") and damage > 0:
                damage = max(0, damage - 1)

        # Deal HP_LOSS damage (ignores block)
        ctx.owner.hp -= damage
        if ctx.owner.hp < 0:
            ctx.owner.hp = 0

        # Track damage for player
        if ctx.owner == ctx.player:
            ctx.state.total_damage_taken += damage
        else:
            ctx.state.total_damage_dealt += damage

        # Decrement poison
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


@power_trigger("atStartOfTurn", power="Foresight")
def foresight_start(ctx: PowerContext) -> None:
    """Foresight: Scry at start of turn."""
    # Track scry count for the turn - combat engine handles actual scry UI
    ctx.state.pending_scry = getattr(ctx.state, 'pending_scry', 0) + ctx.amount


@power_trigger("atStartOfTurn", power="InfiniteBlades")
def infinite_blades_start(ctx: PowerContext) -> None:
    """Infinite Blades: Add Shiv(s) to hand at start of turn."""
    for _ in range(ctx.amount):
        ctx.add_card_to_hand("Shiv")


@power_trigger("atStartOfTurn", power="BattleHymn")
def battle_hymn_start(ctx: PowerContext) -> None:
    """Battle Hymn: Add Smite(s) to hand at start of turn."""
    for _ in range(ctx.amount):
        ctx.add_card_to_hand("Smite")


@power_trigger("atStartOfTurn", power="FlameBarrier")
def flame_barrier_remove(ctx: PowerContext) -> None:
    """Flame Barrier: Remove at start of turn."""
    if "FlameBarrier" in ctx.player.statuses:
        del ctx.player.statuses["FlameBarrier"]


@power_trigger("atStartOfTurn", power="Mayhem")
def mayhem_start(ctx: PowerContext) -> None:
    """Mayhem: Play top card(s) of draw pile at start of turn."""
    for _ in range(ctx.amount):
        if ctx.state.draw_pile:
            card = ctx.state.draw_pile.pop()
            # Store cards to auto-play - combat engine handles actual play
            if not hasattr(ctx.state, 'cards_to_auto_play'):
                ctx.state.cards_to_auto_play = []
            ctx.state.cards_to_auto_play.append(card)


@power_trigger("atStartOfTurn", power="Magnetism")
def magnetism_start(ctx: PowerContext) -> None:
    """Magnetism: Add random colorless card to hand at start of turn."""
    import random
    from ..content.cards import ALL_CARDS, CardColor
    colorless_cards = [
        cid for cid, c in ALL_CARDS.items()
        if hasattr(c, 'color') and c.color == CardColor.COLORLESS
    ]
    for _ in range(ctx.amount):
        if colorless_cards and len(ctx.state.hand) < 10:
            ctx.state.hand.append(random.choice(colorless_cards))


@power_trigger("atStartOfTurn", power="CreativeAI")
def creative_ai_start(ctx: PowerContext) -> None:
    """Creative AI: Add random Power card to hand at start of turn."""
    import random
    from ..content.cards import ALL_CARDS, CardType
    power_cards = [
        cid for cid, c in ALL_CARDS.items()
        if c.card_type == CardType.POWER
    ]
    for _ in range(ctx.amount):
        if power_cards and len(ctx.state.hand) < 10:
            ctx.state.hand.append(random.choice(power_cards))


@power_trigger("atStartOfTurn", power="Loop")
def loop_start(ctx: PowerContext) -> None:
    """Loop: Trigger rightmost orb's passive at start of turn.

    Java: LoopPower.atStartOfTurn() calls both onStartOfTurn() AND onEndOfTurn()
    on orbs.get(0), which is the rightmost orb. This triggers the passive effect.
    """
    from ..effects.orbs import get_orb_manager
    manager = get_orb_manager(ctx.state)
    if manager.orbs:
        # Trigger rightmost orb's passive ctx.amount times
        # Note: rightmost orb is at index -1 (end of list)
        rightmost_orb = manager.orbs[-1]
        for _ in range(ctx.amount):
            manager._execute_passive(rightmost_orb, ctx.state, manager.focus)


# =============================================================================
# AT_START_OF_TURN_POST_DRAW Triggers (after draw)
# =============================================================================

@power_trigger("atStartOfTurnPostDraw", power="DemonForm")
def demon_form_start(ctx: PowerContext) -> None:
    """Demon Form: Gain Strength at start of turn (after draw)."""
    ctx.apply_power_to_player("Strength", ctx.amount)


@power_trigger("atStartOfTurnPostDraw", power="Brutality")
def brutality_start(ctx: PowerContext) -> None:
    """Brutality: Draw cards and lose HP at start of turn (after draw)."""
    ctx.draw_cards(ctx.amount)
    ctx.player.hp -= ctx.amount
    if ctx.player.hp < 0:
        ctx.player.hp = 0


@power_trigger("atStartOfTurnPostDraw", power="NoxiousFumes")
def noxious_fumes_start(ctx: PowerContext) -> None:
    """Noxious Fumes: Apply Poison to all enemies at start of turn."""
    ctx.apply_power_to_all_enemies("Poison", ctx.amount)


@power_trigger("atStartOfTurnPostDraw", power="Devotion")
def devotion_start(ctx: PowerContext) -> None:
    """Devotion: Gain Mantra at start of turn."""
    current_mantra = ctx.player.statuses.get("Mantra", 0)
    new_mantra = current_mantra + ctx.amount
    if new_mantra >= 10:
        # Enter Divinity
        ctx.state.stance = "Divinity"
        ctx.player.statuses["Mantra"] = new_mantra - 10
        if ctx.player.statuses["Mantra"] <= 0:
            del ctx.player.statuses["Mantra"]
    else:
        ctx.player.statuses["Mantra"] = new_mantra


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


@power_trigger("atEndOfTurn", power="Study")
def study_end(ctx: PowerContext) -> None:
    """Study: Shuffle Insight into draw pile at end of turn."""
    import random
    for _ in range(ctx.amount):
        ctx.state.draw_pile.append("Insight")
    # Shuffle to random position
    random.shuffle(ctx.state.draw_pile)


@power_trigger("atEndOfTurn", power="WraithFormPower")
def wraith_form_end(ctx: PowerContext) -> None:
    """Wraith Form: Lose Dexterity at end of turn."""
    current_dex = ctx.player.statuses.get("Dexterity", 0)
    ctx.player.statuses["Dexterity"] = current_dex - ctx.amount
    # Remove at 0
    if ctx.player.statuses["Dexterity"] == 0:
        del ctx.player.statuses["Dexterity"]


@power_trigger("atEndOfTurn", power="Omega")
def omega_end(ctx: PowerContext) -> None:
    """Omega: Deal damage to all enemies at end of turn."""
    for enemy in ctx.living_enemies:
        # THORNS type damage
        blocked = min(enemy.block, ctx.amount)
        enemy.block -= blocked
        enemy.hp -= (ctx.amount - blocked)
        if enemy.hp < 0:
            enemy.hp = 0


@power_trigger("atEndOfTurn", power="Bias")
def bias_end(ctx: PowerContext) -> None:
    """Bias: Lose Focus at start of turn (processed at end of prev turn)."""
    current_focus = ctx.player.statuses.get("Focus", 0)
    ctx.player.statuses["Focus"] = current_focus - ctx.amount
    if ctx.player.statuses["Focus"] == 0:
        del ctx.player.statuses["Focus"]


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
    card = ctx.card
    if card is None:
        return
    try:
        from ..content.cards import CardType, get_card
        if isinstance(card, str):
            card_obj = get_card(card)
        else:
            card_obj = card
        if card_obj.card_type == CardType.ATTACK:
            if ctx.owner and "Vigor" in ctx.owner.statuses:
                del ctx.owner.statuses["Vigor"]
    except Exception:
        # If card lookup fails, ignore
        return


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


@power_trigger("onUseCard", power="Panache")
def panache_on_use(ctx: PowerContext) -> None:
    """Panache: Deal 10 damage to all enemies after playing 5 cards."""
    counter = ctx.player.statuses.get("PanacheCounter", 0) + 1
    if counter >= 5:
        # Deal 10 damage to all enemies (THORNS type, bypasses Strength)
        for enemy in ctx.living_enemies:
            blocked = min(enemy.block, 10)
            enemy.block -= blocked
            enemy.hp -= (10 - blocked)
            if enemy.hp < 0:
                enemy.hp = 0
        counter = 0
    ctx.player.statuses["PanacheCounter"] = counter


@power_trigger("onUseCard", power="ThousandCuts")
def thousand_cuts_on_use(ctx: PowerContext) -> None:
    """Thousand Cuts: Deal damage to all enemies when playing any card."""
    for enemy in ctx.living_enemies:
        # THORNS type damage
        blocked = min(enemy.block, ctx.amount)
        enemy.block -= blocked
        enemy.hp -= (ctx.amount - blocked)
        if enemy.hp < 0:
            enemy.hp = 0


@power_trigger("onUseCard", power="Heatsink")
def heatsink_on_use(ctx: PowerContext) -> None:
    """Heatsink: Draw cards when playing a Power card."""
    from ..content.cards import ALL_CARDS, CardType
    card_id = ctx.trigger_data.get("card_id", "")
    if card_id in ALL_CARDS and ALL_CARDS[card_id].card_type == CardType.POWER:
        ctx.draw_cards(ctx.amount)


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
# ON_CARD_DRAW Triggers
# =============================================================================

@power_trigger("onCardDraw", power="Evolve")
def evolve_draw(ctx: PowerContext) -> None:
    """Evolve: Draw card(s) when Status drawn."""
    card_id = ctx.trigger_data.get("card_id", "")
    from ..content.cards import ALL_CARDS, CardType
    if card_id in ALL_CARDS and ALL_CARDS[card_id].card_type == CardType.STATUS:
        ctx.draw_cards(ctx.amount)


@power_trigger("onCardDraw", power="FireBreathing")
def fire_breathing_draw(ctx: PowerContext) -> None:
    """Fire Breathing: Deal damage to all enemies when drawing Status/Curse."""
    card_id = ctx.trigger_data.get("card_id", "")
    from ..content.cards import ALL_CARDS, CardType
    if card_id in ALL_CARDS:
        card_type = ALL_CARDS[card_id].card_type
        if card_type in (CardType.STATUS, CardType.CURSE):
            for enemy in ctx.living_enemies:
                blocked = min(enemy.block, ctx.amount)
                enemy.block -= blocked
                enemy.hp -= (ctx.amount - blocked)
                if enemy.hp < 0:
                    enemy.hp = 0


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
# ON_ATTACKED Triggers (when player is attacked)
# =============================================================================

@power_trigger("onAttacked", power="FlameBarrier")
def flame_barrier_attacked(ctx: PowerContext) -> None:
    """Flame Barrier: Deal damage back when attacked."""
    attacker = ctx.trigger_data.get("attacker")
    if attacker and hasattr(attacker, 'hp'):
        # Flame Barrier deals THORNS damage (bypasses block)
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
# ON_APPLY_POWER Triggers
# =============================================================================

@power_trigger("onApplyPower", power="SadisticNature")
def sadistic_nature_apply(ctx: PowerContext) -> None:
    """Sadistic Nature: Deal damage when applying a debuff to an enemy."""
    power_id = ctx.trigger_data.get("power_id")
    target = ctx.trigger_data.get("target")
    # Debuffs that trigger Sadistic Nature
    debuffs = {"Weakened", "Vulnerable", "Frail", "Poison", "Slow", "Choked"}
    if power_id in debuffs and target and hasattr(target, 'hp') and target != ctx.player:
        # Deal THORNS damage
        blocked = min(target.block, ctx.amount)
        target.block -= blocked
        target.hp -= (ctx.amount - blocked)
        if target.hp < 0:
            target.hp = 0


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
