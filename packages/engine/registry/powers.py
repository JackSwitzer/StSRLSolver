"""
Power Trigger Implementations.

This module contains power trigger handlers using the registry pattern.
Powers are buffs/debuffs that trigger at various points during combat.

Organized by trigger hook for easier maintenance.
"""

from __future__ import annotations

from . import power_trigger, PowerContext


def _owner_runtime_key(ctx: PowerContext) -> str:
    """Stable owner key for per-power runtime state."""
    if ctx.owner is None:
        return "none"
    if ctx.owner is ctx.player:
        return "player"
    for idx, enemy in enumerate(ctx.state.enemies):
        if enemy is ctx.owner:
            return f"enemy:{idx}:{enemy.id}"
    return f"owner:{id(ctx.owner)}"


def _runtime_counter_key(ctx: PowerContext, token: str) -> str:
    return f"__power_runtime__:{token}:{_owner_runtime_key(ctx)}"


def _ensure_runtime_base(ctx: PowerContext, token: str) -> tuple[str, int]:
    """Ensure a persistent runtime base value exists for a power owner."""
    key = _runtime_counter_key(ctx, token)
    base = int(ctx.state.relic_counters.get(key, 0))
    if base <= 0:
        base = max(0, int(ctx.amount))
        ctx.state.relic_counters[key] = base
    return key, base


def _sync_owner_power_amount(
    ctx: PowerContext, power_id: str, amount: int, *, keep_zero: bool = False
) -> None:
    """Set/remove a power amount while respecting alias/canonical key variants."""
    if ctx.owner is None:
        return

    from ..content.powers import resolve_power_id, normalize_power_id

    canonical = resolve_power_id(power_id)
    token = normalize_power_id(canonical)

    matched_keys = []
    for existing_key in list(ctx.owner.statuses.keys()):
        existing_canonical = resolve_power_id(existing_key)
        if (
            existing_canonical == canonical
            or normalize_power_id(existing_canonical) == token
        ):
            matched_keys.append(existing_key)

    if amount < 0:
        amount = 0

    if amount == 0 and not keep_zero:
        for existing_key in matched_keys:
            ctx.owner.statuses.pop(existing_key, None)
        return

    target_key = matched_keys[0] if matched_keys else canonical
    ctx.owner.statuses[target_key] = int(amount)
    for existing_key in matched_keys[1:]:
        ctx.owner.statuses.pop(existing_key, None)


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
    """Foresight: Scry N at start of turn.

    Uses heuristic mode because the agent cannot take an intermediate action
    during start-of-turn triggers (auto-discards curses/statuses, keeps rest).
    """
    engine = getattr(ctx.state, '_combat_engine_ref', None)
    if engine:
        engine._scry(ctx.amount, heuristic=True)
    else:
        # Fallback: set pending_scry for external handling
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
    from ..content.cards import ALL_CARDS, CardColor
    colorless_cards = [
        cid for cid, c in ALL_CARDS.items()
        if hasattr(c, 'color') and c.color == CardColor.COLORLESS
    ]
    for _ in range(ctx.amount):
        if colorless_cards and len(ctx.state.hand) < 10:
            ctx.state.hand.append(ctx.random_choice(colorless_cards))


@power_trigger("atStartOfTurn", power="CreativeAI")
def creative_ai_start(ctx: PowerContext) -> None:
    """Creative AI: Add random Power card to hand at start of turn."""
    from ..content.cards import ALL_CARDS, CardType
    power_cards = [
        cid for cid, c in ALL_CARDS.items()
        if c.card_type == CardType.POWER
    ]
    for _ in range(ctx.amount):
        if power_cards and len(ctx.state.hand) < 10:
            ctx.state.hand.append(ctx.random_choice(power_cards))


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


@power_trigger("atStartOfTurn", power="Bias")
def bias_start(ctx: PowerContext) -> None:
    """Bias: Lose Focus at start of turn (Biased Cognition timing)."""
    current_focus = ctx.player.statuses.get("Focus", 0)
    ctx.player.statuses["Focus"] = current_focus - ctx.amount
    if ctx.player.statuses["Focus"] == 0:
        del ctx.player.statuses["Focus"]


@power_trigger("atStartOfTurn", power="DisciplinePower")
def discipline_power_start(ctx: PowerContext) -> None:
    """DisciplinePower: Draw saved amount at turn start, then reset to sentinel."""
    if ctx.owner is None or ctx.amount == -1:
        return
    if ctx.owner == ctx.player and ctx.amount > 0:
        ctx.draw_cards(ctx.amount)
    ctx.owner.statuses["DisciplinePower"] = -1


@power_trigger("atStartOfTurn", power="Flight", priority=50)
def flight_start(ctx: PowerContext) -> None:
    """Flight: reset to stored stack count each turn."""
    if ctx.owner is None:
        return
    key, base = _ensure_runtime_base(ctx, "flight_base")
    # If power was stacked by another effect, use the higher amount as new baseline.
    if ctx.amount > base:
        base = int(ctx.amount)
        ctx.state.relic_counters[key] = base
    _sync_owner_power_amount(ctx, "Flight", base)


@power_trigger("atStartOfTurn", power="Invincible", priority=99)
def invincible_start(ctx: PowerContext) -> None:
    """Invincible: reset remaining turn cap at start of turn."""
    if ctx.owner is None:
        return
    key, base = _ensure_runtime_base(ctx, "invincible_max")
    if ctx.amount > base:
        base = int(ctx.amount)
        ctx.state.relic_counters[key] = base
    _sync_owner_power_amount(ctx, "Invincible", base, keep_zero=True)


@power_trigger("atStartOfTurn", power="Echo Form")
def echo_form_start(ctx: PowerContext) -> None:
    """Echo Form: reset per-turn doubled-card counter."""
    ctx.state.relic_counters[_runtime_counter_key(ctx, "echo_form_doubled")] = 0


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
    """Devotion: Gain Mantra at start of turn.

    Uses engine._add_mantra to ensure Divinity transition fires
    onChangeStance hooks (MentalFortress, Rushdown, Violet Lotus).
    """
    engine = getattr(ctx.state, '_combat_engine_ref', None)
    if engine:
        engine._add_mantra(ctx.amount)
    else:
        # Fallback for tests without engine: manipulate state.mantra directly
        ctx.state.mantra += ctx.amount
        if ctx.state.mantra >= 10:
            ctx.state.mantra -= 10
            ctx.state.stance = "Divinity"


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


@power_trigger("atEndOfTurn", power="DisciplinePower")
def discipline_power_end(ctx: PowerContext) -> None:
    """DisciplinePower: Save current energy at end of turn if greater than 0."""
    if ctx.owner == ctx.player and ctx.state.energy > 0:
        ctx.owner.statuses["DisciplinePower"] = ctx.state.energy


@power_trigger("atEndOfTurn", power="Study")
def study_end(ctx: PowerContext) -> None:
    """Study: Shuffle Insight into draw pile at end of turn."""
    for _ in range(ctx.amount):
        ctx.state.draw_pile.append("Insight")
    # Shuffle to random position
    ctx.shuffle_in_place(ctx.state.draw_pile)


@power_trigger("atEndOfTurn", power="WraithFormPower")
def wraith_form_end(ctx: PowerContext) -> None:
    """Wraith Form: Lose Dexterity at end of turn.

    Uses apply_power_to_player with negative amount to respect Artifact.
    In Java, this uses ApplyPowerAction which Artifact can block.
    """
    # Apply negative dexterity - this respects Artifact
    ctx.apply_power_to_player("Dexterity", -ctx.amount)


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


@power_trigger("atEndOfTurn", power="Malleable")
def malleable_end_turn(ctx: PowerContext) -> None:
    """Malleable: monsters reset to base amount at end of turn."""
    if ctx.owner is None or ctx.owner is ctx.player:
        return
    _, base = _ensure_runtime_base(ctx, "malleable_base")
    _sync_owner_power_amount(ctx, "Malleable", base)


@power_trigger("atEndOfTurn", power="Equilibrium")
def equilibrium_end(ctx: PowerContext) -> None:
    """Equilibrium: keep retain-hand marker active while power has stacks."""
    if ctx.owner is not ctx.player or ctx.amount <= 0:
        return
    ctx.player.statuses["RetainHand"] = 1


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


@power_trigger("atEndOfRound", power="Slow")
def slow_end_round(ctx: PowerContext) -> None:
    """Slow: Reset stacks at end of round (power persists)."""
    if ctx.owner and "Slow" in ctx.owner.statuses:
        ctx.owner.statuses["Slow"] = 0


