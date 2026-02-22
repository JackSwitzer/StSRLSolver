"""
Potion Effect Implementations.

This module contains all potion effect handlers using the registry pattern.
Each handler is registered via decorator and called when the potion is used.

Potions are organized by rarity for easier maintenance.
"""

from __future__ import annotations

from . import potion_effect, PotionContext


def _get_rng(state, *attrs):
    """Best-effort RNG resolver from state-attached RNG references."""
    for attr in attrs:
        rng = getattr(state, attr, None)
        if rng is not None:
            return rng
    return None


# =============================================================================
# COMMON POTIONS
# =============================================================================

@potion_effect("Block Potion")
def block_potion(ctx: PotionContext) -> None:
    """Block Potion: Gain 12 Block (24 with Sacred Bark)."""
    ctx.gain_block(ctx.potency)


@potion_effect("Dexterity Potion")
def dexterity_potion(ctx: PotionContext) -> None:
    """Dexterity Potion: Gain 2 Dexterity (4 with Sacred Bark)."""
    ctx.apply_power_to_player("Dexterity", ctx.potency)


@potion_effect("Energy Potion")
def energy_potion(ctx: PotionContext) -> None:
    """Energy Potion: Gain 2 Energy (4 with Sacred Bark)."""
    ctx.gain_energy(ctx.potency)


@potion_effect("Explosive Potion")
def explosive_potion(ctx: PotionContext) -> None:
    """Explosive Potion: Deal 10 damage to ALL enemies (20 with Sacred Bark)."""
    ctx.deal_damage_to_all_enemies(ctx.potency)


@potion_effect("Fire Potion", requires_target=True)
def fire_potion(ctx: PotionContext) -> None:
    """Fire Potion: Deal 20 damage to target enemy (40 with Sacred Bark)."""
    ctx.deal_damage_to_target(ctx.potency)


@potion_effect("Strength Potion")
def strength_potion(ctx: PotionContext) -> None:
    """Strength Potion: Gain 2 Strength (4 with Sacred Bark)."""
    ctx.apply_power_to_player("Strength", ctx.potency)


@potion_effect("Swift Potion")
def swift_potion(ctx: PotionContext) -> None:
    """Swift Potion: Draw 3 cards (6 with Sacred Bark)."""
    ctx.draw_cards(ctx.potency)


@potion_effect("Weak Potion", requires_target=True)
def weak_potion(ctx: PotionContext) -> None:
    """Weak Potion: Apply 3 Weak to target (6 with Sacred Bark)."""
    if ctx.target:
        ctx.apply_power(ctx.target, "Weakened", ctx.potency)


@potion_effect("FearPotion", requires_target=True)
def fear_potion(ctx: PotionContext) -> None:
    """Fear Potion: Apply 3 Vulnerable to target (6 with Sacred Bark)."""
    if ctx.target:
        ctx.apply_power(ctx.target, "Vulnerable", ctx.potency)


@potion_effect("AttackPotion")
def attack_potion(ctx: PotionContext) -> None:
    """Attack Potion: Discover an Attack card (costs 0 this turn).
    With Sacred Bark, add 2 copies instead of 1."""
    # Simplified: add a deterministic attack card at cost 0 this turn
    from ..content.cards import ALL_CARDS, CardType, CardColor

    attacks = [
        cid for cid, card in ALL_CARDS.items()
        if card.card_type == CardType.ATTACK
        and card.color != CardColor.COLORLESS
    ]

    if attacks:
        chosen = attacks[0]
        copies = ctx.potency
        for _ in range(copies):
            if len(ctx.state.hand) < 10:
                ctx.state.hand.append(chosen)
                ctx.state.card_costs[chosen] = 0  # Costs 0 this turn


@potion_effect("SkillPotion")
def skill_potion(ctx: PotionContext) -> None:
    """Skill Potion: Discover a Skill card (costs 0 this turn)."""
    from ..content.cards import ALL_CARDS, CardType, CardColor

    skills = [
        cid for cid, card in ALL_CARDS.items()
        if card.card_type == CardType.SKILL
        and card.color != CardColor.COLORLESS
    ]

    if skills:
        chosen = skills[0]
        copies = ctx.potency
        for _ in range(copies):
            if len(ctx.state.hand) < 10:
                ctx.state.hand.append(chosen)
                ctx.state.card_costs[chosen] = 0


