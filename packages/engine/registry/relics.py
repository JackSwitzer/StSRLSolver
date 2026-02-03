"""
Relic Trigger Implementations.

This module contains all relic trigger handlers using the registry pattern.
Each handler is registered via decorator and called during combat when
the corresponding hook is triggered.

Organized by trigger hook for easier maintenance.
"""

from __future__ import annotations

from . import relic_trigger, RelicContext


# =============================================================================
# AT_BATTLE_START Triggers
# =============================================================================

@relic_trigger("atBattleStart", relic="Akabeko")
def akabeko_start(ctx: RelicContext) -> None:
    """Akabeko: Gain 8 Vigor at combat start."""
    ctx.apply_power_to_player("Vigor", 8)


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


@relic_trigger("atBattleStart", relic="Damaru")
def damaru_start(ctx: RelicContext) -> None:
    """Damaru: Gain 1 Mantra at combat start."""
    ctx.apply_power_to_player("Mantra", 1)


@relic_trigger("atBattleStart", relic="FossilizedHelix")
def fossilized_helix_start(ctx: RelicContext) -> None:
    """Fossilized Helix: Gain 1 Buffer at combat start."""
    ctx.apply_power_to_player("Buffer", 1)


@relic_trigger("atBattleStart", relic="Ginger")
def ginger_start(ctx: RelicContext) -> None:
    """Ginger: Gain immunity to Weak (handled elsewhere, but mark it)."""
    # This is actually handled in the debuff application logic
    pass


@relic_trigger("atBattleStart", relic="HornCleat")
def horn_cleat_init(ctx: RelicContext) -> None:
    """Horn Cleat: Initialize for turn 2 trigger."""
    ctx.set_relic_counter("HornCleat", 0)


@relic_trigger("atBattleStart", relic="Lantern")
def lantern_init(ctx: RelicContext) -> None:
    """Lantern: Reset counter for turn 1 energy."""
    ctx.set_relic_counter("Lantern", 0)


@relic_trigger("atBattleStart", relic="Oddly Smooth Stone")
def oddly_smooth_stone_start(ctx: RelicContext) -> None:
    """Oddly Smooth Stone: Gain 1 Dexterity at combat start."""
    ctx.apply_power_to_player("Dexterity", 1)


@relic_trigger("atBattleStart", relic="Orichalcum")
def orichalcum_init(ctx: RelicContext) -> None:
    """Orichalcum: Initialize relic state."""
    pass  # Main logic in onPlayerEndTurn


@relic_trigger("atBattleStart", relic="Pen Nib")
def pen_nib_init(ctx: RelicContext) -> None:
    """Pen Nib: Keep counter from run state if it exists."""
    # Counter is preserved from run state
    if ctx.get_relic_counter("Pen Nib", -1) < 0:
        ctx.set_relic_counter("Pen Nib", 0)


@relic_trigger("atBattleStart", relic="PreservedInsect")
def preserved_insect_start(ctx: RelicContext) -> None:
    """Preserved Insect: Enemies have 25% less HP in Elite rooms."""
    # This is handled during enemy HP initialization
    pass


@relic_trigger("atBattleStart", relic="Red Skull")
def red_skull_init(ctx: RelicContext) -> None:
    """Red Skull: Check HP threshold and apply strength if needed."""
    is_bloodied = ctx.player.hp <= ctx.player.max_hp // 2
    if is_bloodied:
        ctx.apply_power_to_player("Strength", 3)
        ctx.set_relic_counter("Red Skull", 1)


@relic_trigger("atBattleStart", relic="SnakeSkull")
def snake_skull_start(ctx: RelicContext) -> None:
    """Snake Skull: Gain 1 Poison at combat start (Silent starter relic effect)."""
    # Actually applies poison to enemies when they have no poison
    pass


@relic_trigger("atBattleStart", relic="Thread and Needle")
def thread_and_needle_start(ctx: RelicContext) -> None:
    """Thread and Needle: Gain 4 Plated Armor at combat start."""
    ctx.apply_power_to_player("Plated Armor", 4)


