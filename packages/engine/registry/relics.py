"""
Relic Trigger Implementations.

This module contains all relic trigger handlers using the registry pattern.
Each handler is registered via decorator and called during combat when
the corresponding hook is triggered.

Organized by trigger hook for easier maintenance.
"""

from __future__ import annotations

from . import relic_trigger, RelicContext
from .relic_factories import counter_relic


# =============================================================================
# AT_BATTLE_START Triggers
# =============================================================================

@relic_trigger("atBattleStart", relic="Akabeko")
def akabeko_start(ctx: RelicContext) -> None:
    """Akabeko: Gain 8 Vigor at combat start."""
    ctx.apply_power_to_player("Vigor", 8)


@relic_trigger("atBattleStart", relic="ClockworkSouvenir")
def clockwork_souvenir_start(ctx: RelicContext) -> None:
    """Clockwork Souvenir: Gain 1 Artifact at combat start."""
    ctx.apply_power_to_player("Artifact", 1)


@relic_trigger("atBattleStart", relic="CultistMask")
def cultist_mask_start(ctx: RelicContext) -> None:
    """Cultist Headpiece: Gain 1 Ritual at combat start."""
    ctx.apply_power_to_player("Ritual", 1)


@relic_trigger("atBattleStart", relic="Anchor")
def anchor_start(ctx: RelicContext) -> None:
    """Anchor: Gain 10 Block at combat start."""
    ctx.gain_block(10)


@relic_trigger("atBattleStart", relic="Bag of Marbles")
def bag_of_marbles_start(ctx: RelicContext) -> None:
    """Bag of Marbles: Apply 1 Vulnerable to ALL enemies."""
    ctx.apply_power_to_all_enemies("Vulnerable", 1)


@relic_trigger("atBattleStart", relic="Blood Vial")
def blood_vial_start(ctx: RelicContext) -> None:
    """Blood Vial: Heal 2 HP at combat start."""
    ctx.heal_player(2)


@relic_trigger("atBattleStart", relic="Bronze Scales")
def bronze_scales_start(ctx: RelicContext) -> None:
    """Bronze Scales: Gain 3 Thorns at combat start."""
    ctx.apply_power_to_player("Thorns", 3)


@relic_trigger("atBattleStart", relic="Centennial Puzzle")
def centennial_puzzle_start(ctx: RelicContext) -> None:
    """Centennial Puzzle: Reset counter at combat start."""
    ctx.set_relic_counter("Centennial Puzzle", 0)


@relic_trigger("atBattleStart", relic="Data Disk")
def data_disk_start(ctx: RelicContext) -> None:
    """Data Disk: Gain 1 Focus at combat start."""
    ctx.apply_power_to_player("Focus", 1)


@relic_trigger("atBattleStart", relic="Du-Vu Doll")
def duvu_doll_start(ctx: RelicContext) -> None:
    """Du-Vu Doll: Gain 1 Strength per Curse in deck at combat start."""
    from ..content.cards import ALL_CARDS, CardType
    # Count curse cards across all piles (matching Java which counts master deck)
    all_cards = (ctx.state.draw_pile + ctx.state.discard_pile +
                 ctx.state.hand + ctx.state.exhaust_pile)
    curse_count = 0
    for card_id in all_cards:
        if card_id in ALL_CARDS:
            card = ALL_CARDS[card_id]
            if card.card_type == CardType.CURSE:
                curse_count += 1

    if curse_count > 0:
        ctx.apply_power_to_player("Strength", curse_count)


@relic_trigger("atTurnStart", relic="Damaru")
def damaru_turn(ctx: RelicContext) -> None:
    """Damaru: Gain 1 Mantra at the start of each turn."""
    ctx.apply_power_to_player("Mantra", 1)


@relic_trigger("atTurnStart", relic="Warped Tongs")
def warped_tongs_turn_start(ctx: RelicContext) -> None:
    """Warped Tongs: At start of each turn, Upgrade a random card in your hand for the rest of combat."""
    import random
    if ctx.state.hand:
        idx = random.randint(0, len(ctx.state.hand) - 1)
        card_id = ctx.state.hand[idx]
        # Create temporary upgrade tracking
        if not hasattr(ctx.state, 'combat_upgrades'):
            ctx.state.combat_upgrades = set()
        if not card_id.endswith('+'):
            ctx.state.combat_upgrades.add(card_id)


@relic_trigger("atTurnStart", relic="Gold-Plated Cables")
def gold_plated_cables_turn_start(ctx: RelicContext) -> None:
    """Gold-Plated Cables: At start of turn, if 0 Block, trigger rightmost orb's passive."""
    if ctx.player.block == 0:
        # TODO: When orb system implemented, trigger rightmost orb passive
        if hasattr(ctx.state, 'orbs') and ctx.state.orbs:
            # Trigger passive of rightmost orb
            pass


@relic_trigger("atBattleStart", relic="FossilizedHelix")
def fossilized_helix_start(ctx: RelicContext) -> None:
    """Fossilized Helix: Gain 1 Buffer at combat start."""
    ctx.apply_power_to_player("Buffer", 1)


@relic_trigger("atBattleStart", relic="GremlinMask")
def gremlin_mask_start(ctx: RelicContext) -> None:
    """Gremlin Visage: Apply 1 Weak to self at combat start."""
    ctx.apply_power_to_player("Weakened", 1)


@relic_trigger("atBattleStart", relic="HornCleat")
def horn_cleat_init(ctx: RelicContext) -> None:
    """Horn Cleat: Initialize for turn 2 trigger."""
    ctx.set_relic_counter("HornCleat", 0)


@relic_trigger("atBattleStart", relic="Lantern")
def lantern_init(ctx: RelicContext) -> None:
    """Lantern: Reset counter for turn 1 energy."""
    ctx.set_relic_counter("Lantern", 0)


@relic_trigger("atBattleStart", relic="Mark of Pain")
def mark_of_pain_start(ctx: RelicContext) -> None:
    """Mark of Pain: Shuffle 2 Wounds into draw pile at combat start."""
    import random
    ctx.state.draw_pile.append("Wound")
    ctx.state.draw_pile.append("Wound")
    random.shuffle(ctx.state.draw_pile)


@relic_trigger("atBattleStart", relic="MutagenicStrength")
def mutagenic_strength_start(ctx: RelicContext) -> None:
    """Mutagenic Strength: Gain 3 Strength, lose 3 at end of turn."""
    ctx.apply_power_to_player("Strength", 3)
    ctx.apply_power_to_player("LoseStrength", 3)


@relic_trigger("atBattleStart", relic="Oddly Smooth Stone")
def oddly_smooth_stone_start(ctx: RelicContext) -> None:
    """Oddly Smooth Stone: Gain 1 Dexterity at combat start."""
    ctx.apply_power_to_player("Dexterity", 1)


@relic_trigger("atBattleStart", relic="Pantograph")
def pantograph_start(ctx: RelicContext) -> None:
    """Pantograph: Heal 25 HP in boss combats."""
    if ctx.state.combat_type == "boss":
        ctx.heal_player(25)


@relic_trigger("atBattleStart", relic="Pen Nib")
def pen_nib_init(ctx: RelicContext) -> None:
    """Pen Nib: Keep counter from run state if it exists."""
    # Counter is preserved from run state
    if ctx.get_relic_counter("Pen Nib", -1) < 0:
        ctx.set_relic_counter("Pen Nib", 0)


@relic_trigger("atBattleStart", relic="Philosopher's Stone")
def philosophers_stone_start(ctx: RelicContext) -> None:
    """Philosopher's Stone: ALL enemies gain 1 Strength at combat start."""
    for enemy in ctx.living_enemies:
        ctx.apply_power(enemy, "Strength", 1)


@relic_trigger("atBattleStart", relic="Red Mask")
def red_mask_start(ctx: RelicContext) -> None:
    """Red Mask: Apply 1 Weak to ALL enemies at combat start."""
    ctx.apply_power_to_all_enemies("Weakened", 1)