@potion_effect("PowerPotion")
def power_potion(ctx: PotionContext) -> None:
    """Power Potion: Discover a Power card (costs 0 this turn)."""
    from ..content.cards import ALL_CARDS, CardType, CardColor

    powers = [
        cid for cid, card in ALL_CARDS.items()
        if card.card_type == CardType.POWER
        and card.color != CardColor.COLORLESS
    ]

    if powers:
        chosen = powers[0]
        copies = ctx.potency
        for _ in range(copies):
            if len(ctx.state.hand) < 10:
                ctx.state.hand.append(chosen)
                ctx.state.card_costs[chosen] = 0


@potion_effect("ColorlessPotion")
def colorless_potion(ctx: PotionContext) -> None:
    """Colorless Potion: Discover a Colorless card (costs 0 this turn)."""
    from ..content.cards import ALL_CARDS, CardColor

    colorless = [cid for cid, card in ALL_CARDS.items() if card.color == CardColor.COLORLESS]

    if colorless:
        chosen = colorless[0]
        copies = ctx.potency
        for _ in range(copies):
            if len(ctx.state.hand) < 10:
                ctx.state.hand.append(chosen)
                ctx.state.card_costs[chosen] = 0


@potion_effect("SpeedPotion")
def speed_potion(ctx: PotionContext) -> None:
    """Speed Potion: Gain 5 temporary Dexterity (10 with Sacred Bark)."""
    ctx.apply_power_to_player("Dexterity", ctx.potency)
    ctx.apply_power_to_player("LoseDexterity", ctx.potency)


@potion_effect("SteroidPotion")
def steroid_potion(ctx: PotionContext) -> None:
    """Flex Potion: Gain 5 temporary Strength (10 with Sacred Bark)."""
    ctx.apply_power_to_player("Strength", ctx.potency)
    ctx.apply_power_to_player("LoseStrength", ctx.potency)


@potion_effect("BlessingOfTheForge")
def blessing_of_forge(ctx: PotionContext) -> None:
    """Blessing of the Forge: Upgrade all cards in hand for combat."""
    upgraded_hand = []
    for card_id in ctx.state.hand:
        if not card_id.endswith("+"):
            upgraded_hand.append(card_id + "+")
        else:
            upgraded_hand.append(card_id)
    ctx.state.hand = upgraded_hand


# Class-specific COMMON potions

@potion_effect("BloodPotion")
def blood_potion(ctx: PotionContext) -> None:
    """Blood Potion (Ironclad): Heal 20% of Max HP (40% with Sacred Bark)."""
    heal_amount = (ctx.player.max_hp * ctx.potency) // 100
    ctx.heal_player(heal_amount)


@potion_effect("Poison Potion", requires_target=True)
def poison_potion(ctx: PotionContext) -> None:
    """Poison Potion (Silent): Apply 6 Poison (12 with Sacred Bark)."""
    if ctx.target:
        ctx.apply_power(ctx.target, "Poison", ctx.potency)


@potion_effect("FocusPotion")
def focus_potion(ctx: PotionContext) -> None:
    """Focus Potion (Defect): Gain 2 Focus (4 with Sacred Bark)."""
    ctx.apply_power_to_player("Focus", ctx.potency)


@potion_effect("BottledMiracle")
def bottled_miracle(ctx: PotionContext) -> None:
    """Bottled Miracle (Watcher): Add 2 Miracles to hand (4 with Sacred Bark)."""
    for _ in range(ctx.potency):
        ctx.add_card_to_hand("Miracle")


# =============================================================================
# UNCOMMON POTIONS
# =============================================================================

@potion_effect("Ancient Potion")
def ancient_potion(ctx: PotionContext) -> None:
    """Ancient Potion: Gain 1 Artifact (2 with Sacred Bark)."""
    ctx.apply_power_to_player("Artifact", ctx.potency)


@potion_effect("Regen Potion")
def regen_potion(ctx: PotionContext) -> None:
    """Regeneration Potion: Gain 5 Regeneration (10 with Sacred Bark)."""
    ctx.apply_power_to_player("Regeneration", ctx.potency)


@potion_effect("GamblersBrew")
def gamblers_brew(ctx: PotionContext) -> None:
    """Gambler's Brew: Discard any number of cards, draw that many.
    For simulation, discard all and redraw."""
    hand_size = len(ctx.state.hand)
    # Move all cards from hand to discard
    ctx.state.discard_pile.extend(ctx.state.hand)
    ctx.state.hand.clear()
    # Draw same number
    ctx.draw_cards(hand_size)