@power_trigger("atEndOfRound", power="Malleable")
def malleable_end_round(ctx: PowerContext) -> None:
    """Malleable: player resets to base amount at end of round."""
    if ctx.owner is not ctx.player:
        return
    _, base = _ensure_runtime_base(ctx, "malleable_base")
    _sync_owner_power_amount(ctx, "Malleable", base)


@power_trigger("atEndOfRound", power="Equilibrium")
def equilibrium_end_round(ctx: PowerContext) -> None:
    """Equilibrium: reduce at end of round and clear retain marker when removed."""
    if ctx.owner is not ctx.player:
        return
    if ctx.amount > 1:
        _sync_owner_power_amount(ctx, "Equilibrium", ctx.amount - 1)
        ctx.player.statuses["RetainHand"] = 1
    else:
        _sync_owner_power_amount(ctx, "Equilibrium", 0)
        ctx.player.statuses.pop("RetainHand", None)


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


@power_trigger("onUseCard", power="Heatsink")
def heatsink_on_use(ctx: PowerContext) -> None:
    """Heatsink: Draw cards when playing a Power card."""
    from ..content.cards import ALL_CARDS, CardType
    card_id = ctx.trigger_data.get("card_id", "")
    if card_id in ALL_CARDS and ALL_CARDS[card_id].card_type == CardType.POWER:
        ctx.draw_cards(ctx.amount)


@power_trigger("onUseCard", power="Pen Nib")
def pen_nib_on_use(ctx: PowerContext) -> None:
    """Pen Nib: remove after an Attack is used."""
    card = ctx.trigger_data.get("card")
    if card is None:
        return
    from ..content.cards import CardType
    if getattr(card, "card_type", None) == CardType.ATTACK:
        _sync_owner_power_amount(ctx, "Pen Nib", 0)


@power_trigger("onUseCard", power="Echo Form")
def echo_form_on_use(ctx: PowerContext) -> None:
    """Echo Form: mark the first N cards each turn for replay."""
    if ctx.owner is not ctx.player or ctx.amount <= 0:
        return
    if ctx.trigger_data.get("is_echo_copy"):
        return
    card = ctx.trigger_data.get("card")
    if card is None:
        return
    if getattr(card, "purge_on_use", False):
        return

    key = _runtime_counter_key(ctx, "echo_form_doubled")
    cards_doubled = int(ctx.state.relic_counters.get(key, 0))
    cards_played = int(getattr(ctx.state, "cards_played_this_turn", 0))
    if cards_played - cards_doubled <= int(ctx.amount):
        ctx.state.relic_counters[key] = cards_doubled + 1
        repeats = int(ctx.trigger_data.get("repeat_play_count", 0))
        ctx.trigger_data["repeat_play_count"] = repeats + 1


# =============================================================================
# ON_AFTER_USE_CARD Triggers
# =============================================================================

@power_trigger("onAfterUseCard", power="BeatOfDeath")
def beat_of_death_after_use(ctx: PowerContext) -> None:
    """Beat of Death: Deal THORNS damage to player after each card."""
    if ctx.amount <= 0:
        return
    blocked = min(ctx.player.block, ctx.amount)
    hp_damage = ctx.amount - blocked
    ctx.player.block -= blocked
    ctx.player.hp -= hp_damage
    if ctx.player.hp < 0:
        ctx.player.hp = 0
    ctx.state.total_damage_taken += hp_damage


@power_trigger("onAfterUseCard", power="Slow")
def slow_after_use(ctx: PowerContext) -> None:
    """Slow: Increase stacks by 1 whenever a card is played."""
    if ctx.owner is None:
        return
    ctx.owner.statuses["Slow"] = ctx.owner.statuses.get("Slow", 0) + 1


@power_trigger("onAfterUseCard", power="Time Warp")
def time_warp_after_use(ctx: PowerContext) -> None:
    """Time Warp: Count cards; at 12, end turn and all enemies gain Strength."""
    if ctx.owner is None:
        return
    counter = ctx.owner.statuses.get("Time Warp", 0) + 1
    if counter >= 12:
        counter = 0
        for enemy in ctx.living_enemies:
            enemy.statuses["Strength"] = enemy.statuses.get("Strength", 0) + 2
        ctx.trigger_data["force_end_turn"] = True
    ctx.owner.statuses["Time Warp"] = counter


# =============================================================================
# ON_AFTER_CARD_PLAYED Triggers
# =============================================================================

@power_trigger("onAfterCardPlayed", power="ThousandCuts")
def thousand_cuts_after_played(ctx: PowerContext) -> None:
    """Thousand Cuts: Deal THORNS damage to all enemies after a card is played."""
    for enemy in ctx.living_enemies:
        blocked = min(enemy.block, ctx.amount)
        enemy.block -= blocked
        enemy.hp -= (ctx.amount - blocked)
        if enemy.hp < 0:
            enemy.hp = 0


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


@power_trigger("atDamageGive", power="Pen Nib", priority=6)
def pen_nib_damage_give(ctx: PowerContext) -> int:
    """Pen Nib: double NORMAL damage."""
    base_damage = ctx.trigger_data.get("value", 0)
    if ctx.trigger_data.get("damage_type", "NORMAL") != "NORMAL":
        return base_damage
    return base_damage * 2.0


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


@power_trigger("atDamageReceive", power="Slow")
def slow_damage_receive(ctx: PowerContext) -> int:
    """Slow: Increase NORMAL damage taken by 10% per stack."""
    base_damage = ctx.trigger_data.get("value", 0)
    if ctx.damage_type != "NORMAL" or ctx.amount <= 0:
        return base_damage
    return int(base_damage * (1 + (ctx.amount * 0.1)))


@power_trigger("atDamageFinalReceive", power="Intangible", priority=1)
@power_trigger("atDamageFinalReceive", power="IntangiblePlayer", priority=1)
def intangible_damage_final(ctx: PowerContext) -> int:
    """Intangible: Reduce all damage to 1."""
    return 1


@power_trigger("atDamageFinalReceive", power="Flight", priority=50)
def flight_damage_final(ctx: PowerContext) -> float:
    """Flight: halve incoming non-HP_LOSS/non-THORNS damage."""
    base_damage = ctx.trigger_data.get("value", 0)
    damage_type = ctx.trigger_data.get("damage_type", "NORMAL")
    if damage_type in ("HP_LOSS", "THORNS"):
        return base_damage
    return base_damage / 2.0


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


@power_trigger("onAttackedToChangeDamage", power="Invincible", priority=99)
def invincible_change_damage(ctx: PowerContext) -> int:
    """Invincible: cap incoming hit damage to remaining per-turn amount."""
    incoming = int(ctx.trigger_data.get("value", 0))
    if incoming <= 0:
        return 0

    key, base = _ensure_runtime_base(ctx, "invincible_max")
    if ctx.amount > base:
        base = int(ctx.amount)
        ctx.state.relic_counters[key] = base

    capped = min(incoming, int(ctx.amount))
    remaining = max(0, int(ctx.amount) - capped)
    _sync_owner_power_amount(ctx, "Invincible", remaining, keep_zero=True)
    return capped


# =============================================================================
# WAS_HP_LOST Triggers
# =============================================================================

@power_trigger("wasHPLost", power="Rupture")
def rupture_hp_lost(ctx: PowerContext) -> None:
    """Rupture: Gain Strength when losing HP from self-damage (Java: info.owner == this.owner)."""
    # Triggers from ANY self-damage (cards, powers, effects) - not just cards
    # Java checks: damageAmount > 0 && info.owner == this.owner
    damage_amount = ctx.trigger_data.get("damage", 0)
    is_self_damage = ctx.trigger_data.get("is_self_damage", False)
    if damage_amount > 0 and is_self_damage:
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
@power_trigger("onAttacked", power="Thorns")
def thorns_on_attacked(ctx: PowerContext) -> None:
    """Thorns: Deal damage back when attacked."""
    attacker = ctx.trigger_data.get("attacker")
    if attacker and hasattr(attacker, 'hp') and ctx.trigger_data.get("damage", 0) > 0:
        blocked = min(getattr(attacker, "block", 0), ctx.amount)
        if hasattr(attacker, "block"):
            attacker.block -= blocked
        attacker.hp -= (ctx.amount - blocked)
        if attacker.hp < 0:
            attacker.hp = 0


# =============================================================================
# ON_ATTACKED Triggers (when player is attacked)
# =============================================================================