@relic_trigger("atBattleStart", relic="Red Skull")
def red_skull_init(ctx: RelicContext) -> None:
    """Red Skull: Check HP threshold and apply strength if needed."""
    is_bloodied = ctx.player.hp <= ctx.player.max_hp // 2
    if is_bloodied:
        ctx.apply_power_to_player("Strength", 3)
        ctx.set_relic_counter("Red Skull", 1)


@relic_trigger("atBattleStart", relic="Sling")
def sling_start(ctx: RelicContext) -> None:
    """Sling of Courage: Gain 2 Strength in Elite combats."""
    if ctx.state.combat_type == "elite":
        ctx.apply_power_to_player("Strength", 2)


@relic_trigger("atBattleStart", relic="Snecko Eye")
def snecko_eye_start(ctx: RelicContext) -> None:
    """Snecko Eye: Apply Confused power (randomize card costs)."""
    ctx.apply_power_to_player("Confused", 1)


@relic_trigger("atBattleStart", relic="TeardropLocket")
def teardrop_locket_enter(ctx: RelicContext) -> None:
    """Teardrop Locket: Enter Calm stance at combat start."""
    ctx.state.stance = "Calm"


@relic_trigger("atBattleStart", relic="Thread and Needle")
def thread_and_needle_start(ctx: RelicContext) -> None:
    """Thread and Needle: Gain 4 Plated Armor at combat start."""
    ctx.apply_power_to_player("Plated Armor", 4)


@relic_trigger("atBattleStart", relic="TwistedFunnel")
def twisted_funnel_start(ctx: RelicContext) -> None:
    """Twisted Funnel: Apply 4 Poison to ALL enemies at combat start."""
    ctx.apply_power_to_all_enemies("Poison", 4)


@relic_trigger("atBattleStart", relic="Vajra")
def vajra_start(ctx: RelicContext) -> None:
    """Vajra: Gain 1 Strength at combat start."""
    ctx.apply_power_to_player("Strength", 1)


@relic_trigger("atBattleStart", relic="Nuclear Battery")
def nuclear_battery_start(ctx: RelicContext) -> None:
    """Nuclear Battery: Channel 1 Plasma at combat start."""
    # TODO: When orb system implemented, channel Plasma
    if hasattr(ctx.state, 'orbs'):
        ctx.channel_orb("Plasma")


@relic_trigger("atBattleStart", relic="Symbiotic Virus")
def symbiotic_virus_start(ctx: RelicContext) -> None:
    """Symbiotic Virus: Channel 1 Dark at combat start."""
    # TODO: When orb system implemented, channel Dark
    if hasattr(ctx.state, 'orbs'):
        ctx.channel_orb("Dark")


@relic_trigger("atBattleStart", relic="Preserved Insect")
def preserved_insect_start(ctx: RelicContext) -> None:
    """Preserved Insect: Elites have 25% less HP."""
    for enemy in ctx.state.enemies:
        if hasattr(enemy, 'is_elite') and enemy.is_elite:
            # Reduce max HP by 25%
            original_max = enemy.max_hp
            new_max = int(original_max * 0.75)
            enemy.max_hp = new_max
            # Also reduce current HP proportionally if enemy is at full HP
            if enemy.hp == original_max:
                enemy.hp = new_max
            elif enemy.hp > new_max:
                enemy.hp = new_max


@relic_trigger("atBattleStart", relic="InkBottle")
def ink_bottle_init(ctx: RelicContext) -> None:
    """Ink Bottle: Keep counter from run state."""
    if ctx.get_relic_counter("InkBottle", -1) < 0:
        ctx.set_relic_counter("InkBottle", 0)


@relic_trigger("atBattleStart", relic="Happy Flower")
def happy_flower_init(ctx: RelicContext) -> None:
    """Happy Flower: Initialize counter."""
    if ctx.get_relic_counter("Happy Flower", -1) < 0:
        ctx.set_relic_counter("Happy Flower", 0)


@relic_trigger("atBattleStart", relic="Sundial")
def sundial_init(ctx: RelicContext) -> None:
    """Sundial: Initialize counter."""
    if ctx.get_relic_counter("Sundial", -1) < 0:
        ctx.set_relic_counter("Sundial", 0)


@relic_trigger("atBattleStart", relic="Nunchaku")
def nunchaku_init(ctx: RelicContext) -> None:
    """Nunchaku: Initialize counter."""
    if ctx.get_relic_counter("Nunchaku", -1) < 0:
        ctx.set_relic_counter("Nunchaku", 0)


@relic_trigger("atBattleStart", relic="Shuriken")
def shuriken_init(ctx: RelicContext) -> None:
    """Shuriken: Initialize counter for this combat."""
    ctx.set_relic_counter("Shuriken", 0)


@relic_trigger("atBattleStart", relic="Kunai")
def kunai_init(ctx: RelicContext) -> None:
    """Kunai: Initialize counter for this combat."""
    ctx.set_relic_counter("Kunai", 0)


@relic_trigger("atBattleStart", relic="Ornamental Fan")
def ornamental_fan_init(ctx: RelicContext) -> None:
    """Ornamental Fan: Initialize counter for this combat."""
    ctx.set_relic_counter("Ornamental Fan", 0)


@relic_trigger("atBattleStart", relic="Letter Opener")
def letter_opener_init(ctx: RelicContext) -> None:
    """Letter Opener: Initialize counter for this combat."""
    ctx.set_relic_counter("Letter Opener", 0)


@relic_trigger("atBattleStart", relic="CaptainsWheel")
def captains_wheel_init(ctx: RelicContext) -> None:
    """Captain's Wheel: Initialize counter at combat start."""
    ctx.set_relic_counter("CaptainsWheel", 0)


@relic_trigger("atBattleStart", relic="Incense Burner")
def incense_burner_init(ctx: RelicContext) -> None:
    """Incense Burner: Initialize counter at combat start if not set."""
    if ctx.get_relic_counter("Incense Burner", -1) < 0:
        ctx.set_relic_counter("Incense Burner", 0)


@relic_trigger("atBattleStart", relic="Inserter")
def inserter_init(ctx: RelicContext) -> None:
    """Inserter: Initialize counter at combat start if not set."""
    if ctx.get_relic_counter("Inserter", -1) < 0:
        ctx.set_relic_counter("Inserter", 0)


# =============================================================================
# AT_BATTLE_START_PRE_DRAW Triggers
# =============================================================================

@relic_trigger("atBattleStartPreDraw", relic="PureWater")
def pure_water_start(ctx: RelicContext) -> None:
    """Pure Water: Add Miracle to hand at combat start."""
    ctx.add_card_to_hand("Miracle")


@relic_trigger("atBattleStartPreDraw", relic="Bag of Preparation")
def bag_of_preparation_start(ctx: RelicContext) -> None:
    """Bag of Preparation: Draw 2 additional cards at combat start."""
    ctx.draw_cards(2)


@relic_trigger("atBattleStartPreDraw", relic="Gambling Chip")
def gambling_chip_init(ctx: RelicContext) -> None:
    """Gambling Chip: Initialize for turn 1 trigger."""
    ctx.set_relic_counter("Gambling Chip", 0)


@relic_trigger("atTurnStartPostDraw", relic="Gambling Chip")
def gambling_chip_turn_start(ctx: RelicContext) -> None:
    """Gambling Chip: On turn 1, discard hand and redraw."""
    activated = ctx.get_relic_counter("Gambling Chip", 0)
    if activated == 0:
        # Discard entire hand
        hand_size = len(ctx.state.hand)
        ctx.state.discard_pile.extend(ctx.state.hand)
        ctx.state.hand.clear()
        # Redraw same number of cards
        ctx.draw_cards(hand_size)
        ctx.set_relic_counter("Gambling Chip", 1)