@potion_effect("LiquidBronze")
def liquid_bronze(ctx: PotionContext) -> None:
    """Liquid Bronze: Gain 3 Thorns (6 with Sacred Bark)."""
    ctx.apply_power_to_player("Thorns", ctx.potency)


@potion_effect("LiquidMemories")
def liquid_memories(ctx: PotionContext) -> None:
    """Liquid Memories: Return card(s) from discard to hand (cost 0).
    With Sacred Bark, return 2 cards."""
    cards_to_return = ctx.potency
    for _ in range(cards_to_return):
        if ctx.state.discard_pile and len(ctx.state.hand) < 10:
            card = ctx.state.discard_pile.pop()
            ctx.state.hand.append(card)
            ctx.state.card_costs[card] = 0


@potion_effect("EssenceOfSteel")
def essence_of_steel(ctx: PotionContext) -> None:
    """Essence of Steel: Gain 4 Plated Armor (8 with Sacred Bark)."""
    ctx.apply_power_to_player("Plated Armor", ctx.potency)


@potion_effect("DuplicationPotion")
def duplication_potion(ctx: PotionContext) -> None:
    """Duplication Potion: Next card played twice (2 with Sacred Bark)."""
    ctx.apply_power_to_player("Duplication", ctx.potency)


@potion_effect("DistilledChaos")
def distilled_chaos(ctx: PotionContext) -> None:
    """Distilled Chaos: Play top 3 cards of draw pile (6 with Sacred Bark)."""
    engine = getattr(ctx.state, "_combat_engine_ref", None)
    if engine is not None and hasattr(engine, "play_top_cards_from_draw_pile"):
        played_cards = engine.play_top_cards_from_draw_pile(ctx.potency)
    else:
        # Fallback for direct registry calls without an active runtime engine.
        from ..combat_engine import CombatEngine
        fallback_engine = CombatEngine(ctx.state)
        played_cards = fallback_engine.play_top_cards_from_draw_pile(ctx.potency)

    ctx.result_data["played_cards"] = played_cards
    ctx.result_data["effects"] = [{"type": "play_top_cards", "amount": len(played_cards)}]


# Class-specific UNCOMMON potions

@potion_effect("ElixirPotion")
def elixir_potion(ctx: PotionContext) -> None:
    """Elixir (Ironclad): Exhaust any number of cards.
    For simulation, exhaust all cards in hand."""
    if ctx.state.hand:
        ctx.state.exhaust_pile.extend(ctx.state.hand)
        ctx.state.hand.clear()


@potion_effect("CunningPotion")
def cunning_potion(ctx: PotionContext) -> None:
    """Cunning Potion (Silent): Add 3 upgraded Shivs (6 with Sacred Bark)."""
    for _ in range(ctx.potency):
        ctx.add_card_to_hand("Shiv+")


@potion_effect("PotionOfCapacity")
def potion_of_capacity(ctx: PotionContext) -> None:
    """Potion of Capacity (Defect): Gain 2 Orb slots (4 with Sacred Bark)."""
    ctx.apply_power_to_player("OrbSlots", ctx.potency)


@potion_effect("StancePotion")
def stance_potion(ctx: PotionContext) -> None:
    """Stance Potion (Watcher): Enter Calm or Wrath.
    For simulation, enter Calm (generally safer)."""
    # Toggle between Calm/Wrath, defaulting to Calm
    if ctx.state.stance == "Calm":
        ctx.state.stance = "Wrath"
    else:
        ctx.state.stance = "Calm"


# =============================================================================
# RARE POTIONS
# =============================================================================

@potion_effect("CultistPotion")
def cultist_potion(ctx: PotionContext) -> None:
    """Cultist Potion: Gain 1 Ritual (2 with Sacred Bark).
    Ritual grants Strength at end of turn."""
    ctx.apply_power_to_player("Ritual", ctx.potency)


@potion_effect("Fruit Juice")
def fruit_juice(ctx: PotionContext) -> None:
    """Fruit Juice: Gain 5 Max HP (10 with Sacred Bark)."""
    ctx.player.max_hp += ctx.potency
    ctx.player.hp += ctx.potency  # Also heal that amount