@power_trigger("onAttacked", power="Flight", priority=50)
def flight_on_attacked(ctx: PowerContext) -> None:
    """Flight: lose one stack when hit by eligible damage and still alive."""
    if ctx.owner is None:
        return
    _ensure_runtime_base(ctx, "flight_base")
    attacker = ctx.trigger_data.get("attacker")
    damage_type = ctx.trigger_data.get("damage_type", "NORMAL")
    unblocked = int(ctx.trigger_data.get("unblocked_damage", ctx.trigger_data.get("damage", 0)))
    if (
        attacker is None
        or unblocked <= 0
        or damage_type in ("HP_LOSS", "THORNS")
        or ctx.owner.hp <= 0
    ):
        return

    next_amount = max(0, int(ctx.amount) - 1)
    _sync_owner_power_amount(ctx, "Flight", next_amount)
    if next_amount == 0:
        ctx.state.relic_counters.pop(_runtime_counter_key(ctx, "flight_base"), None)


@power_trigger("onAttacked", power="Malleable")
def malleable_on_attacked(ctx: PowerContext) -> None:
    """Malleable: gain block, then increase amount by 1 after eligible hits."""
    if ctx.owner is None:
        return
    attacker = ctx.trigger_data.get("attacker")
    damage_type = ctx.trigger_data.get("damage_type", "NORMAL")
    unblocked = int(ctx.trigger_data.get("unblocked_damage", ctx.trigger_data.get("damage", 0)))
    if (
        attacker is None
        or unblocked <= 0
        or damage_type != "NORMAL"
        or ctx.owner.hp <= 0
    ):
        return

    _ensure_runtime_base(ctx, "malleable_base")
    gain = max(0, int(ctx.amount))
    if ctx.owner is ctx.player:
        ctx.gain_block(gain)
    else:
        ctx.owner.block += gain
    _sync_owner_power_amount(ctx, "Malleable", int(ctx.amount) + 1)


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
    if ctx.living_enemies:
        target = ctx.random_choice(ctx.living_enemies)
        blocked = min(target.block, ctx.amount)
        target.block -= blocked
        target.hp -= (ctx.amount - blocked)
        if target.hp < 0:
            target.hp = 0


@power_trigger("onGainBlock", power="WaveOfTheHand")
def wave_of_hand_gain_block(ctx: PowerContext) -> None:
    """Wave of the Hand: Apply Weak to all enemies when gaining block."""
    ctx.apply_power_to_all_enemies("Weakened", ctx.amount)


@power_trigger("atEndOfRound", power="WaveOfTheHandPower")
def wave_of_the_hand_end_of_round(ctx: PowerContext) -> None:
    """Wave of the Hand: expires at end of round."""
    if "WaveOfTheHandPower" in ctx.player.statuses:
        del ctx.player.statuses["WaveOfTheHandPower"]
    if "WaveOfTheHand" in ctx.player.statuses:
        del ctx.player.statuses["WaveOfTheHand"]


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


@power_trigger("onEnergyRecharge", power="EnergizedBlue")
def energized_blue_energy(ctx: PowerContext) -> None:
    """EnergizedBlue (Defect): Identical to Energized. Gain energy, then remove.

    Java: EnergizedBluePower.onEnergyRecharge -- same mechanic as EnergizedPower.
    """
    ctx.gain_energy(ctx.amount)
    del ctx.player.statuses["EnergizedBlue"]


@power_trigger("atStartOfTurn", power="Berserk")
def berserk_energy(ctx: PowerContext) -> None:
    """Berserk: Gain 1 energy at start of each turn (Java: BerserkPower.atStartOfTurn)."""
    ctx.gain_energy(ctx.amount)


# =============================================================================
# ADDITIONAL IRONCLAD POWER TRIGGERS
# =============================================================================

@power_trigger("onCardDraw", power="Corruption")
def corruption_on_draw(ctx: PowerContext) -> None:
    """Corruption: Skills cost 0 when drawn (Java: card.setCostForTurn(-9))."""
    from ..content.cards import ALL_CARDS, CardType
    card_id = ctx.trigger_data.get("card_id", "")
    base_id = card_id.rstrip("+")
    if base_id in ALL_CARDS and ALL_CARDS[base_id].card_type == CardType.SKILL:
        # Mark this card as cost 0 for this turn
        # The combat engine should check for Corruption and set skill cost to 0
        ctx.trigger_data["set_cost_to_zero"] = True


@power_trigger("onUseCard", power="Corruption")
def corruption_on_use(ctx: PowerContext) -> None:
    """Corruption: Exhaust Skills when played (Java: action.exhaustCard = true)."""
    from ..content.cards import ALL_CARDS, CardType
    card_id = ctx.trigger_data.get("card_id", "")
    base_id = card_id.rstrip("+")
    if base_id in ALL_CARDS and ALL_CARDS[base_id].card_type == CardType.SKILL:
        # Mark this card to be exhausted after playing
        ctx.trigger_data["exhaust_card"] = True


@power_trigger("atStartOfTurnPostDraw", power="Barricade")
def barricade_start(ctx: PowerContext) -> None:
    """Barricade: Block is not removed at start of turn."""
    # This is handled by preventing block reset in combat engine
    pass


@power_trigger("atStartOfTurnPostDraw", power="Rage")
def rage_start(ctx: PowerContext) -> None:
    """Rage: Reset at start of turn (lasts this turn only)."""
    # Rage is applied fresh each turn, previous turn's Rage is removed
    if "Rage" in ctx.player.statuses:
        del ctx.player.statuses["Rage"]


@power_trigger("onUseCard", power="Rage")
def rage_on_attack(ctx: PowerContext) -> None:
    """Rage: Gain Block when playing an Attack card."""
    from ..content.cards import ALL_CARDS, CardType
    card = ctx.trigger_data.get("card")
    if card is not None and getattr(card, "card_type", None) == CardType.ATTACK:
        ctx.gain_block(ctx.amount)
        return
    card_id = ctx.trigger_data.get("card_id", "")
    base_id = card_id.rstrip("+")
    if base_id in ALL_CARDS and ALL_CARDS[base_id].card_type == CardType.ATTACK:
        ctx.gain_block(ctx.amount)


@power_trigger("onUseCard", power="DoubleTap")
def double_tap_on_attack(ctx: PowerContext) -> None:
    """Double Tap: Play Attack card twice (handled by combat engine)."""
    from ..content.cards import ALL_CARDS, CardType
    card = ctx.trigger_data.get("card")
    if card is not None and getattr(card, "card_type", None) == CardType.ATTACK:
        # Mark that this attack should be played again
        ctx.state.play_card_again = True
        # Decrement DoubleTap counter
        if ctx.amount > 1:
            ctx.player.statuses["DoubleTap"] = ctx.amount - 1
        else:
            del ctx.player.statuses["DoubleTap"]
        return
    card_id = ctx.trigger_data.get("card_id", "")
    base_id = card_id.rstrip("+")
    if base_id in ALL_CARDS and ALL_CARDS[base_id].card_type == CardType.ATTACK:
        # Mark that this attack should be played again
        ctx.state.play_card_again = True
        # Decrement DoubleTap counter
        if ctx.amount > 1:
            ctx.player.statuses["DoubleTap"] = ctx.amount - 1
        else:
            del ctx.player.statuses["DoubleTap"]
# =============================================================================
# SILENT POWER TRIGGERS
# =============================================================================

# -----------------------------------------------------------------------------
# Start of Turn
# -----------------------------------------------------------------------------

@power_trigger("atStartOfTurnPostDraw", power="ToolsOfTheTrade")
def tools_of_trade_start(ctx: PowerContext) -> None:
    """Tools of the Trade: Draw after normal turn draw, then require discard."""
    ctx.draw_cards(1)
    # Mark that discard is needed
    ctx.state.pending_tools_discard = True


@power_trigger("atStartOfTurnPostDraw", power="NextTurnDraw")
@power_trigger("atStartOfTurnPostDraw", power="DrawCardNextTurn")
def next_turn_draw_start(ctx: PowerContext) -> None:
    """DrawCardNextTurnPower: draw cards post-draw, then remove."""
    ctx.draw_cards(ctx.amount)
    ctx.player.statuses.pop("NextTurnDraw", None)
    ctx.player.statuses.pop("DrawCardNextTurn", None)