@relic_trigger("atBattleStartPreDraw", relic="Enchiridion")
def enchiridion_start(ctx: RelicContext) -> None:
    """Enchiridion: Add a random Power card to hand (costs 0 this turn)."""
    from ..content.cards import ALL_CARDS, CardType
    import random
    powers = [cid for cid, card in ALL_CARDS.items() if card.card_type == CardType.POWER]
    if powers:
        chosen = random.choice(powers)
        ctx.add_card_to_hand(chosen)
        # Set the card to cost 0 for this turn
        if hasattr(ctx.state, 'card_costs'):
            ctx.state.card_costs[chosen] = 0


@relic_trigger("atBattleStartPreDraw", relic="HolyWater")
def holy_water_start(ctx: RelicContext) -> None:
    """Holy Water: Add 3 Miracles to hand at combat start."""
    for _ in range(3):
        ctx.add_card_to_hand("Miracle")


@relic_trigger("atBattleStartPreDraw", relic="Ninja Scroll")
def ninja_scroll_start(ctx: RelicContext) -> None:
    """Ninja Scroll: Add 3 Shivs to hand at combat start."""
    for _ in range(3):
        ctx.add_card_to_hand("Shiv")


@relic_trigger("atBattleStartPreDraw", relic="Ring of the Snake")
def ring_of_snake_start(ctx: RelicContext) -> None:
    """Ring of the Snake: Draw 2 additional cards at combat start."""
    ctx.draw_cards(2)


# =============================================================================
# AT_TURN_START Triggers
# =============================================================================

@relic_trigger("atTurnStart", relic="Ancient Tea Set")
def ancient_tea_set_turn(ctx: RelicContext) -> None:
    """Ancient Tea Set: Gain 2 Energy on turn 1 if counter is -2 (came from rest room)."""
    if ctx.state.turn == 1:
        counter = ctx.get_relic_counter("Ancient Tea Set", 0)
        if counter == -2:
            ctx.gain_energy(2)
            # Reset counter after use
            ctx.set_relic_counter("Ancient Tea Set", 0)


@relic_trigger("atTurnStart", relic="Art of War")
def art_of_war_turn(ctx: RelicContext) -> None:
    """Art of War: Gain 1 energy if no attacks played last turn."""
    flag = ctx.get_relic_counter("Art of War", 0)
    if flag == 0:  # 0 = no attacks last turn
        ctx.gain_energy(1)


@relic_trigger("atTurnStart", relic="HornCleat")
def horn_cleat_turn(ctx: RelicContext) -> None:
    """Horn Cleat: Gain 14 Block on turn 2."""
    if ctx.state.turn == 2:
        ctx.gain_block(14)


@relic_trigger("atTurnStart", relic="Lantern")
def lantern_turn(ctx: RelicContext) -> None:
    """Lantern: Gain 1 energy on turn 1 only."""
    if ctx.state.turn == 1:
        ctx.gain_energy(1)


# Happy Flower: Gain 1 energy every 3 turns
counter_relic(
    "atTurnStart", "Happy Flower", 3,
    lambda ctx: ctx.gain_energy(1)
)


@relic_trigger("atTurnStart", relic="Shuriken")
def shuriken_reset(ctx: RelicContext) -> None:
    """Shuriken: Reset counter at turn start."""
    ctx.set_relic_counter("Shuriken", 0)


@relic_trigger("atTurnStart", relic="Kunai")
def kunai_reset(ctx: RelicContext) -> None:
    """Kunai: Reset counter at turn start."""
    ctx.set_relic_counter("Kunai", 0)


@relic_trigger("atTurnStart", relic="Ornamental Fan")
def ornamental_fan_reset(ctx: RelicContext) -> None:
    """Ornamental Fan: Reset counter at turn start."""
    ctx.set_relic_counter("Ornamental Fan", 0)


@relic_trigger("atTurnStart", relic="Calipers")
def calipers_turn_start(ctx: RelicContext) -> None:
    """Calipers: Lose only 15 block at turn start instead of all block."""
    # Without Barricade or Blur, normally all block is lost
    # With Calipers, only lose 15 block
    has_barricade = ctx.player.statuses.get("Barricade", 0) > 0
    has_blur = ctx.player.statuses.get("Blur", 0) > 0

    if not has_barricade and not has_blur:
        if ctx.player.block > 15:
            ctx.player.block -= 15
        else:
            ctx.player.block = 0


@relic_trigger("atTurnStart", relic="Letter Opener")
def letter_opener_reset(ctx: RelicContext) -> None:
    """Letter Opener: Reset counter at turn start."""
    ctx.set_relic_counter("Letter Opener", 0)


@relic_trigger("atTurnStart", relic="Mercury Hourglass")
def mercury_hourglass_turn(ctx: RelicContext) -> None:
    """Mercury Hourglass: Deal 3 damage to ALL enemies at start of turn."""
    for enemy in ctx.living_enemies:
        blocked = min(enemy.block, 3)
        enemy.block -= blocked
        enemy.hp -= (3 - blocked)
        if enemy.hp < 0:
            enemy.hp = 0


@relic_trigger("atTurnStart", relic="Necronomicon")
def necronomicon_reset(ctx: RelicContext) -> None:
    """Necronomicon: Reset triggered flag at turn start."""
    ctx.set_relic_counter("Necronomicon", 0)


@relic_trigger("atTurnStart", relic="Velvet Choker")
def velvet_choker_reset(ctx: RelicContext) -> None:
    """Velvet Choker: Reset cards played counter at turn start."""
    ctx.set_relic_counter("Velvet Choker", 0)


@relic_trigger("atTurnStart", relic="OrangePellets")
def orange_pellets_reset(ctx: RelicContext) -> None:
    """Orange Pellets: Reset card type bitmask at turn start."""
    ctx.set_relic_counter("OrangePellets", 0)

@relic_trigger("atTurnStart", relic="Brimstone")
def brimstone_turn_start(ctx: RelicContext) -> None:
    """Brimstone: At turn start, gain 2 Strength, ALL enemies gain 1 Strength."""
    ctx.apply_power_to_player("Strength", 2)
    for enemy in ctx.living_enemies:
        ctx.apply_power(enemy, "Strength", 1)


@relic_trigger("atTurnStart", relic="SlaversCollar")
def slavers_collar_turn_start(ctx: RelicContext) -> None:
    """Slaver's Collar: Gain +1 Energy at start of turn in Elite/Boss fights."""
    if ctx.state.combat_type in ("elite", "boss"):
        ctx.gain_energy(1)


@relic_trigger("atTurnStart", relic="Incense Burner")
def incense_burner_turn_start(ctx: RelicContext) -> None:
    """Incense Burner: Every 6 turns gain 1 Intangible."""
    counter = ctx.get_relic_counter("Incense Burner", 0) + 1
    if counter >= 6:
        ctx.apply_power_to_player("Intangible", 1)
        counter = 0
    ctx.set_relic_counter("Incense Burner", counter)


@relic_trigger("atTurnStart", relic="CaptainsWheel")
def captains_wheel_turn_start(ctx: RelicContext) -> None:
    """Captain's Wheel: At start of turn 3, gain 18 Block (once per combat)."""
    counter = ctx.get_relic_counter("CaptainsWheel", 0) + 1
    ctx.set_relic_counter("CaptainsWheel", counter)
    if counter == 3:
        ctx.gain_block(18)


@relic_trigger("atTurnStart", relic="Inserter")
def inserter_turn_start(ctx: RelicContext) -> None:
    """Inserter: Every 2 turns, gain 1 Orb Slot (Defect only).

    Note: Orb system not yet implemented. This is a placeholder for future implementation.
    """
    counter = ctx.get_relic_counter("Inserter", 0) + 1
    ctx.set_relic_counter("Inserter", counter)
    if counter >= 2:
        # TODO: Implement orb slot increase when orb system is added
        # For now, just reset counter to track the trigger timing
        ctx.set_relic_counter("Inserter", 0)


# =============================================================================
# ON_PLAYER_END_TURN Triggers
# =============================================================================