@potion_effect("SneckoOil")
def snecko_oil(ctx: PotionContext) -> None:
    """Snecko Oil: Draw 5 cards and randomize all hand costs (0-3)."""
    import random
    from ..content.cards import get_card

    ctx.draw_cards(ctx.potency)
    rng = _get_rng(ctx.state, "card_random_rng", "card_rng")
    # Randomize all card costs in hand
    for card_id in ctx.state.hand:
        base_cost = None
        try:
            base = card_id.rstrip("+")
            upgraded = card_id.endswith("+")
            base_cost = get_card(base, upgraded).cost
        except Exception:
            base_cost = None

        # X-cost/unplayable/status costs are not randomized by the base game action.
        if base_cost is not None and base_cost < 0:
            continue

        if rng is not None:
            new_cost = rng.random(3)
        else:
            new_cost = random.randint(0, 3)

        # Preserve exact no-op behavior when cost does not change.
        if base_cost is not None and base_cost == new_cost:
            continue
        ctx.state.card_costs[card_id] = new_cost


@potion_effect("FairyPotion")
def fairy_potion(ctx: PotionContext) -> None:
    """Fairy in a Bottle: Auto-triggers on death, not manual use."""
    # This potion triggers automatically when player would die
    # Manual use does nothing
    return None


@potion_effect("SmokeBomb")
def smoke_bomb(ctx: PotionContext) -> None:
    """Smoke Bomb: Escape from non-boss combat."""
    has_back_attack = any(enemy.statuses.get("BackAttack", 0) > 0 for enemy in ctx.living_enemies)
    if (
        getattr(ctx.state, "is_boss_combat", False)
        or getattr(ctx.state, "cannot_escape", False)
        or has_back_attack
    ):
        ctx.state.escape_blocked_reason = "cannot_escape"
        ctx.result_data["success"] = False
        ctx.result_data["error"] = "Smoke Bomb cannot be used in this combat"
        return
    # Set combat end flag - handled by combat engine
    ctx.state.escaped = True


@potion_effect("EntropicBrew")
def entropic_brew(ctx: PotionContext) -> None:
    """Entropic Brew: Fill empty potion slots with random potions."""
    from ..generation.potions import get_potion_pool_for_class

    if ctx.has_relic("Sozu"):
        return

    player_class = str(getattr(ctx.state, "player_class", "WATCHER")).upper()
    available = list(get_potion_pool_for_class(player_class))

    if not available:
        return

    rng = _get_rng(ctx.state, "potion_rng")

    # Fill all empty slots
    for i, slot in enumerate(ctx.state.potions):
        if not slot:
            if rng is not None:
                idx = rng.random(len(available) - 1)
                ctx.state.potions[i] = available[idx]
            else:
                import random
                ctx.state.potions[i] = random.choice(available)


# Class-specific RARE potions

@potion_effect("HeartOfIron")
def heart_of_iron(ctx: PotionContext) -> None:
    """Heart of Iron (Ironclad): Gain 6 Metallicize (12 with Sacred Bark)."""
    ctx.apply_power_to_player("Metallicize", ctx.potency)


@potion_effect("GhostInAJar")
def ghost_in_jar(ctx: PotionContext) -> None:
    """Ghost In A Jar (Silent): Gain 1 Intangible (2 with Sacred Bark)."""
    ctx.apply_power_to_player("Intangible", ctx.potency)


@potion_effect("EssenceOfDarkness")
def essence_of_darkness(ctx: PotionContext) -> None:
    """Essence of Darkness (Defect): Channel Dark orbs (1 per slot).
    With Sacred Bark, channel 2 per slot."""
    orb_slots = ctx.player.statuses.get("OrbSlots", 3)
    dark_count = orb_slots * ctx.potency
    # Would need orb system - for now just track it
    ctx.apply_power_to_player("DarkOrbs", dark_count)


@potion_effect("Ambrosia")
def ambrosia(ctx: PotionContext) -> None:
    """Ambrosia (Watcher): Enter Divinity stance."""
    old_stance = ctx.state.stance

    # Exit Calm bonus if applicable (base 2, Violet Lotus adds +1 via relic trigger)
    if old_stance == "Calm":
        ctx.gain_energy(2)

    # Enter Divinity
    ctx.state.stance = "Divinity"
    ctx.gain_energy(3)  # Divinity grants +3 energy

    # Execute relic triggers for stance change (Violet Lotus)
    from . import execute_relic_triggers
    execute_relic_triggers("onChangeStance", ctx.state, {"new_stance": "Divinity", "old_stance": old_stance})

    # Trigger Mental Fortress if applicable
    mental_fortress = ctx.player.statuses.get("MentalFortress", 0)
    if mental_fortress > 0:
        ctx.gain_block(mental_fortress)