@power_trigger("atEndOfRound", power="IntangiblePlayer")
def intangible_player_end_of_round(ctx: PowerContext) -> None:
    """IntangiblePlayer: decrement and remove at end of round."""
    current = ctx.player.statuses.get("IntangiblePlayer", 0)
    if current > 1:
        ctx.player.statuses["IntangiblePlayer"] = current - 1
    elif current == 1:
        del ctx.player.statuses["IntangiblePlayer"]


@power_trigger("atStartOfTurn", power="NextTurnEnergy")
def next_turn_energy_start(ctx: PowerContext) -> None:
    """Next Turn Energy: Gain energy, then remove."""
    ctx.gain_energy(ctx.amount)
    del ctx.player.statuses["NextTurnEnergy"]


@power_trigger("atStartOfTurn", power="PhantasmalKiller")
def phantasmal_killer_start(ctx: PowerContext) -> None:
    """Phantasmal Killer: Double damage this turn, then remove."""
    # Mark that damage should be doubled
    ctx.state.double_damage_this_turn = True
    del ctx.player.statuses["PhantasmalKiller"]


@power_trigger("atEndOfRound", power="Blur")
def blur_end_of_round(ctx: PowerContext) -> None:
    """Blur: decrement at end of round (Java: BlurPower.atEndOfRound)."""
    current = ctx.player.statuses.get("Blur", 0)
    if current > 1:
        ctx.player.statuses["Blur"] = current - 1
    else:
        del ctx.player.statuses["Blur"]


# -----------------------------------------------------------------------------
# On Card Play
# -----------------------------------------------------------------------------

# Note: ThousandCuts is defined above in the ON_AFTER_CARD_PLAYED section

@power_trigger("onUseCard", power="Burst")
def burst_on_use(ctx: PowerContext) -> None:
    """Burst: Play the next skill(s) twice."""
    from ..content.cards import ALL_CARDS, CardType
    card = ctx.trigger_data.get("card")
    card_id = getattr(card, "id", "") if card is not None else ctx.trigger_data.get("card_id", "")
    base_id = card_id.rstrip("+")
    card_type = getattr(card, "card_type", None)
    if card_type is None and base_id in ALL_CARDS:
        card_type = ALL_CARDS[base_id].card_type
    if card_type == CardType.SKILL and base_id != "Burst":
        # Mark for double play
        ctx.state.play_again = True
        # Decrement Burst
        current = ctx.player.statuses.get("Burst", 0)
        if current > 1:
            ctx.player.statuses["Burst"] = current - 1
        else:
            del ctx.player.statuses["Burst"]


@power_trigger("onUseCard", power="Accuracy")
def accuracy_on_shiv(ctx: PowerContext) -> None:
    """Accuracy: Shivs deal extra damage (applied in damage calculation)."""
    # This is handled in damage calculation, not on card play
    pass


# -----------------------------------------------------------------------------
# On Discard
# -----------------------------------------------------------------------------

@power_trigger("onManualDiscard", power="Reflex")
def reflex_on_discard(ctx: PowerContext) -> None:
    """Reflex: Draw cards when discarded."""
    card_id = ctx.trigger_data.get("card_id", "")
    if card_id.startswith("Reflex"):
        # Get amount from the card itself (magic_number)
        from ..content.cards import ALL_CARDS
        if card_id in ALL_CARDS:
            card = ALL_CARDS[card_id]
            draw_amount = card.magic_number if card.magic_number > 0 else 2
            ctx.draw_cards(draw_amount)


@power_trigger("onManualDiscard", power="Tactician")
def tactician_on_discard(ctx: PowerContext) -> None:
    """Tactician: Gain energy when discarded."""
    card_id = ctx.trigger_data.get("card_id", "")
    if card_id.startswith("Tactician"):
        # Get amount from the card itself (magic_number)
        from ..content.cards import ALL_CARDS
        if card_id in ALL_CARDS:
            card = ALL_CARDS[card_id]
            energy_amount = card.magic_number if card.magic_number > 0 else 1
            ctx.gain_energy(energy_amount)


@power_trigger("onManualDiscard", power="SneakyStrike")
def sneaky_strike_discard_tracker(ctx: PowerContext) -> None:
    """Track that a card was discarded this turn for Sneaky Strike."""
    ctx.state.discarded_this_turn = getattr(ctx.state, 'discarded_this_turn', 0) + 1


# -----------------------------------------------------------------------------
# End of Turn
# -----------------------------------------------------------------------------

@power_trigger("atEndOfTurn", power="WellLaidPlans")
def well_laid_plans_end(ctx: PowerContext) -> None:
    """Well-Laid Plans: Mark cards to retain (selection happens in UI)."""
    ctx.state.retain_selection_count = ctx.amount


@power_trigger("atEndOfTurn", power="NoDraw")
def no_draw_end(ctx: PowerContext) -> None:
    """NoDraw (from Battle Trance): Remove at end of turn."""
    if "NoDraw" in ctx.player.statuses:
        del ctx.player.statuses["NoDraw"]
    if "No Draw" in ctx.player.statuses:
        del ctx.player.statuses["No Draw"]


@power_trigger("atEndOfTurn", power="ZeroCostCards")
def zero_cost_cards_end(ctx: PowerContext) -> None:
    """Zero Cost Cards: Remove at end of turn (Bullet Time)."""
    if "ZeroCostCards" in ctx.player.statuses:
        del ctx.player.statuses["ZeroCostCards"]


@power_trigger("atEndOfTurn", power="Burst")
def burst_end_of_turn(ctx: PowerContext) -> None:
    """Burst: Remove at end of turn even if no skills were played.

    In Java, BurstPower.atEndOfTurn() removes the power regardless of whether
    any skills were doubled. This prevents Burst from persisting to next turn.
    """
    if "Burst" in ctx.player.statuses:
        del ctx.player.statuses["Burst"]


# -----------------------------------------------------------------------------
# Damage Modifiers
# -----------------------------------------------------------------------------

@power_trigger("atDamageGive", power="Accuracy")
def accuracy_damage_give(ctx: PowerContext) -> int:
    """Accuracy: Shivs deal extra damage."""
    card_id = ctx.trigger_data.get("card_id", "")
    base_damage = ctx.trigger_data.get("value", 0)
    if card_id.startswith("Shiv"):
        return base_damage + ctx.amount
    return base_damage


# -----------------------------------------------------------------------------
# On Death (Corpse Explosion)
# -----------------------------------------------------------------------------

@power_trigger("onDeath", power="CorpseExplosion")
def corpse_explosion_on_death(ctx: PowerContext) -> None:
    """Corpse Explosion: Deal damage to all enemies when enemy dies."""
    dying_enemy = ctx.trigger_data.get("dying_enemy")
    if dying_enemy:
        # Deal damage equal to dying enemy's max HP to all other enemies
        max_hp = dying_enemy.max_hp
        for enemy in ctx.living_enemies:
            if enemy != dying_enemy:
                # THORNS type damage (bypasses block? Actually uses attack damage calculation)
                blocked = min(enemy.block, max_hp)
                enemy.block -= blocked
                enemy.hp -= (max_hp - blocked)
                if enemy.hp < 0:
                    enemy.hp = 0


# =============================================================================
# POW-002B / POW-003A -- System / Shared Powers
# =============================================================================

# --- NoBlockPower: modifyBlockLast -> 0, atEndOfRound decrement ---

@power_trigger("modifyBlockLast", power="NoBlockPower")
def no_block_modify_block_last(ctx: PowerContext) -> int:
    """NoBlockPower: Reduce all block gained from cards to 0.

    Java: NoBlockPower.modifyBlockLast() returns 0.0f.
    """
    return 0


@power_trigger("atEndOfRound", power="NoBlockPower")
def no_block_end_round(ctx: PowerContext) -> None:
    """NoBlockPower: Decrement at end of round; remove at 0.

    Java: atEndOfRound() -- if amount == 0, remove; else reduce by 1.
    justApplied flag is handled in combat engine (skip first round).
    """
    if ctx.owner is None:
        return
    if ctx.amount <= 0:
        _sync_owner_power_amount(ctx, "NoBlockPower", 0)
    else:
        _sync_owner_power_amount(ctx, "NoBlockPower", ctx.amount - 1)


# --- Entangled: canPlayCard check is in combat engine, atEndOfTurn remove ---

@power_trigger("atEndOfTurn", power="Entangled")
def entangled_end(ctx: PowerContext) -> None:
    """Entangled: Remove at end of turn.

    Java: EntanglePower.atEndOfTurn(isPlayer) -- removes itself.
    The canPlayCard check is inline in CombatEngine._can_play_card().
    """
    if ctx.owner == ctx.player:
        ctx.player.statuses.pop("Entangled", None)