@relic_trigger("onPlayerEndTurn", relic="Orichalcum")
def orichalcum_end_turn(ctx: RelicContext) -> None:
    """Orichalcum: Gain 6 Block if you have no Block."""
    if ctx.player.block == 0:
        ctx.gain_block(6)


@relic_trigger("onPlayerEndTurn", relic="Frozen Core")
def frozen_core_end_turn(ctx: RelicContext) -> None:
    """Frozen Core: At end of turn, if no empty orb slots, channel 1 Frost."""
    # TODO: When orb system implemented, check if slots are full and channel Frost
    if hasattr(ctx.state, 'orbs') and hasattr(ctx.state, 'max_orb_slots'):
        if len(ctx.state.orbs) >= ctx.state.max_orb_slots:
            ctx.channel_orb("Frost")


@relic_trigger("onPlayerEndTurn", relic="StoneCalendar")
def stone_calendar_end_turn(ctx: RelicContext) -> None:
    """Stone Calendar: Deal 52 damage to ALL enemies at end of turn 7."""
    if ctx.state.turn == 7:
        for enemy in ctx.living_enemies:
            blocked = min(enemy.block, 52)
            enemy.block -= blocked
            enemy.hp -= (52 - blocked)
            if enemy.hp < 0:
                enemy.hp = 0


@relic_trigger("onPlayerEndTurn", relic="Art of War")
def art_of_war_end_turn(ctx: RelicContext) -> None:
    """Art of War: Track if attacks were played this turn."""
    # Flag: 0 = no attacks, 1 = attacks played
    flag = 1 if ctx.state.attacks_played_this_turn > 0 else 0
    ctx.set_relic_counter("Art of War", flag)


@relic_trigger("onPlayerEndTurn", relic="Ice Cream")
def ice_cream_end_turn(ctx: RelicContext) -> None:
    """Ice Cream: Unused energy is conserved between turns."""
    # TODO: Actual implementation should:
    # 1. Track unused energy at end of turn
    # 2. Add it back at start of next turn
    # This is primarily a passive flag checked by energy system (see relics_passive.py)
    pass


@relic_trigger("atTurnStart", relic="Ice Cream")
def ice_cream_turn_start(ctx: RelicContext) -> None:
    """Ice Cream: Add conserved energy from previous turn."""
    # TODO: Actual implementation should:
    # 1. Restore conserved energy from previous turn
    # This is primarily a passive flag checked by energy system (see relics_passive.py)
    pass


@relic_trigger("onPlayerEndTurn", relic="CloakClasp")
def cloak_clasp_end(ctx: RelicContext) -> None:
    """Cloak Clasp: Gain 1 Block per card in hand."""
    ctx.gain_block(len(ctx.state.hand))


@relic_trigger("onPlayerEndTurn", relic="Pocketwatch")
def pocketwatch_end(ctx: RelicContext) -> None:
    """Pocketwatch: Draw 3 next turn if played 3 or fewer cards."""
    cards_played = getattr(ctx.state, 'cards_played_this_turn', 0)
    if cards_played <= 3:
        ctx.apply_power_to_player("Draw", 3)


@relic_trigger("onPlayerEndTurn", relic="Nilry's Codex")
def nilrys_codex_end(ctx: RelicContext) -> None:
    """Nilry's Codex: Add card choice for next turn (simplified: random)."""
    from ..content.cards import ALL_CARDS
    import random
    pool = list(ALL_CARDS.keys())[:100]  # Limit for performance
    if pool:
        chosen = random.choice(pool)
        if not hasattr(ctx.state, 'cards_to_add_next_turn') or ctx.state.cards_to_add_next_turn is None:
            ctx.state.cards_to_add_next_turn = []
        ctx.state.cards_to_add_next_turn.append(chosen)


# =============================================================================
# WAS_HP_LOST Triggers
# =============================================================================

@relic_trigger("wasHPLost", relic="Centennial Puzzle")
def centennial_puzzle_hp_lost(ctx: RelicContext) -> None:
    """Centennial Puzzle: Draw 3 cards first time you lose HP."""
    counter = ctx.get_relic_counter("Centennial Puzzle", 0)
    if counter == 0 and ctx.hp_lost > 0:
        ctx.draw_cards(3)
        ctx.set_relic_counter("Centennial Puzzle", 1)


@relic_trigger("wasHPLost", relic="Red Skull")
def red_skull_hp_lost(ctx: RelicContext) -> None:
    """Red Skull: Toggle strength based on HP threshold."""
    is_bloodied = ctx.player.hp <= ctx.player.max_hp // 2
    has_strength = ctx.get_relic_counter("Red Skull", 0) == 1

    if is_bloodied and not has_strength:
        ctx.apply_power_to_player("Strength", 3)
        ctx.set_relic_counter("Red Skull", 1)
    elif not is_bloodied and has_strength:
        # Lose strength (apply negative)
        current_str = ctx.player.statuses.get("Strength", 0)
        ctx.player.statuses["Strength"] = current_str - 3
        ctx.set_relic_counter("Red Skull", 0)


@relic_trigger("wasHPLost", relic="Runic Cube")
def runic_cube_hp_lost(ctx: RelicContext) -> None:
    """Runic Cube: Draw 1 card whenever you lose HP."""
    if ctx.hp_lost > 0:
        ctx.draw_cards(1)


@relic_trigger("wasHPLost", relic="Self Forming Clay")
def self_forming_clay_hp_lost(ctx: RelicContext) -> None:
    """Self-Forming Clay: Gain 3 Block next turn when losing HP."""
    if ctx.hp_lost > 0:
        current = ctx.player.statuses.get("NextTurnBlock", 0)
        ctx.apply_power_to_player("NextTurnBlock", 3)


@relic_trigger("wasHPLost", relic="Emotion Chip")
def emotion_chip_hp_lost(ctx: RelicContext) -> None:
    """Emotion Chip: Track that HP was lost to trigger orb passives next turn."""
    if ctx.hp_lost > 0:
        ctx.set_relic_counter("Emotion Chip", 1)  # Flag: will trigger next turn


@relic_trigger("atTurnStart", relic="Emotion Chip")
def emotion_chip_turn_start(ctx: RelicContext) -> None:
    """Emotion Chip: If HP was lost last turn, trigger all orb passives.

    Note: Orb system not fully implemented. This tracks the flag for future use.
    Java implementation: ImpulseAction triggers onStartOfTurn() and onEndOfTurn()
    for all orbs (and rightmost orb again if Cables relic is present).
    """
    if ctx.get_relic_counter("Emotion Chip", 0) == 1:
        # TODO: When orb system is implemented, trigger passive of all orbs here
        # For now, just clear the flag
        ctx.set_relic_counter("Emotion Chip", 0)


@relic_trigger("atBattleStart", relic="Emotion Chip")
def emotion_chip_battle_start(ctx: RelicContext) -> None:
    """Emotion Chip: Reset flag at combat start."""
    ctx.set_relic_counter("Emotion Chip", 0)


@relic_trigger("onVictory", relic="Emotion Chip")
def emotion_chip_victory(ctx: RelicContext) -> None:
    """Emotion Chip: Clear pulse flag at combat end (matching Java's onVictory)."""
    ctx.set_relic_counter("Emotion Chip", 0)


# =============================================================================
# ON_PLAY_CARD Triggers
# =============================================================================

# Shuriken: Gain 1 Strength after playing 3 Attacks
counter_relic(
    "onPlayCard", "Shuriken", 3,
    lambda ctx: ctx.apply_power_to_player("Strength", 1),
    card_type_filter="ATTACK"
)


# Kunai: Gain 1 Dexterity after playing 3 Attacks
counter_relic(
    "onPlayCard", "Kunai", 3,
    lambda ctx: ctx.apply_power_to_player("Dexterity", 1),
    card_type_filter="ATTACK"
)


# Nunchaku: Gain 1 energy after playing 10 Attacks
counter_relic(
    "onPlayCard", "Nunchaku", 10,
    lambda ctx: ctx.gain_energy(1),
    card_type_filter="ATTACK"
)