@relic_trigger("atBattleStart", relic="Turnip")
def turnip_start(ctx: RelicContext) -> None:
    """Turnip: Gain immunity to Frail (handled elsewhere)."""
    pass


@relic_trigger("atBattleStart", relic="Vajra")
def vajra_start(ctx: RelicContext) -> None:
    """Vajra: Gain 1 Strength at combat start."""
    ctx.apply_power_to_player("Strength", 1)


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


# =============================================================================
# AT_BATTLE_START_PRE_DRAW Triggers
# =============================================================================

@relic_trigger("atBattleStartPreDraw", relic="PureWater")
def pure_water_start(ctx: RelicContext) -> None:
    """Pure Water: Add Miracle to hand at combat start."""
    ctx.add_card_to_hand("Miracle")


# =============================================================================
# AT_TURN_START Triggers
# =============================================================================

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


@relic_trigger("atTurnStart", relic="Happy Flower")
def happy_flower_turn(ctx: RelicContext) -> None:
    """Happy Flower: Gain 1 energy every 3 turns."""
    counter = ctx.get_relic_counter("Happy Flower", 0)
    counter += 1
    if counter >= 3:
        ctx.gain_energy(1)
        counter = 0
    ctx.set_relic_counter("Happy Flower", counter)


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


@relic_trigger("atTurnStart", relic="Letter Opener")
def letter_opener_reset(ctx: RelicContext) -> None:
    """Letter Opener: Reset counter at turn start."""
    ctx.set_relic_counter("Letter Opener", 0)


# =============================================================================
# ON_PLAYER_END_TURN Triggers
# =============================================================================

@relic_trigger("onPlayerEndTurn", relic="Orichalcum")
def orichalcum_end_turn(ctx: RelicContext) -> None:
    """Orichalcum: Gain 6 Block if you have no Block."""
    if ctx.player.block == 0:
        ctx.gain_block(6)


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


# =============================================================================
# ON_PLAY_CARD Triggers
# =============================================================================

@relic_trigger("onPlayCard", relic="Shuriken")
def shuriken_on_play(ctx: RelicContext) -> None:
    """Shuriken: Gain 1 Strength after playing 3 Attacks."""
    if ctx.card and hasattr(ctx.card, 'card_type'):
        from ..content.cards import CardType
        if ctx.card.card_type == CardType.ATTACK:
            counter = ctx.get_relic_counter("Shuriken", 0)
            counter += 1
            if counter >= 3:
                ctx.apply_power_to_player("Strength", 1)
                counter = 0
            ctx.set_relic_counter("Shuriken", counter)


@relic_trigger("onPlayCard", relic="Kunai")
def kunai_on_play(ctx: RelicContext) -> None:
    """Kunai: Gain 1 Dexterity after playing 3 Attacks."""
    if ctx.card and hasattr(ctx.card, 'card_type'):
        from ..content.cards import CardType
        if ctx.card.card_type == CardType.ATTACK:
            counter = ctx.get_relic_counter("Kunai", 0)
            counter += 1
            if counter >= 3:
                ctx.apply_power_to_player("Dexterity", 1)
                counter = 0
            ctx.set_relic_counter("Kunai", counter)


@relic_trigger("onPlayCard", relic="Nunchaku")
def nunchaku_on_play(ctx: RelicContext) -> None:
    """Nunchaku: Gain 1 energy after playing 10 Attacks."""
    if ctx.card and hasattr(ctx.card, 'card_type'):
        from ..content.cards import CardType
        if ctx.card.card_type == CardType.ATTACK:
            counter = ctx.get_relic_counter("Nunchaku", 0)
            counter += 1
            if counter >= 10:
                ctx.gain_energy(1)
                counter = 0
            ctx.set_relic_counter("Nunchaku", counter)