# --- Double Damage: atDamageGive double, atEndOfRound decrement ---

@power_trigger("atDamageGive", power="Double Damage", priority=6)
def double_damage_give(ctx: PowerContext) -> int:
    """Double Damage: Double NORMAL damage dealt.

    Java: DoubleDamagePower.atDamageGive -- damage * 2 for NORMAL type.
    Priority 6 matches Java.
    """
    base_damage = ctx.trigger_data.get("value", 0)
    if ctx.trigger_data.get("damage_type", "NORMAL") != "NORMAL":
        return base_damage
    return base_damage * 2.0


@power_trigger("atEndOfRound", power="Double Damage")
def double_damage_end_round(ctx: PowerContext) -> None:
    """Double Damage: Decrement at end of round; remove at 0.

    Java: DoubleDamagePower.atEndOfRound -- justApplied skip then reduce/remove.
    """
    if ctx.owner is None:
        return
    if ctx.amount <= 0:
        _sync_owner_power_amount(ctx, "Double Damage", 0)
    else:
        _sync_owner_power_amount(ctx, "Double Damage", ctx.amount - 1)


# --- Repair: onVictory heal ---

@power_trigger("onVictory", power="Repair")
def repair_on_victory(ctx: PowerContext) -> None:
    """Repair: Heal amount on victory if player alive.

    Java: RepairPower.onVictory -- p.heal(this.amount) if currentHealth > 0.
    """
    if ctx.player.hp > 0:
        heal = min(ctx.amount, ctx.player.max_hp - ctx.player.hp)
        ctx.player.hp += heal


# --- Lock-On: atEndOfRound decrement ---
# Note: Lock-On's orb damage modifier is passive (checked by orb system), not a hook.

@power_trigger("atEndOfRound", power="Lockon")
@power_trigger("atEndOfRound", power="Lock-On")
def lockon_end_round(ctx: PowerContext) -> None:
    """Lock-On: Decrement at end of round; remove at 0.

    Java: LockOnPower.atEndOfRound -- if amount == 0, remove; else reduce by 1.
    """
    if ctx.owner is None:
        return
    if ctx.amount <= 0:
        _sync_owner_power_amount(ctx, "Lockon", 0)
    else:
        _sync_owner_power_amount(ctx, "Lockon", ctx.amount - 1)


# --- Panache: atStartOfTurn reset counter ---
# Note: The onUseCard handler for Panache already exists above. This adds the
# atStartOfTurn reset that Java does in PanachePower.atStartOfTurn().

@power_trigger("atStartOfTurn", power="Panache")
def panache_start_of_turn(ctx: PowerContext) -> None:
    """Panache: Reset card counter to 5 at start of turn.

    Java: PanachePower.atStartOfTurn -- this.amount = 5.
    We track counter in PanacheCounter status separately.
    """
    ctx.player.statuses["PanacheCounter"] = 0


# =============================================================================
# POW-002B / POW-003A -- Boss / Enemy Powers
# =============================================================================

# --- Angry: onAttacked gain Strength ---

@power_trigger("onAttacked", power="Angry")
def angry_on_attacked(ctx: PowerContext) -> None:
    """Angry: Gain Strength when hit by non-HP_LOSS/THORNS damage.

    Java: AngryPower.onAttacked -- if info.owner != null && damageAmount > 0
    && info.type != HP_LOSS && info.type != THORNS => gain Strength.
    """
    if ctx.owner is None:
        return
    attacker = ctx.trigger_data.get("attacker")
    unblocked = ctx.trigger_data.get("unblocked_damage", ctx.trigger_data.get("damage", 0))
    damage_type = ctx.trigger_data.get("damage_type", "NORMAL")
    if attacker is not None and unblocked > 0 and damage_type not in ("HP_LOSS", "THORNS"):
        ctx.owner.statuses["Strength"] = ctx.owner.statuses.get("Strength", 0) + ctx.amount


# --- Curiosity: onPlayCard Power -> gain Strength ---
# Java fires onUseCard on each monster's powers; Python dispatches onPlayCard to enemies.

@power_trigger("onPlayCard", power="Curiosity")
def curiosity_on_play(ctx: PowerContext) -> None:
    """Curiosity: Gain Strength when player plays a Power card.

    Java: CuriosityPower.onUseCard -- if card.type == POWER, gain Strength.
    """
    if ctx.owner is None:
        return
    from ..content.cards import CardType
    card = ctx.trigger_data.get("card")
    if card is not None and getattr(card, "card_type", None) == CardType.POWER:
        ctx.owner.statuses["Strength"] = ctx.owner.statuses.get("Strength", 0) + ctx.amount


# --- GrowthPower: atEndOfRound gain Strength with skip-first logic ---

@power_trigger("atEndOfRound", power="GrowthPower")
def growth_power_end_round(ctx: PowerContext) -> None:
    """GrowthPower: Gain Strength at end of round, skip first round.

    Java: GrowthPower.atEndOfRound -- skipFirst flag; on subsequent rounds,
    applies Strength equal to amount.
    """
    if ctx.owner is None:
        return
    key = _runtime_counter_key(ctx, "growth_skip_first")
    skip = ctx.state.relic_counters.get(key, 1)
    if skip:
        ctx.state.relic_counters[key] = 0
    else:
        ctx.owner.statuses["Strength"] = ctx.owner.statuses.get("Strength", 0) + ctx.amount


# --- Fading: duringTurn decrement, suicide at 0 ---
# Java uses duringTurn() which fires during the enemy's turn. We dispatch
# this at start of each enemy turn in the enemy turn loop.

@power_trigger("atEndOfTurn", power="Fading")
def fading_during_turn(ctx: PowerContext) -> None:
    """Fading: Decrement each turn; at 1, the creature dies (suicide).

    Java: FadingPower.duringTurn -- if amount == 1, suicide; else reduce by 1.
    We use atEndOfTurn for enemy owners since duringTurn fires per-enemy.
    """
    if ctx.owner is None or ctx.owner is ctx.player:
        return
    if ctx.amount <= 1:
        # Kill the creature
        ctx.owner.hp = 0
    else:
        ctx.owner.statuses["Fading"] = ctx.amount - 1


# --- Thievery: onAttack steal gold ---
# Java: ThieveryPower has no hook methods itself -- the gold steal is done
# in the damage action. We add an onAttack hook for completeness.

@power_trigger("onAttack", power="Thievery")
def thievery_on_attack(ctx: PowerContext) -> None:
    """Thievery: Steal gold from player on unblocked attack damage.

    Java: The steal is handled in the attack action -- steal amount gold
    when dealing unblocked damage.
    """
    if ctx.owner is None or ctx.owner is ctx.player:
        return
    unblocked = ctx.trigger_data.get("unblocked_damage", 0)
    if unblocked > 0 and hasattr(ctx.state, "gold"):
        stolen = min(ctx.amount, ctx.state.gold)
        ctx.state.gold -= stolen


# =============================================================================
# POW-002B / POW-003A -- Defect Powers
# =============================================================================

# --- Storm: onUseCard Power -> channel Lightning ---

@power_trigger("onUseCard", power="Storm")
def storm_on_use(ctx: PowerContext) -> None:
    """Storm: Channel Lightning when playing a Power card.

    Java: StormPower.onUseCard -- if card.type == POWER, channel Lightning
    amount times.
    """
    from ..content.cards import CardType
    card = ctx.trigger_data.get("card")
    if card is not None and getattr(card, "card_type", None) == CardType.POWER:
        for _ in range(ctx.amount):
            ctx.channel_orb("Lightning")


# --- Static Discharge: onAttacked channel Lightning ---

@power_trigger("onAttacked", power="StaticDischarge")
def static_discharge_on_attacked(ctx: PowerContext) -> None:
    """Static Discharge: Channel Lightning when taking non-THORNS/HP_LOSS damage.

    Java: StaticDischargePower.onAttacked -- if type != THORNS && type != HP_LOSS
    && info.owner != null && info.owner != this.owner && damageAmount > 0.
    """
    attacker = ctx.trigger_data.get("attacker")
    damage_type = ctx.trigger_data.get("damage_type", "NORMAL")
    unblocked = ctx.trigger_data.get("unblocked_damage", ctx.trigger_data.get("damage", 0))
    if (
        attacker is not None
        and attacker is not ctx.owner
        and unblocked > 0
        and damage_type not in ("THORNS", "HP_LOSS")
    ):
        for _ in range(ctx.amount):
            ctx.channel_orb("Lightning")