# Ornamental Fan: Gain 4 Block after playing 3 Attacks
counter_relic(
    "onPlayCard", "Ornamental Fan", 3,
    lambda ctx: ctx.gain_block(4),
    card_type_filter="ATTACK"
)


def _letter_opener_damage(ctx: RelicContext) -> None:
    """Helper: Deal 5 damage to ALL enemies."""
    for enemy in ctx.living_enemies:
        blocked = min(enemy.block, 5)
        enemy.block -= blocked
        enemy.hp -= (5 - blocked)
        if enemy.hp < 0:
            enemy.hp = 0

# Letter Opener: Deal 5 damage to ALL enemies after playing 3 Skills
counter_relic(
    "onPlayCard", "Letter Opener", 3,
    _letter_opener_damage,
    card_type_filter="SKILL"
)


# Ink Bottle: Draw 1 card after playing 10 cards
counter_relic(
    "onPlayCard", "InkBottle", 10,
    lambda ctx: ctx.draw_cards(1)
)


@relic_trigger("onPlayCard", relic="Bird Faced Urn")
def bird_faced_urn_on_play(ctx: RelicContext) -> None:
    """Bird-Faced Urn: Heal 2 HP when you play a Power."""
    if ctx.card and hasattr(ctx.card, 'card_type'):
        from ..content.cards import CardType
        if ctx.card.card_type == CardType.POWER:
            ctx.heal_player(2)


@relic_trigger("onPlayCard", relic="Mummified Hand")
def mummified_hand_on_play(ctx: RelicContext) -> None:
    """Mummified Hand: Reduce a random card's cost by 1 after playing a Power."""
    if ctx.card and hasattr(ctx.card, 'card_type'):
        from ..content.cards import CardType
        if ctx.card.card_type == CardType.POWER:
            if ctx.state.hand:
                import random
                idx = random.randint(0, len(ctx.state.hand) - 1)
                card_id = ctx.state.hand[idx]
                current_cost = ctx.state.card_costs.get(card_id, 1)
                ctx.state.card_costs[card_id] = max(0, current_cost - 1)


@relic_trigger("onPlayCard", relic="Yang")
def yang_on_play(ctx: RelicContext) -> None:
    """Duality (Yang): Gain 1 temporary Dexterity on Attack play."""
    if ctx.card and hasattr(ctx.card, 'card_type'):
        from ..content.cards import CardType
        if ctx.card.card_type == CardType.ATTACK:
            ctx.apply_power_to_player("Dexterity", 1)
            ctx.apply_power_to_player("LoseDexterity", 1)


@relic_trigger("onPlayCard", relic="Necronomicon")
def necronomicon_on_play(ctx: RelicContext) -> None:
    """Necronomicon: First 2+ cost Attack each turn plays twice."""
    if ctx.card and hasattr(ctx.card, 'card_type'):
        from ..content.cards import CardType
        if ctx.card.card_type == CardType.ATTACK:
            card_cost = ctx.state.card_costs.get(ctx.card.id, ctx.card.cost)
            triggered = ctx.get_relic_counter("Necronomicon", 0)
            if card_cost >= 2 and triggered == 0:
                ctx.set_relic_counter("Necronomicon", 1)
                # Mark for replay - actual replay handled by combat engine
                if not hasattr(ctx.state, 'cards_to_replay') or ctx.state.cards_to_replay is None:
                    ctx.state.cards_to_replay = []
                ctx.state.cards_to_replay.append(ctx.card.id)


@relic_trigger("onPlayCard", relic="Velvet Choker")
def velvet_choker_on_play(ctx: RelicContext) -> None:
    """Velvet Choker: Track cards played this turn."""
    counter = ctx.get_relic_counter("Velvet Choker", 0)
    ctx.set_relic_counter("Velvet Choker", counter + 1)


@relic_trigger("onPlayCard", relic="OrangePellets")
def orange_pellets_on_play(ctx: RelicContext) -> None:
    """Orange Pellets: Remove debuffs when Attack, Skill, Power played same turn."""
    if ctx.card and hasattr(ctx.card, 'card_type'):
        from ..content.cards import CardType
        counter = ctx.get_relic_counter("OrangePellets", 0)
        # Bitmask: 1=Attack, 2=Skill, 4=Power
        if ctx.card.card_type == CardType.ATTACK:
            counter |= 1
        elif ctx.card.card_type == CardType.SKILL:
            counter |= 2
        elif ctx.card.card_type == CardType.POWER:
            counter |= 4
        ctx.set_relic_counter("OrangePellets", counter)
        if counter == 7:  # All three types played
            # Remove all debuffs
            debuffs = ["Weakened", "Vulnerable", "Frail", "Choked", "Constricted"]
            for debuff in debuffs:
                if debuff in ctx.player.statuses:
                    del ctx.player.statuses[debuff]
            ctx.set_relic_counter("OrangePellets", 0)


@relic_trigger("onPlayCard", relic="Blue Candle")
def blue_candle_on_play(ctx: RelicContext) -> None:
    """Blue Candle: Curses can be played. Playing a curse exhausts it and deals 1 damage to you."""
    if ctx.card and hasattr(ctx.card, 'card_type'):
        from ..content.cards import CardType
        if ctx.card.card_type == CardType.CURSE:
            # TODO: Actual implementation should:
            # 1. Allow curse to be played (checked via passive flag in relics_passive.py)
            # 2. Exhaust the curse after playing
            # 3. Deal 1 damage to player
            pass


@relic_trigger("onPlayCard", relic="Medical Kit")
def medical_kit_on_play(ctx: RelicContext) -> None:
    """Medical Kit: Status cards can be played. Playing a status exhausts it."""
    if ctx.card and hasattr(ctx.card, 'card_type'):
        from ..content.cards import CardType
        if ctx.card.card_type == CardType.STATUS:
            # TODO: Actual implementation should:
            # 1. Allow status to be played (checked via passive flag in relics_passive.py)
            # 2. Exhaust the status after playing
            pass


# Pen Nib: Special case - uses two hooks (onPlayCard + atDamageGive)
# Not refactored to counter_relic because the action happens in atDamageGive, not onPlayCard
@relic_trigger("onPlayCard", relic="Pen Nib")
def pen_nib_on_play(ctx: RelicContext) -> None:
    """Pen Nib: Increment counter on attack play."""
    if ctx.card and hasattr(ctx.card, 'card_type'):
        from ..content.cards import CardType
        if ctx.card.card_type == CardType.ATTACK:
            counter = ctx.get_relic_counter("Pen Nib", 0)
            ctx.set_relic_counter("Pen Nib", counter + 1)


# =============================================================================
# ON_EXHAUST Triggers
# =============================================================================

@relic_trigger("onExhaust", relic="Charons Ashes")
def charons_ashes_exhaust(ctx: RelicContext) -> None:
    """Charon's Ashes: Deal 3 damage to ALL enemies when a card is exhausted."""
    for enemy in ctx.living_enemies:
        blocked = min(enemy.block, 3)
        enemy.block -= blocked
        enemy.hp -= (3 - blocked)
        if enemy.hp < 0:
            enemy.hp = 0


@relic_trigger("onExhaust", relic="Dead Branch")
def dead_branch_exhaust(ctx: RelicContext) -> None:
    """Dead Branch: Add a random card to hand when a card is exhausted."""
    from ..content.cards import ALL_CARDS
    import random

    # Get all non-curse, non-status cards
    pool = [cid for cid, card in ALL_CARDS.items()
            if card.card_type.value not in ("CURSE", "STATUS")]

    if pool:
        random_card = random.choice(pool)
        ctx.add_card_to_hand(random_card)


# =============================================================================
# ON_DEATH Triggers
# =============================================================================