@relic_trigger("onPlayCard", relic="Ornamental Fan")
def ornamental_fan_on_play(ctx: RelicContext) -> None:
    """Ornamental Fan: Gain 4 Block after playing 3 Attacks."""
    if ctx.card and hasattr(ctx.card, 'card_type'):
        from ..content.cards import CardType
        if ctx.card.card_type == CardType.ATTACK:
            counter = ctx.get_relic_counter("Ornamental Fan", 0)
            counter += 1
            if counter >= 3:
                ctx.gain_block(4)
                counter = 0
            ctx.set_relic_counter("Ornamental Fan", counter)


@relic_trigger("onPlayCard", relic="Letter Opener")
def letter_opener_on_play(ctx: RelicContext) -> None:
    """Letter Opener: Deal 5 damage to ALL enemies after playing 3 Skills."""
    if ctx.card and hasattr(ctx.card, 'card_type'):
        from ..content.cards import CardType
        if ctx.card.card_type == CardType.SKILL:
            counter = ctx.get_relic_counter("Letter Opener", 0)
            counter += 1
            if counter >= 3:
                for enemy in ctx.living_enemies:
                    blocked = min(enemy.block, 5)
                    enemy.block -= blocked
                    enemy.hp -= (5 - blocked)
                    if enemy.hp < 0:
                        enemy.hp = 0
                counter = 0
            ctx.set_relic_counter("Letter Opener", counter)


@relic_trigger("onPlayCard", relic="InkBottle")
def ink_bottle_on_play(ctx: RelicContext) -> None:
    """Ink Bottle: Draw 1 card after playing 10 cards."""
    counter = ctx.get_relic_counter("InkBottle", 0)
    counter += 1
    if counter >= 10:
        ctx.draw_cards(1)
        counter = 0
    ctx.set_relic_counter("InkBottle", counter)


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
# ON_VICTORY Triggers
# =============================================================================

@relic_trigger("onVictory", relic="Burning Blood")
def burning_blood_victory(ctx: RelicContext) -> None:
    """Burning Blood: Heal 6 HP at end of combat."""
    ctx.heal_player(6)


@relic_trigger("onVictory", relic="Black Blood")
def black_blood_victory(ctx: RelicContext) -> None:
    """Black Blood: Heal 12 HP at end of combat."""
    ctx.heal_player(12)


@relic_trigger("onVictory", relic="Blood Vial")
def blood_vial_victory(ctx: RelicContext) -> None:
    """Blood Vial: Already healed at battle start, no victory effect."""
    pass


@relic_trigger("onVictory", relic="Meat on the Bone")
def meat_on_bone_victory(ctx: RelicContext) -> None:
    """Meat on the Bone: Heal 12 HP if at 50% or less HP."""
    if ctx.player.hp <= ctx.player.max_hp // 2:
        ctx.heal_player(12)


# =============================================================================
# ON_SHUFFLE Triggers
# =============================================================================

@relic_trigger("onShuffle", relic="Sundial")
def sundial_shuffle(ctx: RelicContext) -> None:
    """Sundial: Gain 2 energy every 3 shuffles."""
    counter = ctx.get_relic_counter("Sundial", 0)
    counter += 1
    if counter >= 3:
        ctx.gain_energy(2)
        counter = 0
    ctx.set_relic_counter("Sundial", counter)


# =============================================================================
# ON_CHANGE_STANCE Triggers (Watcher)
# =============================================================================

@relic_trigger("onChangeStance", relic="VioletLotus")
def violet_lotus_stance(ctx: RelicContext) -> None:
    """Violet Lotus: Handled in stance change logic (+1 energy on Calm exit)."""
    # The actual energy gain is handled in stance change logic
    # because it modifies the Calm exit bonus from 2 to 3
    pass


@relic_trigger("onChangeStance", relic="TeardropLocket")
def teardrop_locket_stance(ctx: RelicContext) -> None:
    """Teardrop Locket: Start combat in Calm (handled at battle start)."""
    pass