# =============================================================================
# POW-002B / POW-003A -- Watcher Powers
# =============================================================================

# --- BlockReturnPower: onAttacked gain block ---

@power_trigger("onAttacked", power="BlockReturnPower")
def block_return_on_attacked(ctx: PowerContext) -> None:
    """BlockReturn (Talk to the Hand mark): Player gains block when attacking
    an enemy that has this power.

    Java: BlockReturnPower is applied to enemies. When the enemy is attacked
    and takes unblocked damage, the player gains block equal to the power amount.
    """
    if ctx.owner is None or ctx.owner is ctx.player:
        return
    # This power lives on the enemy; when that enemy is attacked, player gains block
    attacker = ctx.trigger_data.get("attacker")
    unblocked = ctx.trigger_data.get("unblocked_damage", ctx.trigger_data.get("damage", 0))
    if attacker is not None and unblocked > 0:
        ctx.player.block += ctx.amount


# --- EstablishmentPower: reduce retained card costs ---

# Establishment: cost reduction for retained cards handled inline in
# combat_engine.py _start_player_turn() using card_costs dict.
# Reduces cost of cards in hand (retained from last turn) by Establishment stacks.


# --- FreeAttackPower: onUseCard decrement on ATTACK ---

@power_trigger("onUseCard", power="FreeAttackPower")
def free_attack_on_use(ctx: PowerContext) -> None:
    """FreeAttackPower: Decrement when an Attack card is played.

    Java: FreeAttackPower.onUseCard -- if card.type == ATTACK, reduce amount.
    The cost reduction is applied when the card cost is calculated.
    """
    from ..content.cards import CardType
    card = ctx.trigger_data.get("card")
    if card is not None and getattr(card, "card_type", None) == CardType.ATTACK:
        if ctx.amount > 1:
            ctx.player.statuses["FreeAttackPower"] = ctx.amount - 1
        else:
            ctx.player.statuses.pop("FreeAttackPower", None)


# --- CannotChangeStancePower: block stance change + remove at end of turn ---

@power_trigger("atEndOfTurn", power="CannotChangeStancePower")
def cannot_change_stance_end(ctx: PowerContext) -> None:
    """CannotChangeStancePower: Remove at end of turn.

    Java: CannotChangeStancePower is removed at end of turn.
    The stance-change blocking check is inline in the combat engine.
    """
    ctx.player.statuses.pop("CannotChangeStancePower", None)


# =============================================================================
# MISSING WATCHER POWERS (added from Java parity audit 2026-03-03)
# =============================================================================

# --- EndTurnDeath (Blasphemy): die at end of turn ---
# NOTE: Java fires at atStartOfTurn (next turn), but for RL simplicity
# we keep end-of-turn timing. The rare edge cases (Blasphemy+Vault,
# Blasphemy+Fairy) don't affect agent training.

@power_trigger("atEndOfTurn", power="EndTurnDeath")
@power_trigger("atEndOfTurn", power="Blasphemy")
def end_turn_death(ctx: PowerContext) -> None:
    """EndTurnDeath/Blasphemy: Kill player at end of turn.

    Java timing: atStartOfTurn (next turn). Simplified to end-of-turn
    for RL training since the timing edge cases are rare.
    """
    ctx.player.hp = 0
    ctx.player.statuses.pop("EndTurnDeath", None)
    ctx.player.statuses.pop("Blasphemy", None)


# --- EnergyDownPower (Fasting): lose energy at start of turn ---

@power_trigger("atStartOfTurn", power="EnergyDownPower")
@power_trigger("atStartOfTurn", power="Fasting")
def energy_down_start(ctx: PowerContext) -> None:
    """EnergyDownPower: Lose energy at start of turn (Fasting card).

    Java: EnergyDownPower.atStartOfTurn() → LoseEnergyAction(amount).
    Persists until removed.
    """
    ctx.state.energy = max(0, ctx.state.energy - ctx.amount)


# --- AngelForm / LiveForeverPower: gain Plated Armor at end of turn ---

@power_trigger("atEndOfTurn", power="AngelForm")
@power_trigger("atEndOfTurn", power="LiveForever")
def angel_form_end(ctx: PowerContext) -> None:
    """AngelForm/LiveForever: Gain Plated Armor at end of turn.

    Java: LiveForeverPower.atEndOfTurn() → ApplyPowerAction(PlatedArmor, amount).
    """
    current = ctx.player.statuses.get("Plated Armor", 0)
    ctx.player.statuses["Plated Armor"] = current + ctx.amount


# =============================================================================
# MISSING ENEMY POWERS (batch implementation 2026-03-03)
# =============================================================================

# --- AngerPower: Nob gains Str when player plays Skill ---
@power_trigger("onPlayCard", power="Anger")
def anger_on_play_card(ctx: PowerContext) -> None:
    """Anger: Gain Strength when player plays a Skill card.
    Java: AngerPower.onPlayCard — if card.type == SKILL, gain amount Str.
    Used by GremlinNob.
    """
    card = ctx.trigger_data.get("card")
    if card is not None:
        from ..content.cards import CardType
        if getattr(card, "card_type", None) == CardType.SKILL:
            if ctx.owner is not None:
                current = ctx.owner.statuses.get("Strength", 0)
                ctx.owner.statuses["Strength"] = current + ctx.amount


# --- HexPower: shuffle Daze on non-Attack play ---
@power_trigger("onUseCard", power="Hex")
def hex_on_use(ctx: PowerContext) -> None:
    """Hex: Add Daze to draw pile when player plays non-Attack.
    Java: HexPower.onUseCard — if card.type != ATTACK, shuffle Daze into draw.
    Used by Hexaghost (via enemy power).
    """
    card = ctx.trigger_data.get("card")
    if card is not None:
        from ..content.cards import CardType
        if getattr(card, "card_type", None) != CardType.ATTACK:
            ctx.state.draw_pile.append("Daze")
            ctx.shuffle_in_place(ctx.state.draw_pile)


# --- GainStrengthPower (Shackled): regain Str at end of turn ---
@power_trigger("atEndOfTurn", power="Shackled")
@power_trigger("atEndOfTurn", power="GainStrength")
def shackled_end(ctx: PowerContext) -> None:
    """Shackled/GainStrength: Regain Strength at end of turn.
    Java: GainStrengthPower.atEndOfTurn — gain amount Str then remove.
    Applied when temporary Strength loss expires.
    """
    if ctx.owner is not None:
        current = ctx.owner.statuses.get("Strength", 0)
        ctx.owner.statuses["Strength"] = current + ctx.amount
        if ctx.owner.statuses["Strength"] == 0:
            ctx.owner.statuses.pop("Strength", None)
    _sync_owner_power_amount(ctx, "Shackled", 0)
    _sync_owner_power_amount(ctx, "GainStrength", 0)


# --- GenericStrengthUpPower: gain Str each round ---
@power_trigger("atEndOfRound", power="Generic Strength Up Power")
def generic_str_up_end(ctx: PowerContext) -> None:
    """GenericStrengthUpPower: Gain Strength at end of round."""
    if ctx.owner is not None:
        current = ctx.owner.statuses.get("Strength", 0)
        ctx.owner.statuses["Strength"] = current + ctx.amount


# --- ForcefieldPower (Nullify Attack): reduce non-Attack damage ---
@power_trigger("atDamageFinalReceive", power="Nullify Attack")
@power_trigger("atDamageFinalReceive", power="Forcefield")
def forcefield_receive(ctx: PowerContext) -> int:
    """Forcefield: Reduce all non-HP_LOSS damage to 0.
    Java: ForcefieldPower.atDamageFinalReceive — if not HP_LOSS, return 0.
    Used by Spheric Guardian.
    """
    damage = ctx.trigger_data.get("value", ctx.damage)
    if ctx.damage_type != "HP_LOSS":
        return 0
    return damage


# --- ShiftingPower: lose Strength equal to unblocked damage (temporary) ---
@power_trigger("onAttacked", power="Shifting")
def shifting_attacked(ctx: PowerContext) -> None:
    """Shifting: Lose Strength equal to unblocked damage taken, regain at end of turn.

    Java: ShiftingPower.onAttacked — apply -damageAmount Str + GainStrength(damageAmount).
    Used by Nemesis, Book of Stabbing.
    """
    damage = ctx.trigger_data.get("unblocked_damage", ctx.trigger_data.get("damage", 0))
    if damage > 0 and ctx.owner is not None:
        ctx.owner.statuses["Strength"] = ctx.owner.statuses.get("Strength", 0) - damage
        # GainStrength: restores Strength at end of turn (unless Artifact blocks it)
        if ctx.owner.statuses.get("Artifact", 0) <= 0:
            ctx.owner.statuses["GainStrength"] = ctx.owner.statuses.get("GainStrength", 0) + damage