@relic_trigger("onDeath", relic="Lizard Tail")
def lizard_tail_death(ctx: RelicContext) -> bool:
    """Lizard Tail: When you would die, heal to 50% max HP. Only works once per combat."""
    counter = ctx.get_relic_counter("Lizard Tail", 0)
    if counter == 0:
        # Revive player
        ctx.player.hp = ctx.player.max_hp // 2
        ctx.set_relic_counter("Lizard Tail", 1)  # Used up
        return True  # Death prevented
    return False  # Death not prevented


# =============================================================================
# ON_VICTORY Triggers
# =============================================================================

@relic_trigger("onVictory", relic="Meat on the Bone", priority=50)
def meat_on_bone_victory(ctx: RelicContext) -> None:
    """Meat on the Bone: Heal 12 HP if at 50% or less HP.

    Priority 50 ensures this triggers before other healing relics,
    matching Java behavior where onTrigger() is called before onVictory().
    """
    if ctx.player.hp <= ctx.player.max_hp // 2:
        ctx.heal_player(12)


@relic_trigger("onVictory", relic="Burning Blood")
def burning_blood_victory(ctx: RelicContext) -> None:
    """Burning Blood: Heal 6 HP at end of combat."""
    ctx.heal_player(6)


@relic_trigger("onVictory", relic="Black Blood")
def black_blood_victory(ctx: RelicContext) -> None:
    """Black Blood: Heal 12 HP at end of combat."""
    ctx.heal_player(12)


# =============================================================================
# ON_SHUFFLE Triggers
# =============================================================================

# Sundial: Gain 2 energy every 3 shuffles
counter_relic(
    "onShuffle", "Sundial", 3,
    lambda ctx: ctx.gain_energy(2)
)


@relic_trigger("onShuffle", relic="TheAbacus")
def abacus_shuffle(ctx: RelicContext) -> None:
    """The Abacus: Gain 6 Block when shuffling."""
    ctx.gain_block(6)


# =============================================================================
# ON_CHANGE_STANCE Triggers (Watcher)
# =============================================================================

@relic_trigger("onChangeStance", relic="VioletLotus")
def violet_lotus_stance_change(ctx: RelicContext) -> None:
    """Violet Lotus: Gain 1 additional Energy when exiting Calm stance."""
    from_stance = ctx.trigger_data.get("old_stance", "")
    if from_stance == "Calm":
        ctx.gain_energy(1)


@relic_trigger("onChangeStance", relic="Violet Lotus")
def violet_lotus_with_space_stance_change(ctx: RelicContext) -> None:
    """Violet Lotus: Gain 1 additional Energy when exiting Calm stance (name with space)."""
    from_stance = ctx.trigger_data.get("old_stance", "")
    if from_stance == "Calm":
        ctx.gain_energy(1)


# =============================================================================
# AT_DAMAGE_GIVE Triggers
# =============================================================================

@relic_trigger("atDamageGive", relic="Pen Nib")
def pen_nib_damage(ctx: RelicContext) -> int:
    """Pen Nib: Double damage on 10th attack, then reset counter."""
    counter = ctx.get_relic_counter("Pen Nib", 0)
    base_damage = ctx.trigger_data.get("value", ctx.damage)
    if counter >= 9:  # 10th attack triggers
        ctx.set_relic_counter("Pen Nib", 0)
        return base_damage * 2
    return base_damage


@relic_trigger("atDamageGive", relic="WristBlade")
def wrist_blade_damage(ctx: RelicContext) -> int:
    """Wrist Blade: 0-cost Attacks deal 4 additional damage."""
    damage = ctx.trigger_data.get("value", ctx.damage)
    if ctx.card:
        card_cost = ctx.state.card_costs.get(ctx.card.id, getattr(ctx.card, 'cost', 1))
        if card_cost == 0:
            return damage + 4
    return damage


@relic_trigger("atDamageGive", relic="StrikeDummy")
def strike_dummy_damage(ctx: RelicContext) -> int:
    """Strike Dummy: Cards containing 'Strike' deal 3 extra damage."""
    damage = ctx.trigger_data.get("value", ctx.damage)
    if ctx.card and hasattr(ctx.card, 'id'):
        if "Strike" in ctx.card.id:
            return damage + 3
    return damage


# =============================================================================
# AT_DAMAGE_FINAL_GIVE Triggers
# =============================================================================

@relic_trigger("atDamageFinalGive", relic="Boot")
def boot_damage(ctx: RelicContext) -> int:
    """The Boot: Attacks deal minimum 5 damage."""
    damage = ctx.trigger_data.get("value", ctx.damage)
    return max(damage, 5) if damage > 0 else damage


# =============================================================================
# ON_ATTACKED_TO_CHANGE_DAMAGE Triggers (Damage Receive)
# =============================================================================

@relic_trigger("onAttackedToChangeDamage", relic="Torii")
def torii_damage(ctx: RelicContext) -> int:
    """Torii: If receiving 2-5 unblocked damage, reduce to 1."""
    damage = ctx.trigger_data.get("value", ctx.damage)
    if 2 <= damage <= 5:
        return 1
    return damage


@relic_trigger("onAttackedToChangeDamage", relic="Paper Crane")
def paper_crane_damage(ctx: RelicContext) -> int:
    """Paper Crane: Weak enemies deal 40% damage (not 25%)."""
    # This modifies the weak effectiveness constant
    # The actual damage calculation should check for this relic
    # For now, return the damage as-is; actual logic is in damage calc
    damage = ctx.trigger_data.get("value", ctx.damage)
    return damage


# =============================================================================
# ON_LOSE_HP_LAST Triggers
# =============================================================================

@relic_trigger("onLoseHpLast", relic="TungstenRod")
def tungsten_rod_hp_loss(ctx: RelicContext) -> int:
    """Tungsten Rod: Reduce HP loss by 1."""
    hp_loss = ctx.trigger_data.get("value", ctx.hp_lost)
    return max(0, hp_loss - 1)


# =============================================================================
# ON_APPLY_POWER Triggers
# =============================================================================

@relic_trigger("onApplyPower", relic="Champion Belt")
def champion_belt_apply(ctx: RelicContext) -> None:
    """Champion Belt: Applying Vulnerable also applies 1 Weak."""
    power_id = ctx.trigger_data.get("power_id")
    target = ctx.trigger_data.get("target")
    if power_id == "Vulnerable" and target:
        ctx.apply_power(target, "Weakened", 1)


@relic_trigger("onApplyPower", relic="Snake Skull")
def snecko_skull_apply(ctx: RelicContext) -> int:
    """Snecko Skull: Apply 1 additional Poison."""
    power_id = ctx.trigger_data.get("power_id")
    amount = ctx.trigger_data.get("value", 0)
    if power_id == "Poison":
        return amount + 1
    return amount


# =============================================================================
# ON_BLOCK_BROKEN Triggers
# =============================================================================

@relic_trigger("onBlockBroken", relic="Hand Drill")
def hand_drill_block_broken(ctx: RelicContext) -> None:
    """Hand Drill: Apply 2 Vulnerable when enemy block is broken."""
    target = ctx.trigger_data.get("target")
    if target:
        ctx.apply_power(target, "Vulnerable", 2)


# =============================================================================
# ON_EXHAUST Triggers (Discard/Exhaust Related)
# =============================================================================

@relic_trigger("onExhaust", relic="Strange Spoon")
def strange_spoon_exhaust(ctx: RelicContext) -> None:
    """Strange Spoon: 50% chance exhausted card goes to discard instead."""
    import random
    card = ctx.trigger_data.get("card_id")
    if card and random.random() < 0.5:
        # Move from exhaust to discard
        if card in ctx.state.exhaust_pile:
            ctx.state.exhaust_pile.remove(card)
            ctx.state.discard_pile.append(card)


# =============================================================================
# ON_MANUAL_DISCARD Triggers
# =============================================================================

@relic_trigger("onManualDiscard", relic="Tingsha")
def tingsha_discard(ctx: RelicContext) -> None:
    """Tingsha: Deal 3 damage to random enemy when discarding."""
    import random
    if ctx.living_enemies:
        target = random.choice(ctx.living_enemies)
        blocked = min(target.block, 3)
        target.block -= blocked
        target.hp -= (3 - blocked)
        if target.hp < 0:
            target.hp = 0