# --- TheBombPower: deal damage after 3 turns ---
@power_trigger("atEndOfTurn", power="TheBomb")
def the_bomb_end(ctx: PowerContext) -> None:
    """TheBomb: Count down, deal massive damage when reaching 0.
    Java: TheBombPower.atEndOfTurn — decrement, deal 40 dmg at 0.
    """
    if ctx.owner is not None:
        key = f"__bomb_counter__{id(ctx.owner)}"
        counter = ctx.state.relic_counters.get(key, 3)
        counter -= 1
        if counter <= 0:
            # Deal damage to all enemies (it's a player power)
            for enemy in ctx.living_enemies:
                hp_dmg = max(0, ctx.amount - enemy.block)
                enemy.block = max(0, enemy.block - ctx.amount)
                enemy.hp = max(0, enemy.hp - hp_dmg)
            ctx.state.relic_counters.pop(key, None)
            ctx.player.statuses.pop("TheBomb", None)
        else:
            ctx.state.relic_counters[key] = counter


# --- RegenerateMonsterPower: enemy heals at end of turn ---
@power_trigger("atEndOfTurn", power="Regenerate")
@power_trigger("atEndOfTurn", power="RegenerateMonster")
def regenerate_monster_end(ctx: PowerContext) -> None:
    """RegenerateMonsterPower: Heal at end of turn.
    Java: RegenerateMonsterPower.atEndOfTurn — heal(amount).
    """
    if ctx.owner is not None and ctx.owner is not ctx.player:
        heal = min(ctx.amount, ctx.owner.max_hp - ctx.owner.hp)
        if heal > 0:
            ctx.owner.hp += heal


# --- PainfulStabsPower: add Wound when attacked ---
@power_trigger("onAttacked", power="Painful Stabs")
def painful_stabs_attacked(ctx: PowerContext) -> None:
    """PainfulStabs: Add Wound to player discard when dealing unblocked damage.
    Java: PainfulStabsPower.onAttacked — if dmg > 0, add Wound.
    """
    damage = ctx.trigger_data.get("damage", 0)
    if damage > 0:
        ctx.state.discard_pile.append("Wound")


# --- NoSkillsPower: remove at end of turn (can't play check is in combat engine) ---
@power_trigger("atEndOfTurn", power="NoSkills")
def no_skills_end(ctx: PowerContext) -> None:
    """NoSkillsPower: Remove at end of turn."""
    ctx.player.statuses.pop("NoSkills", None)


# --- ConfusionPower (Snecko Eye): randomize card costs on draw ---
@power_trigger("onCardDraw", power="Confusion")
def confusion_on_draw(ctx: PowerContext) -> None:
    """Confusion: Randomize drawn card cost to 0-3.
    Java: ConfusionPower.onCardDraw — set cost to cardRandomRng.random(3).
    """
    card_id = ctx.trigger_data.get("card_id", "")
    if card_id:
        rng = ctx._card_random_rng()
        new_cost = rng.random(3) if rng else 0
        ctx.state.card_costs[card_id] = new_cost


# --- CollectPower: add Miracles at start of turn ---
@power_trigger("onEnergyRecharge", power="Collect")
def collect_energy(ctx: PowerContext) -> None:
    """CollectPower: Add Miracle to hand (from Collect card).
    Java: CollectPower.onEnergyRecharge — add Miracle to hand, decrement.
    """
    for _ in range(ctx.amount):
        ctx.add_card_to_hand("Miracle")
    new_val = ctx.amount - 1
    if new_val <= 0:
        ctx.player.statuses.pop("Collect", None)
    else:
        ctx.player.statuses["Collect"] = new_val


# =============================================================================
# ADDITIONAL MISSING POWERS (batch implementation 2026-03-11)
# =============================================================================

# --- AttackBurnPower: exhaust Attack cards played by the player ---
@power_trigger("onPlayCard", power="Attack Burn")
def attack_burn_on_play(ctx: PowerContext) -> None:
    """Attack Burn: When player plays an Attack, exhaust it instead of discarding.

    Java: AttackBurnPower.onUseCard — sets action.exhaustCard = true on Attacks.
    Also decrements 1 per round after first application (handled by atEndOfRound).
    Used by Book of Stabbing.
    """
    from ..content.cards import CardType
    card = ctx.trigger_data.get("card")
    if card is not None and getattr(card, "card_type", None) == CardType.ATTACK:
        # Mark card for exhaust — the combat engine checks this flag
        ctx.trigger_data["force_exhaust"] = True


@power_trigger("atEndOfRound", power="Attack Burn")
def attack_burn_end_round(ctx: PowerContext) -> None:
    """Attack Burn: Decrement by 1 each round, skip first round.

    Java: AttackBurnPower.atEndOfRound — justApplied skip, then ReducePower 1.
    """
    if ctx.owner is None:
        return
    key = _runtime_counter_key(ctx, "attack_burn_skip")
    skip = ctx.state.relic_counters.get(key, 1)
    if skip:
        ctx.state.relic_counters[key] = 0
    else:
        new_val = ctx.amount - 1
        if new_val <= 0:
            ctx.owner.statuses.pop("Attack Burn", None)
        else:
            _sync_owner_power_amount(ctx, "Attack Burn", new_val)


# --- Compulsive (Reactive): re-roll intent when hit ---
@power_trigger("onAttacked", power="Compulsive")
def compulsive_on_attacked(ctx: PowerContext) -> None:
    """Compulsive/Reactive: Re-roll enemy move when taking non-lethal damage.

    Java: ReactivePower (ID="Compulsive").onAttacked — if info.owner != null,
    type != HP_LOSS/THORNS, damageAmount > 0 and < currentHealth, re-roll move.
    Used by WrithingMass.
    """
    if ctx.owner is None or ctx.owner is ctx.player:
        return
    damage = ctx.trigger_data.get("unblocked_damage", ctx.trigger_data.get("damage", 0))
    damage_type = ctx.trigger_data.get("damage_type", "NORMAL")
    attacker = ctx.trigger_data.get("attacker")
    if (attacker is not None and damage > 0
            and damage_type not in ("HP_LOSS", "THORNS")
            and damage < ctx.owner.hp):
        # Signal the combat engine to re-roll this enemy's move
        ctx.trigger_data["reroll_move"] = True


# --- Sharp Hide: deal THORNS damage to player on Attack play ---
# NOTE: Handled inline in combat_engine._deal_damage_to_enemy per-hit.
# Java fires this per-card-play (onUseCard), but the inline per-hit handler
# has been in the engine longer and tests validate it. We remove the duplicate
# registry trigger to avoid double-dipping.


# --- DrawReduction: reduce draw by amount, decrement at end of round ---
@power_trigger("atEndOfRound", power="Draw Reduction")
def draw_reduction_end_round(ctx: PowerContext) -> None:
    """DrawReduction: Decrement by 1 each round, skip first round.

    Java: DrawReductionPower.atEndOfRound — justApplied skip, then ReducePower 1.
    onInitialApplication reduces gameHandSize, onRemove restores it.
    In Python, draw count check happens in _start_player_turn via DrawDown status.
    """
    if ctx.owner is None:
        return
    key = _runtime_counter_key(ctx, "draw_reduction_skip")
    skip = ctx.state.relic_counters.get(key, 1)
    if skip:
        ctx.state.relic_counters[key] = 0
    else:
        new_val = ctx.amount - 1
        if new_val <= 0:
            ctx.owner.statuses.pop("Draw Reduction", None)
            ctx.owner.statuses.pop("DrawReduction", None)
        else:
            _sync_owner_power_amount(ctx, "Draw Reduction", new_val)