@relic_trigger("onManualDiscard", relic="Tough Bandages")
def tough_bandages_discard(ctx: RelicContext) -> None:
    """Tough Bandages: Gain 3 Block when discarding."""
    ctx.gain_block(3)


@relic_trigger("onManualDiscard", relic="HoveringKite")
def hovering_kite_discard(ctx: RelicContext) -> None:
    """Hovering Kite: First discard each turn gives 1 Energy."""
    triggered = ctx.get_relic_counter("HoveringKite", 0)
    if triggered == 0:
        ctx.gain_energy(1)
        ctx.set_relic_counter("HoveringKite", 1)


@relic_trigger("atTurnStart", relic="HoveringKite")
def hovering_kite_reset(ctx: RelicContext) -> None:
    """Hovering Kite: Reset trigger for new turn."""
    ctx.set_relic_counter("HoveringKite", 0)


# =============================================================================
# EMPTY HAND Triggers (Unceasing Top)
# =============================================================================

@relic_trigger("onEmptyHand", relic="Unceasing Top")
def unceasing_top_empty_hand(ctx: RelicContext) -> None:
    """Unceasing Top: Draw a card if hand becomes empty."""
    # Only draw if hand is actually empty
    if len(ctx.state.hand) == 0:
        ctx.draw_cards(1)


# =============================================================================
# ON_MONSTER_DEATH Triggers
# =============================================================================

@relic_trigger("onMonsterDeath", relic="Gremlin Horn")
def gremlin_horn_death(ctx: RelicContext) -> None:
    """Gremlin Horn: Gain 1 Energy and draw 1 card when enemy dies."""
    ctx.gain_energy(1)
    ctx.draw_cards(1)


@relic_trigger("onMonsterDeath", relic="The Specimen")
def specimen_death(ctx: RelicContext) -> None:
    """The Specimen: Transfer Poison to random enemy when poisoned enemy dies."""
    dead_enemy = ctx.trigger_data.get("enemy")
    if dead_enemy and dead_enemy.statuses.get("Poison", 0) > 0:
        poison = dead_enemy.statuses["Poison"]
        living = ctx.living_enemies
        if living:
            import random
            target = random.choice(living)
            ctx.apply_power(target, "Poison", poison)


# =============================================================================
# ON_GAIN_GOLD Triggers
# =============================================================================

@relic_trigger("onGainGold", relic="Bloody Idol")
def bloody_idol_gain_gold(ctx: RelicContext) -> None:
    """Bloody Idol: Heal 5 HP whenever you gain gold."""
    ctx.heal_player(5)


# =============================================================================
# ON_ENTER_ROOM Triggers
# =============================================================================

@relic_trigger("onEnterRoom", relic="Maw Bank")
def maw_bank_enter_room(ctx: RelicContext) -> None:
    """Maw Bank: Gain 12 gold when entering a non-shop room (deactivates if you spend gold at a shop)."""
    room_type = ctx.trigger_data.get("room_type", "")
    # Maw Bank deactivates if used in shop - check counter: 0 = active, -2 = deactivated
    counter = ctx.get_relic_counter("Maw Bank", 0)
    if counter != -2 and room_type != "SHOP":
        if hasattr(ctx.state, 'gold'):
            ctx.state.gold += 12


@relic_trigger("onEnterRoom", relic="Meal Ticket")
def meal_ticket_enter_room(ctx: RelicContext) -> None:
    """Meal Ticket: Heal 15 HP when entering a shop."""
    room_type = ctx.trigger_data.get("room_type", "")
    if room_type == "SHOP":
        ctx.heal_player(15)


@relic_trigger("onEnterRoom", relic="Ssserpent Head")
def ssserpent_head_enter_room(ctx: RelicContext) -> None:
    """Ssserpent Head: Gain 50 gold when entering a ? room."""
    room_type = ctx.trigger_data.get("room_type", "")
    if room_type == "EVENT":
        if hasattr(ctx.state, 'gold'):
            ctx.state.gold += 50


# =============================================================================
# ON_OBTAIN_CARD Triggers
# =============================================================================

@relic_trigger("onObtainCard", relic="Ceramic Fish")
def ceramic_fish_obtain(ctx: RelicContext) -> None:
    """Ceramic Fish: Gain 9 Gold when adding a card."""
    if hasattr(ctx.state, 'gold'):
        ctx.state.gold += 9


@relic_trigger("onObtainCard", relic="Frozen Egg 2")
def frozen_egg_obtain(ctx: RelicContext) -> str:
    """Frozen Egg: Powers are automatically upgraded."""
    card_id = ctx.trigger_data.get("card_id", "")
    from ..content.cards import ALL_CARDS, CardType
    if card_id in ALL_CARDS:
        card = ALL_CARDS[card_id]
        if card.card_type == CardType.POWER and not card_id.endswith("+"):
            return card_id + "+"
    return card_id


@relic_trigger("onObtainCard", relic="Molten Egg 2")
def molten_egg_obtain(ctx: RelicContext) -> str:
    """Molten Egg: Attacks are automatically upgraded."""
    card_id = ctx.trigger_data.get("card_id", "")
    from ..content.cards import ALL_CARDS, CardType
    if card_id in ALL_CARDS:
        card = ALL_CARDS[card_id]
        if card.card_type == CardType.ATTACK and not card_id.endswith("+"):
            return card_id + "+"
    return card_id


@relic_trigger("onObtainCard", relic="Toxic Egg 2")
def toxic_egg_obtain(ctx: RelicContext) -> str:
    """Toxic Egg: Skills are automatically upgraded."""
    card_id = ctx.trigger_data.get("card_id", "")
    from ..content.cards import ALL_CARDS, CardType
    if card_id in ALL_CARDS:
        card = ALL_CARDS[card_id]
        if card.card_type == CardType.SKILL and not card_id.endswith("+"):
            return card_id + "+"
    return card_id


@relic_trigger("onObtainCard", relic="Darkstone Periapt")
def darkstone_obtain(ctx: RelicContext) -> None:
    """Darkstone Periapt: Gain 6 Max HP when obtaining a Curse."""
    card_id = ctx.trigger_data.get("card_id", "")
    from ..content.cards import ALL_CARDS, CardType
    if card_id in ALL_CARDS:
        card = ALL_CARDS[card_id]
        if card.card_type == CardType.CURSE:
            ctx.player.max_hp += 6
            ctx.player.hp += 6


# =============================================================================
# ON_VICTORY Triggers (Additional)
# =============================================================================

@relic_trigger("onVictory", relic="Face Of Cleric")
def face_of_cleric_victory(ctx: RelicContext) -> None:
    """Face of Cleric: Gain 1 Max HP after combat (and heal 1 HP).

    Note: increaseMaxHp(1, true) in Java heals the new HP as well.
    """
    ctx.player.max_hp += 1
    ctx.player.hp += 1


# =============================================================================
# ON_USE_POTION Triggers
# =============================================================================

@relic_trigger("onUsePotion", relic="Toy Ornithopter")
def toy_ornithopter_potion(ctx: RelicContext) -> None:
    """Toy Ornithopter: Heal 5 HP when using a potion."""
    ctx.heal_player(5)


# =============================================================================
# ON_PLAYER_HEAL Triggers
# =============================================================================

@relic_trigger("onPlayerHeal", relic="Magic Flower")
def magic_flower_heal(ctx: RelicContext) -> int:
    """Magic Flower: Healing is 50% more effective during combat.

    Returns the modified heal amount.
    Java implementation: MathUtils.round((float)healAmount * 1.5f)
    Only applies during combat (checks RoomPhase.COMBAT).
    """
    heal_amount = ctx.trigger_data.get("heal_amount", 0)
    # Round to nearest int (matching Java's MathUtils.round)
    return round(heal_amount * 1.5)