# --- PlatedArmor wasHPLost: decrement by 1 when player loses HP ---
@power_trigger("wasHPLost", power="Plated Armor")
def plated_armor_hp_lost(ctx: PowerContext) -> None:
    """PlatedArmor wasHPLost: Lose 1 stack when taking non-self/non-HP_LOSS damage.

    Java: PlatedArmorPower.wasHPLost — if info.owner != null && owner != self
    && type != HP_LOSS/THORNS && damageAmount > 0, reduce by 1.
    """
    if ctx.owner is None:
        return
    damage = ctx.trigger_data.get("damage", 0)
    damage_type = ctx.trigger_data.get("damage_type", "NORMAL")
    is_self = ctx.trigger_data.get("is_self_damage", False)
    # Accept either explicit damage > 0 or the unblocked flag
    has_damage = damage > 0 or ctx.trigger_data.get("unblocked", False)
    if has_damage and not is_self and damage_type not in ("HP_LOSS", "THORNS"):
        new_val = ctx.amount - 1
        if new_val <= 0:
            ctx.owner.statuses.pop("Plated Armor", None)
        else:
            _sync_owner_power_amount(ctx, "Plated Armor", new_val)


# --- Amplify: next Power card is played twice ---
@power_trigger("onUseCard", power="Amplify")
def amplify_on_use(ctx: PowerContext) -> None:
    """Amplify: When player plays a Power card, play it again (double effect).

    Java: AmplifyPower.onUseCard — if card.type == POWER and !purgeOnUse,
    queue a copy. Decrement, remove at 0. Also removed at end of turn.
    In Python, we signal the engine to replay the card's effects.
    """
    if ctx.owner is None or ctx.owner is not ctx.player:
        return
    from ..content.cards import CardType
    card = ctx.trigger_data.get("card")
    if card is not None and getattr(card, "card_type", None) == CardType.POWER:
        ctx.trigger_data["amplify_replay"] = True
        new_val = ctx.amount - 1
        if new_val <= 0:
            ctx.player.statuses.pop("Amplify", None)
        else:
            ctx.player.statuses["Amplify"] = new_val


@power_trigger("atEndOfTurn", power="Amplify")
def amplify_end_turn(ctx: PowerContext) -> None:
    """Amplify: Remove at end of turn.

    Java: AmplifyPower.atEndOfTurn(isPlayer) — remove.
    """
    if ctx.owner is ctx.player:
        ctx.player.statuses.pop("Amplify", None)


# --- Rebound: next card goes to top of draw pile instead of discard ---
@power_trigger("onAfterUseCard", power="Rebound")
def rebound_after_use(ctx: PowerContext) -> None:
    """Rebound: Put next non-Power card on top of draw pile instead of discard.

    Java: ReboundPower.onAfterUseCard — skip first card (justEvoked),
    then set action.reboundCard = true for non-Power cards. Decrement.
    Removed at end of turn.
    """
    if ctx.owner is None or ctx.owner is not ctx.player:
        return
    key = _runtime_counter_key(ctx, "rebound_skip")
    skip = ctx.state.relic_counters.get(key, 1)
    if skip:
        ctx.state.relic_counters[key] = 0
        return
    from ..content.cards import CardType
    card = ctx.trigger_data.get("card")
    if card is not None and getattr(card, "card_type", None) != CardType.POWER:
        # Move card from discard to top of draw pile
        card_id = ctx.trigger_data.get("card_id", getattr(card, "id", ""))
        upgraded_id = card_id + "+" if getattr(card, "upgraded", False) and not card_id.endswith("+") else card_id
        # Try to find the card in discard pile and move to draw
        for cid in [upgraded_id, card_id]:
            if cid in ctx.state.discard_pile:
                ctx.state.discard_pile.remove(cid)
                ctx.state.draw_pile.append(cid)  # top of draw pile
                break
    new_val = ctx.amount - 1
    if new_val <= 0:
        ctx.player.statuses.pop("Rebound", None)
    else:
        ctx.player.statuses["Rebound"] = new_val


@power_trigger("atEndOfTurn", power="Rebound")
def rebound_end_turn(ctx: PowerContext) -> None:
    """Rebound: Remove at end of turn.

    Java: ReboundPower.atEndOfTurn(isPlayer) — remove.
    """
    if ctx.owner is ctx.player:
        ctx.player.statuses.pop("Rebound", None)


# =============================================================================
# NON-WATCHER POWERS (LOW PRIORITY -- for multi-class support)
# =============================================================================


@power_trigger("atStartOfTurn", power="Hello")
def hello_start_of_turn(ctx: PowerContext) -> None:
    """Hello World (Defect): Add random common card(s) to hand at start of turn.

    Java: HelloPower.atStartOfTurn -- for each amount, add a random common card
    to hand via MakeTempCardInHandAction. Uses cardRandomRng.
    """
    from ..content.cards import ALL_CARDS, CardRarity
    common_cards = [
        cid for cid, c in ALL_CARDS.items()
        if c.rarity == CardRarity.COMMON and not cid.endswith("+")
    ]
    if not common_cards:
        return
    for _ in range(ctx.amount):
        if hasattr(ctx.state, 'card_random_rng') and ctx.state.card_random_rng:
            idx = ctx.state.card_random_rng.random(len(common_cards) - 1)
        else:
            idx = 0
        card_id = common_cards[idx]
        if len(ctx.state.hand) < 10:
            ctx.state.hand.append(card_id)


@power_trigger("atStartOfTurn", power="Night Terror")
def nightmare_start_of_turn(ctx: PowerContext) -> None:
    """Nightmare (Silent): Add copies of stored card to hand, then remove power.

    Java: NightmarePower.atStartOfTurn -- adds `amount` copies of the stored
    card to hand via MakeTempCardInHandAction, then removes the power.
    The stored card ID is kept in trigger_data or state metadata.
    """
    # The card to copy is stored when Nightmare is played.
    # We store it as "NightmareCard" in the player's statuses dict (as a string).
    card_id = ctx.player.statuses.get("NightmareCard", "")
    if card_id:
        for _ in range(ctx.amount):
            if len(ctx.state.hand) < 10:
                ctx.state.hand.append(card_id)
        ctx.player.statuses.pop("NightmareCard", None)
    ctx.player.statuses.pop("Night Terror", None)


@power_trigger("atStartOfTurn", power="Winter")
def winter_start_of_turn(ctx: PowerContext) -> None:
    """Winter (Defect): Channel Frost orb(s) at start of turn.

    Java: WinterPower.atStartOfTurn -- channels `amount` Frost orbs.
    Defect-only power.
    """
    from ..effects.orbs import channel_orb
    for _ in range(ctx.amount):
        channel_orb(ctx.state, "Frost")


# =============================================================================
# STASIS (BronzeOrb onDeath — return stolen card to player)
# =============================================================================

@power_trigger("onDeath", power="Stasis")
def stasis_on_death(ctx: PowerContext) -> None:
    """Stasis: Return stolen card to player's hand (or discard if hand full).

    Java: StasisPower.onDeath — if hand < 10, MakeTempCardInHandAction;
    else MakeTempCardInDiscardAction.  The card was stored by _apply_stasis
    in combat_engine.py under relic_counters['__stasis_card__:{enemy_idx}'].
    """
    dying_enemy = ctx.trigger_data.get("dying_enemy")
    if dying_enemy is None:
        return

    # Find enemy index
    enemy_idx = -1
    for idx, e in enumerate(ctx.state.enemies):
        if e is dying_enemy:
            enemy_idx = idx
            break

    stasis_key = f"__stasis_card__:{enemy_idx}"
    stolen_card = ctx.state.relic_counters.pop(stasis_key, None)
    if stolen_card is None:
        return

    # Return card: hand if < 10, discard otherwise (Java parity)
    if len(ctx.state.hand) < 10:
        ctx.state.hand.append(stolen_card)
    else:
        ctx.state.discard_pile.append(stolen_card)


# =============================================================================
# DEAD CODE DOCUMENTATION
# =============================================================================
# The following Java powers exist as classes but are NEVER instantiated
# by any card, relic, enemy, or event in the base game. Verified by grepping
# the entire decompiled/ directory for "new <ClassName>" — zero matches.
#
# - ConservePower (ID="Conserve"): atEndOfRound decrements. Dead code.
# - RechargingCorePower (ID="RechargingCore"): atStartOfTurn timer for energy.
#     Dead code — NOT the same as Frozen Core relic.
# - SkillBurnPower (ID="Skill Burn"): exhaust Skills when played.
#     Dead code — BookOfStabbing uses PainfulStabsPower, not SkillBurn.
# - StrikeUpPower (ID="StrikeUp"): boost Strike card base damage.
#     Dead code — never applied by any card or relic.
# - TimeMazePower (ID="TimeMazePower"): end turn after N cards, Endless-mode
#     Blight mechanic. TimeEater uses TimeWarpPower (already implemented).
#     Not relevant for standard A20 gameplay.
# =============================================================================