# =============================================================================
# ON_EQUIP Triggers (Fruit Relics)
# =============================================================================

@relic_trigger("onEquip", relic="Pear")
def pear_equip(ctx: RelicContext) -> None:
    """Pear: Increase max HP by 10 when obtained."""
    ctx.player.max_hp += 10
    ctx.player.hp += 10


@relic_trigger("onEquip", relic="Mango")
def mango_equip(ctx: RelicContext) -> None:
    """Mango: Increase max HP by 14 when obtained."""
    ctx.player.max_hp += 14
    ctx.player.hp += 14


@relic_trigger("onEquip", relic="Strawberry")
def strawberry_equip(ctx: RelicContext) -> None:
    """Strawberry: Increase max HP by 7 when obtained."""
    ctx.player.max_hp += 7
    ctx.player.hp += 7


@relic_trigger("onEquip", relic="War Paint")
def war_paint_equip(ctx: RelicContext) -> None:
    """War Paint: Upon pickup, upgrade 2 random Skills in your deck."""
    from ..content.cards import ALL_CARDS, CardType
    import random

    # Check if deck is available (RunState context)
    if not hasattr(ctx.state, 'deck'):
        return

    # Find all skills that can be upgraded
    skills = []
    for i, card_inst in enumerate(ctx.state.deck):
        card_id = card_inst.id if hasattr(card_inst, 'id') else card_inst
        if card_id in ALL_CARDS and not card_inst.upgraded:
            if ALL_CARDS[card_id].card_type == CardType.SKILL:
                skills.append(i)

    # Upgrade 2 random ones
    if skills:
        random.shuffle(skills)
        for idx in skills[:2]:
            ctx.state.deck[idx].upgraded = True


@relic_trigger("onEquip", relic="Whetstone")
def whetstone_equip(ctx: RelicContext) -> None:
    """Whetstone: Upon pickup, upgrade 2 random Attacks in your deck."""
    from ..content.cards import ALL_CARDS, CardType
    import random

    # Check if deck is available (RunState context)
    if not hasattr(ctx.state, 'deck'):
        return

    # Find all attacks that can be upgraded
    attacks = []
    for i, card_inst in enumerate(ctx.state.deck):
        card_id = card_inst.id if hasattr(card_inst, 'id') else card_inst
        if card_id in ALL_CARDS and not card_inst.upgraded:
            if ALL_CARDS[card_id].card_type == CardType.ATTACK:
                attacks.append(i)

    # Upgrade 2 random ones
    if attacks:
        random.shuffle(attacks)
        for idx in attacks[:2]:
            ctx.state.deck[idx].upgraded = True


@relic_trigger("onEquip", relic="BustedCrown")
def busted_crown_equip(ctx: RelicContext) -> None:
    """Busted Crown: Gain +8 Max HP when obtained."""
    ctx.player.max_hp += 8
    ctx.player.hp += 8


@relic_trigger("onEquip", relic="Old Coin")
def old_coin_equip(ctx: RelicContext) -> None:
    """Old Coin: Gain 300 Gold when obtained."""
    if hasattr(ctx.state, 'gold'):
        ctx.state.gold += 300


@relic_trigger("onEquip", relic="Lee's Waffle")
def lees_waffle_equip(ctx: RelicContext) -> None:
    """Lee's Waffle: Gain 7 Max HP and fully heal."""
    ctx.player.max_hp += 7
    ctx.player.hp = ctx.player.max_hp


@relic_trigger("onEquip", relic="Tiny Chest")
def tiny_chest_equip(ctx: RelicContext) -> None:
    """Tiny Chest: Initialize counter at 0."""
    ctx.set_relic_counter("Tiny Chest", 0)


@relic_trigger("onEquip", relic="Matryoshka")
def matryoshka_equip(ctx: RelicContext) -> None:
    """Matryoshka: Next 2 non-boss chests have an additional relic."""
    ctx.set_relic_counter("Matryoshka", 2)


@relic_trigger("onEquip", relic="Omamori")
def omamori_equip(ctx: RelicContext) -> None:
    """Omamori: Negate the next 2 Curses you would obtain."""
    ctx.set_relic_counter("Omamori", 2)


@relic_trigger("onObtainCard", relic="Omamori")
def omamori_obtain_card(ctx: RelicContext) -> str:
    """Omamori: Negate curse if counter > 0."""
    card_id = ctx.trigger_data.get("card_id", "")
    from ..content.cards import ALL_CARDS, CardType

    if card_id in ALL_CARDS:
        card = ALL_CARDS[card_id]
        if card.card_type == CardType.CURSE:
            counter = ctx.get_relic_counter("Omamori", 0)
            if counter > 0:
                ctx.set_relic_counter("Omamori", counter - 1)
                return ""  # Empty string means card is negated
    return card_id  # Card not negated


# =============================================================================
# REST SITE & EVENT RELICS
# =============================================================================

@relic_trigger("onEquip", relic="Girya")
def girya_equip(ctx: RelicContext) -> None:
    """Girya: Can lift 3 times total."""
    ctx.set_relic_counter("Girya", 3)


@relic_trigger("onRestOption", relic="Girya")
def girya_lift(ctx: RelicContext) -> None:
    """Girya: Using lift grants +1 Strength permanently."""
    option = ctx.trigger_data.get("option", "")
    if option == "lift":
        uses = ctx.get_relic_counter("Girya", 0)
        if uses > 0:
            # Grant permanent Strength (stored in run state, not combat state)
            if hasattr(ctx.state, 'permanent_strength'):
                ctx.state.permanent_strength = ctx.state.permanent_strength + 1
            else:
                ctx.state.permanent_strength = 1
            ctx.set_relic_counter("Girya", uses - 1)


@relic_trigger("onEquip", relic="Shovel")
def shovel_equip(ctx: RelicContext) -> None:
    """Shovel: Can dig once."""
    ctx.set_relic_counter("Shovel", 1)


@relic_trigger("onEquip", relic="Wing Boots")
def wing_boots_equip(ctx: RelicContext) -> None:
    """Wing Boots: Can fly 3 times."""
    ctx.set_relic_counter("Wing Boots", 3)


@relic_trigger("onFly", relic="Wing Boots")
def wing_boots_fly(ctx: RelicContext) -> bool:
    """Wing Boots: Use one charge to fly."""
    uses = ctx.get_relic_counter("Wing Boots", 0)
    if uses > 0:
        ctx.set_relic_counter("Wing Boots", uses - 1)
        return True
    return False


@relic_trigger("onEquip", relic="N'loth's Gift")
def nloths_gift_equip(ctx: RelicContext) -> None:
    """N'loth's Gift: Double next event rewards."""
    ctx.set_relic_counter("N'loth's Gift", 1)


@relic_trigger("onGainGold", relic="Golden Idol")
def golden_idol_gold(ctx: RelicContext) -> int:
    """Golden Idol: Gain 25% more gold."""
    gold_amount = ctx.trigger_data.get("amount", 0)
    bonus = gold_amount // 4  # 25% bonus
    return gold_amount + bonus


# =============================================================================
# BOTTLED RELICS
# =============================================================================

@relic_trigger("onEquip", relic="Bottled Flame")
def bottled_flame_equip(ctx: RelicContext) -> None:
    """Bottled Flame: Choose an Attack to become Innate."""
    # Set flag that player needs to choose an Attack
    ctx.set_relic_counter("Bottled Flame", -2)  # -2 = needs selection
    # Actual selection would be handled by UI/game flow


@relic_trigger("onEquip", relic="Bottled Lightning")
def bottled_lightning_equip(ctx: RelicContext) -> None:
    """Bottled Lightning: Choose a Skill to become Innate."""
    ctx.set_relic_counter("Bottled Lightning", -2)


@relic_trigger("onEquip", relic="Bottled Tornado")
def bottled_tornado_equip(ctx: RelicContext) -> None:
    """Bottled Tornado: Choose a Power to become Innate."""
    ctx.set_relic_counter("Bottled Tornado", -2)
