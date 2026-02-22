"""
Defect Card Effect Implementations.

This module registers all Defect card effects using the effect registry.
Effects are implemented as pure functions that modify the EffectContext.

The effects are organized by category:
- Orb channeling effects (Zap, Ball Lightning, Coolheaded, etc.)
- Orb evoke effects (Dualcast, Multi-Cast, Recursion)
- Focus manipulation (Defragment, Consume, Biased Cognition)
- Orb-counting effects (Barrage, Blizzard, Thunder Strike)
- Card manipulation (All For One, Hologram, Seek, Reboot)
- Powers (Echo Form, Creative AI, Storm, Static Discharge)
"""

from __future__ import annotations

from typing import TYPE_CHECKING, List, Optional, Dict, Any
import random

from .registry import (
    effect, effect_simple, effect_custom, EffectContext
)
from .orbs import (
    get_orb_manager, OrbType, channel_orb, channel_random_orb,
    evoke_orb, evoke_all_orbs, trigger_orb_passives
)

if TYPE_CHECKING:
    from ..state.combat import EnemyCombatState


# =============================================================================
# Orb Channeling Effects
# =============================================================================

def _is_zero_cost_card(ctx: EffectContext, card_id: str) -> bool:
    """Check if a card is 0-cost, respecting per-turn overrides."""
    card_cost = ctx.state.card_costs.get(card_id)
    if card_cost is None:
        base_id = card_id.rstrip("+")
        card_cost = ctx.state.card_costs.get(base_id)
    if card_cost is None:
        try:
            from ..content.cards import get_card, normalize_card_id
            base_id, upgraded = normalize_card_id(card_id)
            card_cost = get_card(base_id, upgraded=upgraded).current_cost
        except Exception:
            return False
    return card_cost == 0

@effect_simple("channel_lightning")
def channel_lightning_effect(ctx: EffectContext) -> None:
    """Channel 1 Lightning orb (Zap, Ball Lightning)."""
    channel_orb(ctx.state, "Lightning")


@effect_simple("channel_lightning_magic")
def channel_lightning_magic_effect(ctx: EffectContext) -> None:
    """Channel Lightning orbs equal to magic number (Electrodynamics).

    Java: Electrodynamics channels magicNumber Lightning orbs (base: 2, upgraded: 3).
    """
    count = ctx.magic_number if ctx.magic_number > 0 else 2
    for _ in range(count):
        channel_orb(ctx.state, "Lightning")


@effect_simple("channel_frost")
def channel_frost_effect(ctx: EffectContext) -> None:
    """Channel 1 Frost orb (Cold Snap, Coolheaded)."""
    channel_orb(ctx.state, "Frost")


@effect_simple("channel_dark")
def channel_dark_effect(ctx: EffectContext) -> None:
    """Channel 1 Dark orb (Darkness, Doom and Gloom)."""
    channel_orb(ctx.state, "Dark")


@effect_simple("channel_plasma")
def channel_plasma_effect(ctx: EffectContext) -> None:
    """Channel 1 Plasma orb (Fusion)."""
    channel_orb(ctx.state, "Plasma")


@effect_simple("channel_2_frost")
def channel_2_frost_effect(ctx: EffectContext) -> None:
    """Channel 2 Frost orbs (Glacier)."""
    channel_orb(ctx.state, "Frost")
    channel_orb(ctx.state, "Frost")


@effect_simple("channel_3_plasma")
def channel_3_plasma_effect(ctx: EffectContext) -> None:
    """Channel 3 Plasma orbs (Meteor Strike)."""
    channel_orb(ctx.state, "Plasma")
    channel_orb(ctx.state, "Plasma")
    channel_orb(ctx.state, "Plasma")


@effect_simple("channel_random_orb")
def channel_random_orb_effect(ctx: EffectContext) -> None:
    """Channel random orb(s) (Chaos). Magic number determines count."""
    count = ctx.magic_number if ctx.magic_number > 0 else 1
    for _ in range(count):
        channel_random_orb(ctx.state)


@effect_simple("channel_frost_per_enemy")
def channel_frost_per_enemy_effect(ctx: EffectContext) -> None:
    """Channel 1 Frost per enemy (Chill)."""
    enemy_count = len(ctx.living_enemies)
    for _ in range(enemy_count):
        channel_orb(ctx.state, "Frost")


@effect_simple("channel_lightning_frost_dark")
def channel_lightning_frost_dark_effect(ctx: EffectContext) -> None:
    """Channel Lightning, Frost, and Dark (Rainbow)."""
    channel_orb(ctx.state, "Lightning")
    channel_orb(ctx.state, "Frost")
    channel_orb(ctx.state, "Dark")


@effect_simple("channel_x_lightning")
def channel_x_lightning_effect(ctx: EffectContext) -> None:
    """Channel X Lightning orbs (Tempest). X = energy spent."""
    x = ctx.extra_data.get("x_cost", ctx.state.energy)
    for _ in range(x):
        channel_orb(ctx.state, "Lightning")


# =============================================================================
# Orb Evoke Effects
# =============================================================================

@effect_simple("evoke_orb_twice")
def evoke_orb_twice_effect(ctx: EffectContext) -> None:
    """Evoke the leftmost orb twice (Dualcast)."""
    manager = get_orb_manager(ctx.state)
    if manager.has_orbs():
        evoke_orb(ctx.state, times=2)


@effect_simple("evoke_first_orb_x_times")
def evoke_first_orb_x_times_effect(ctx: EffectContext) -> None:
    """Evoke first orb X times (Multi-Cast). X = energy spent."""
    x = ctx.extra_data.get("x_cost", ctx.state.energy)
    if x > 0:
        manager = get_orb_manager(ctx.state)
        if manager.has_orbs():
            evoke_orb(ctx.state, times=x)


@effect_simple("evoke_then_channel_same_orb")
def evoke_then_channel_same_effect(ctx: EffectContext) -> None:
    """Evoke first orb, then channel the same type (Recursion)."""
    manager = get_orb_manager(ctx.state)
    if manager.has_orbs():
        first_orb = manager.get_first_orb()
        orb_type = first_orb.orb_type.value
        evoke_orb(ctx.state)
        channel_orb(ctx.state, orb_type)


@effect_simple("remove_orbs_gain_energy_and_draw")
def fission_effect(ctx: EffectContext) -> None:
    """
    Remove all orbs, gain 1 energy and draw 1 per orb (Fission).
    Upgraded version gains resources, base version does not.
    """
    gain_resources = ctx.is_upgraded
    result = evoke_all_orbs(ctx.state, gain_resources=gain_resources)

    if gain_resources:
        ctx.gain_energy(result["orbs_evoked"])
        ctx.draw_cards(result["orbs_evoked"])


# =============================================================================
# Focus Manipulation Effects
# =============================================================================

@effect_simple("gain_focus")
def gain_focus_effect(ctx: EffectContext) -> None:
    """Gain Focus (Defragment). Amount from magic_number."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 1
    ctx.apply_status_to_player("Focus", amount)

    # Also update orb manager's focus
    manager = get_orb_manager(ctx.state)
    manager.modify_focus(amount)


@effect_simple("lose_focus")
def lose_focus_effect(ctx: EffectContext) -> None:
    """Lose Focus (Hyperbeam). Amount from magic_number."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 3
    ctx.apply_status_to_player("Focus", -amount)

    manager = get_orb_manager(ctx.state)
    manager.modify_focus(-amount)


@effect_simple("gain_focus_lose_orb_slot")
def consume_effect(ctx: EffectContext) -> None:
    """Gain Focus, lose 1 orb slot (Consume)."""
    focus_amount = ctx.magic_number if ctx.magic_number > 0 else 2
    ctx.apply_status_to_player("Focus", focus_amount)

    manager = get_orb_manager(ctx.state)
    manager.modify_focus(focus_amount)
    manager.remove_orb_slot(1, ctx.state)


@effect_simple("gain_focus_lose_focus_each_turn")
def biased_cognition_effect(ctx: EffectContext) -> None:
    """
    Gain Focus, lose 1 Focus at start of each turn (Biased Cognition).

    Applies the power that loses focus each turn.
    """
    amount = ctx.magic_number if ctx.magic_number > 0 else 4
    ctx.apply_status_to_player("Focus", amount)
    ctx.apply_status_to_player("BiasedCognition", 1)

    manager = get_orb_manager(ctx.state)
    manager.modify_focus(amount)


@effect_simple("lose_focus_gain_strength_dex")
def reprogram_effect(ctx: EffectContext) -> None:
    """Lose 1 Focus, gain Strength and Dexterity (Reprogram)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 1
    ctx.apply_status_to_player("Focus", -amount)
    ctx.apply_status_to_player("Strength", amount)
    ctx.apply_status_to_player("Dexterity", amount)

    manager = get_orb_manager(ctx.state)
    manager.modify_focus(-amount)


# =============================================================================
# Orb Slot Effects
# =============================================================================

@effect_simple("increase_orb_slots")
def increase_orb_slots_effect(ctx: EffectContext) -> None:
    """Increase orb slots (Capacitor)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 2
    ctx.apply_status_to_player("OrbSlots", amount)

    manager = get_orb_manager(ctx.state)
    manager.add_orb_slot(amount)


# =============================================================================
# Orb-Counting Damage Effects
# =============================================================================

@effect_simple("damage_per_orb")
def damage_per_orb_effect(ctx: EffectContext) -> None:
    """Deal damage for each orb channeled (Barrage)."""
    manager = get_orb_manager(ctx.state)
    orb_count = manager.get_orb_count()

    if ctx.card and ctx.target:
        damage = ctx.card.damage
        for _ in range(orb_count):
            ctx.deal_damage_to_target(damage)


@effect_simple("draw_per_unique_orb")
def draw_per_unique_orb_effect(ctx: EffectContext) -> None:
    """Draw 1 card per unique orb type (Compile Driver)."""
    manager = get_orb_manager(ctx.state)
    unique_count = manager.get_unique_orb_types()
    ctx.draw_cards(unique_count)


@effect_simple("damage_per_frost_channeled")
def damage_per_frost_channeled_effect(ctx: EffectContext) -> None:
    """Deal damage per Frost channeled this combat (Blizzard)."""
    manager = get_orb_manager(ctx.state)
    frost_count = manager.frost_channeled

    damage_per = ctx.magic_number if ctx.magic_number > 0 else 2
    total_damage = frost_count * damage_per

    # All enemies target
    for enemy in ctx.living_enemies:
        ctx.deal_damage_to_enemy(enemy, total_damage)


@effect_simple("damage_per_lightning_channeled")
def damage_per_lightning_channeled_effect(ctx: EffectContext) -> None:
    """Deal damage per Lightning channeled this combat (Thunder Strike)."""
    manager = get_orb_manager(ctx.state)
    lightning_count = manager.lightning_channeled

    if ctx.card:
        damage = ctx.card.damage
        # Hit random enemies (lightning_count) times
        for _ in range(lightning_count):
            living = ctx.living_enemies
            if living:
                target = random.choice(living)
                ctx.deal_damage_to_enemy(target, damage)


# =============================================================================
# Orb Passive Trigger Effects
# =============================================================================

@effect_simple("trigger_orb_passive_extra")
def loop_effect(ctx: EffectContext) -> None:
    """Trigger rightmost orb's passive 1 extra time (Loop power)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 1
    ctx.apply_status_to_player("Loop", amount)

    manager = get_orb_manager(ctx.state)
    manager.loop_stacks += amount


@effect_simple("trigger_orb_start_end")
def impulse_effect(ctx: EffectContext) -> None:
    """Impulse: trigger each orb's start/end behavior once, then Cables bonus."""
    manager = get_orb_manager(ctx.state)
    if not manager.orbs:
        return

    # Java ImpulseAction iterates all player orbs and calls onStartOfTurn/onEndOfTurn.
    # For core orb types in this engine, passive behavior is represented by _execute_passive.
    for orb in list(manager.orbs):
        manager._execute_passive(orb, ctx.state, manager.focus)

    # Java relic check uses ID "Cables" for Gold-Plated Cables and triggers rightmost again.
    if ctx.state.has_relic("Cables") and manager.orbs:
        manager._execute_passive(manager.orbs[-1], ctx.state, manager.focus)


@effect_simple("lightning_hits_all")
def electrodynamics_lightning_all_effect(ctx: EffectContext) -> None:
    """Lightning orbs now hit ALL enemies (Electrodynamics)."""
    ctx.apply_status_to_player("Electrodynamics", 1)

    manager = get_orb_manager(ctx.state)
    manager.lightning_hits_all = True


# =============================================================================
# Power Card Effects (Defect)
# =============================================================================

@effect_simple("channel_lightning_on_power_play")
def storm_power_effect(ctx: EffectContext) -> None:
    """When you play a Power, channel 1 Lightning (Storm power)."""
    ctx.apply_status_to_player("Storm", 1)


@effect_simple("channel_lightning_on_damage")
def static_discharge_effect(ctx: EffectContext) -> None:
    """When you take damage, channel 1 Lightning (Static Discharge)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 1
    ctx.apply_status_to_player("StaticDischarge", amount)


@effect_simple("draw_on_power_play")
def heatsinks_effect(ctx: EffectContext) -> None:
    """When you play a Power, draw cards (Heatsinks)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 1
    ctx.apply_status_to_player("Heatsinks", amount)


@effect_simple("play_first_card_twice")
def echo_form_effect(ctx: EffectContext) -> None:
    """First card each turn plays twice (Echo Form)."""
    ctx.apply_status_to_player("EchoForm", 1)


@effect_simple("add_random_power_each_turn")
def creative_ai_effect(ctx: EffectContext) -> None:
    """At start of turn, add a random Power to hand (Creative AI)."""
    ctx.apply_status_to_player("CreativeAI", 1)


@effect_simple("draw_extra_each_turn")
def machine_learning_effect(ctx: EffectContext) -> None:
    """Draw 1 additional card each turn (Machine Learning)."""
    ctx.apply_status_to_player("MachineLearning", 1)


@effect_simple("add_common_card_each_turn")
def hello_world_effect(ctx: EffectContext) -> None:
    """At start of turn, add a random common card to hand (Hello World)."""
    ctx.apply_status_to_player("HelloWorld", 1)


@effect_simple("heal_at_end_of_combat")
def self_repair_effect(ctx: EffectContext) -> None:
    """Heal at end of combat (Self Repair)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 7
    ctx.apply_status_to_player("SelfRepair", amount)


@effect_simple("prevent_next_hp_loss")
def buffer_effect(ctx: EffectContext) -> None:
    """Prevent next HP loss (Buffer)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 1
    ctx.apply_status_to_player("Buffer", amount)


@effect_simple("next_power_plays_twice")
def amplify_effect(ctx: EffectContext) -> None:
    """Next Power card plays twice (Amplify)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 1
    ctx.apply_status_to_player("Amplify", amount)


# =============================================================================
# Card Manipulation Effects (Defect)
# =============================================================================

@effect_simple("return_all_0_cost_from_discard")
def all_for_one_effect(ctx: EffectContext) -> None:
    """Return all 0-cost cards from discard to hand (All For One)."""
    zero_cost_cards = [
        card_id for card_id in ctx.state.discard_pile[:]
        if _is_zero_cost_card(ctx, card_id)
    ]

    # Move to hand (up to hand limit)
    for card_id in zero_cost_cards:
        if len(ctx.state.hand) < 10:
            ctx.state.discard_pile.remove(card_id)
            ctx.state.hand.append(card_id)


@effect_simple("return_card_from_discard")
def hologram_effect(ctx: EffectContext) -> None:
    """Return a card from discard to hand (Hologram). Requires selection."""
    if ctx.state.discard_pile and len(ctx.state.hand) < 10:
        # In simulation, return first card (player would choose)
        card_idx = ctx.extra_data.get("hologram_choice", 0)
        if 0 <= card_idx < len(ctx.state.discard_pile):
            card = ctx.state.discard_pile.pop(card_idx)
            ctx.state.hand.append(card)


@effect_simple("search_draw_pile")
def seek_effect(ctx: EffectContext) -> None:
    """Search draw pile for card(s) to put in hand (Seek)."""
    count = ctx.magic_number if ctx.magic_number > 0 else 1
    choice_indices = ctx.extra_data.get("seek_choices", [0] * count)

    for idx in choice_indices[:count]:
        if ctx.state.draw_pile and len(ctx.state.hand) < 10:
            actual_idx = min(idx, len(ctx.state.draw_pile) - 1)
            if actual_idx >= 0:
                card = ctx.state.draw_pile.pop(actual_idx)
                ctx.state.hand.append(card)


@effect_simple("shuffle_hand_and_discard_draw")
def reboot_effect(ctx: EffectContext) -> None:
    """Shuffle hand and discard into draw, draw X cards (Reboot)."""
    draw_count = ctx.magic_number if ctx.magic_number > 0 else 4

    # Move hand and discard to draw
    ctx.state.draw_pile.extend(ctx.state.hand)
    ctx.state.draw_pile.extend(ctx.state.discard_pile)
    ctx.state.hand.clear()
    ctx.state.discard_pile.clear()

    # Shuffle draw pile
    random.shuffle(ctx.state.draw_pile)

    # Draw cards
    ctx.draw_cards(draw_count)


@effect_simple("exhaust_card_gain_energy")
def recycle_effect(ctx: EffectContext) -> None:
    """Exhaust a card, gain energy equal to its cost (Recycle)."""
    choice_idx = ctx.extra_data.get("recycle_choice", 0)
    if 0 <= choice_idx < len(ctx.state.hand):
        card_id = ctx.state.hand[choice_idx]
        # Get card cost (simplified - would use registry)
        card_cost = ctx.state.card_costs.get(card_id, 1)
        ctx.exhaust_hand_idx(choice_idx)
        ctx.gain_energy(card_cost)


@effect_simple("next_card_on_top_of_draw")
def rebound_effect(ctx: EffectContext) -> None:
    """Next card played goes on top of draw pile (Rebound)."""
    ctx.apply_status_to_player("Rebound", 1)


@effect_simple("add_random_power_to_hand_cost_0")
def white_noise_effect(ctx: EffectContext) -> None:
    """Add a random Power card to hand that costs 0 this turn (White Noise)."""
    # List of Defect power cards
    powers = [
        "Defragment", "Capacitor", "Heatsinks", "Hello World", "Loop",
        "Self Repair", "Static Discharge", "Storm", "Biased Cognition",
        "Buffer", "Creative AI", "Echo Form", "Electrodynamics", "Machine Learning"
    ]
    chosen = random.choice(powers)
    if len(ctx.state.hand) < 10:
        ctx.state.hand.append(chosen)
        # Set cost to 0 for this turn
        ctx.state.card_costs[chosen] = 0


# =============================================================================
# Conditional Effects (Defect)
# =============================================================================

@effect_simple("if_attacking_apply_weak")
def go_for_the_eyes_effect(ctx: EffectContext) -> None:
    """If enemy is attacking, apply Weak (Go for the Eyes)."""
    if ctx.target and ctx.target.is_attacking:
        amount = ctx.magic_number if ctx.magic_number > 0 else 1
        ctx.apply_status_to_target("Weak", amount)


@effect_simple("if_played_less_than_x_draw")
def ftl_effect(ctx: EffectContext) -> None:
    """If played fewer than X cards this turn, draw 1 (FTL)."""
    threshold = ctx.magic_number if ctx.magic_number > 0 else 3
    if ctx.state.cards_played_this_turn < threshold:
        ctx.draw_cards(1)


@effect_simple("if_fatal_gain_3_energy")
def sunder_effect(ctx: EffectContext) -> None:
    """If this kills the enemy, gain 3 energy (Sunder)."""
    if ctx.target and ctx.target.hp <= 0:
        ctx.gain_energy(3)


@effect_simple("only_if_no_block")
def auto_shields_effect(ctx: EffectContext) -> None:
    """Only gain block if you have none (Auto-Shields)."""
    if ctx.state.player.block > 0:
        # Don't apply block (handled by card's base block being conditional)
        ctx.extra_data["auto_shields_blocked"] = True


@effect_simple("remove_enemy_block")
def melter_effect(ctx: EffectContext) -> None:
    """Remove all enemy block before dealing damage (Melter)."""
    if ctx.target:
        ctx.target.block = 0


@effect_simple("damage_random_enemy_twice")
def rip_and_tear_effect(ctx: EffectContext) -> None:
    """Deal damage to a random enemy twice (Rip and Tear)."""
    if ctx.card:
        damage = ctx.card.damage
        for _ in range(2):
            living = ctx.living_enemies
            if living:
                target = random.choice(living)
                ctx.deal_damage_to_enemy(target, damage)


@effect_simple("draw_discard_non_zero_cost")
def scrape_effect(ctx: EffectContext) -> None:
    """Draw X, discard any non-0-cost cards drawn (Scrape)."""
    draw_count = ctx.magic_number if ctx.magic_number > 0 else 4
    drawn = ctx.draw_cards(draw_count)

    # Discard non-0-cost cards that were drawn
    for card_id in drawn:
        if not _is_zero_cost_card(ctx, card_id):
            if card_id in ctx.state.hand:
                ctx.discard_card(card_id)


# =============================================================================
# Block Effects (Defect)
# =============================================================================

@effect_simple("block_equals_discard_size")
def stack_effect(ctx: EffectContext) -> None:
    """Gain block equal to discard pile size (Stack)."""
    discard_count = len(ctx.state.discard_pile)
    # Upgraded version adds +3 base block
    base = 3 if ctx.is_upgraded else 0
    ctx.gain_block(discard_count + base)


@effect_simple("lose_1_block_permanently")
def steam_barrier_effect(ctx: EffectContext) -> None:
    """This card permanently loses 1 block each time played (Steam Barrier)."""
    # Track in card costs (abusing it for block tracking)
    card_id = ctx.card.id if ctx.card else "Steam"
    current_block_loss = ctx.extra_data.get(f"steam_barrier_loss_{card_id}", 0)
    ctx.extra_data[f"steam_barrier_loss_{card_id}"] = current_block_loss + 1
    # The actual block reduction is handled in card execution


@effect_simple("block_x_times")
def reinforced_body_effect(ctx: EffectContext) -> None:
    """Gain block X times (Reinforced Body). X = energy spent."""
    x = ctx.extra_data.get("x_cost", ctx.state.energy)
    if ctx.card:
        block_per = ctx.card.block
        for _ in range(x):
            ctx.gain_block(block_per)


@effect_simple("block_increases_permanently")
def genetic_algorithm_effect(ctx: EffectContext) -> None:
    """Block value increases permanently (Genetic Algorithm)."""
    increase = ctx.magic_number if ctx.magic_number > 0 else 2
    # Track the increase for this specific card
    card_id = ctx.card.id if ctx.card else "Genetic Algorithm"
    key = f"genetic_bonus_{card_id}"
    current_bonus = ctx.extra_data.get(key, 0)
    ctx.extra_data[key] = current_bonus + increase


# =============================================================================
# Energy Effects (Defect)
# =============================================================================

@effect_simple("gain_1_energy_next_turn")
def charge_battery_effect(ctx: EffectContext) -> None:
    """Gain 1 energy next turn (Charge Battery)."""
    ctx.apply_status_to_player("EnergyNextTurn", 1)


@effect_simple("gain_energy_magic")
def gain_energy_magic_effect(ctx: EffectContext) -> None:
    """Gain energy based on magic number (Turbo)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 1
    ctx.gain_energy(amount)


@effect_simple("double_energy")
def double_energy_effect(ctx: EffectContext) -> None:
    """Double your energy (Double Energy)."""
    ctx.state.energy *= 2


@effect_simple("add_void_to_discard")
def turbo_void_effect(ctx: EffectContext) -> None:
    """Add a Void to discard pile (Turbo)."""
    ctx.add_card_to_discard("Void")


@effect_simple("add_burn_to_discard")
def overclock_burn_effect(ctx: EffectContext) -> None:
    """Add a Burn to discard pile (Overclock)."""
    ctx.add_card_to_discard("Burn")


@effect_simple("gain_energy_per_x_cards_in_draw")
def aggregate_effect(ctx: EffectContext) -> None:
    """Gain 1 energy per X cards in draw pile (Aggregate)."""
    divisor = ctx.magic_number if ctx.magic_number > 0 else 4
    draw_size = len(ctx.state.draw_pile)
    energy_gain = draw_size // divisor
    ctx.gain_energy(energy_gain)


# =============================================================================
# Lock-On Effect
# =============================================================================

@effect_simple("apply_lockon")
def lockon_effect(ctx: EffectContext) -> None:
    """Apply Lock-On to target (Lock-On card)."""
    amount = ctx.magic_number if ctx.magic_number > 0 else 2
    ctx.apply_status_to_target("Lock-On", amount)


# =============================================================================
# Retain Effect
# =============================================================================

@effect_simple("retain_hand")
def equilibrium_effect(ctx: EffectContext) -> None:
    """Retain your hand this turn (Equilibrium)."""
    ctx.apply_status_to_player("RetainHand", 1)


# =============================================================================
# Claw Special Effect
# =============================================================================

@effect_simple("increase_all_claw_damage")
def claw_effect(ctx: EffectContext) -> None:
    """Increase damage of ALL Claws by 2 for rest of combat (Claw)."""
    increase = ctx.magic_number if ctx.magic_number > 0 else 2
    current = ctx.extra_data.get("claw_bonus", 0)
    ctx.extra_data["claw_bonus"] = current + increase


@effect_simple("reduce_cost_permanently")
def streamline_effect(ctx: EffectContext) -> None:
    """Reduce cost of this card permanently (Streamline)."""
    if ctx.card:
        current = ctx.state.card_costs.get(ctx.card.id, ctx.card.cost)
        ctx.state.card_costs[ctx.card.id] = max(0, current - 1)


@effect_simple("cost_reduces_per_power_played")
def force_field_effect(ctx: EffectContext) -> None:
    """Cost reduces by 1 for each Power played this combat (Force Field)."""
    # This is tracked passively - the actual cost reduction happens
    # when calculating the card's cost
    pass


# =============================================================================
# Card Effect Registry for Defect
# =============================================================================

DEFECT_CARD_EFFECTS = {
    # Basic
    "Strike_B": [],
    "Defend_B": [],
    "Zap": ["channel_lightning"],
    "Dualcast": ["evoke_orb_twice"],

    # Common Attacks
    "Ball Lightning": ["channel_lightning"],
    "Barrage": ["damage_per_orb"],
    "Beam Cell": ["apply_vulnerable"],
    "Claw": ["increase_all_claw_damage"],
    "Gash": ["increase_all_claw_damage"],
    "Cold Snap": ["channel_frost"],
    "Compile Driver": ["draw_per_unique_orb"],
    "Go for the Eyes": ["if_attacking_apply_weak"],
    "Rebound": ["next_card_on_top_of_draw"],
    "Streamline": ["reduce_cost_permanently"],
    "Sweeping Beam": ["draw_1"],

    # Common Skills
    "Conserve Battery": ["gain_1_energy_next_turn"],
    "Coolheaded": ["channel_frost", "draw_cards"],
    "Hologram": ["return_card_from_discard"],
    "Leap": [],
    "Redo": ["evoke_then_channel_same_orb"],
    "Stack": ["block_equals_discard_size"],
    "Steam": ["lose_1_block_permanently"],
    "Turbo": ["add_void_to_discard"],

    # Uncommon Attacks
    "Blizzard": ["damage_per_frost_channeled"],
    "Doom and Gloom": ["channel_dark"],
    "FTL": ["if_played_less_than_x_draw"],
    "Lockon": ["apply_lockon"],
    "Melter": ["remove_enemy_block"],
    "Rip and Tear": ["damage_random_enemy_twice"],
    "Scrape": ["draw_discard_non_zero_cost"],
    "Sunder": ["if_fatal_gain_3_energy"],

    # Uncommon Skills
    "Aggregate": ["gain_energy_per_x_cards_in_draw"],
    "Auto Shields": ["only_if_no_block"],
    "BootSequence": [],
    "Chaos": ["channel_random_orb"],
    "Chill": ["channel_frost_per_enemy"],
    "Consume": ["gain_focus_lose_orb_slot"],
    "Darkness": ["channel_dark"],
    "Double Energy": ["double_energy"],
    "Undo": ["retain_hand"],
    "Force Field": ["cost_reduces_per_power_played"],
    "Fusion": ["channel_plasma"],
    "Genetic Algorithm": ["block_increases_permanently"],
    "Glacier": ["channel_2_frost"],
    "Steam Power": ["add_burn_to_discard"],
    "Impulse": ["trigger_orb_start_end"],
    "Recycle": ["exhaust_card_gain_energy"],
    "Reinforced Body": ["block_x_times"],
    "Reprogram": ["lose_focus_gain_strength_dex"],
    "Skim": [],  # Just draw
    "Tempest": ["channel_x_lightning"],
    "White Noise": ["add_random_power_to_hand_cost_0"],

    # Uncommon Powers
    "Capacitor": ["increase_orb_slots"],
    "Defragment": ["gain_focus"],
    "Heatsinks": ["draw_on_power_play"],
    "Hello World": ["add_common_card_each_turn"],
    "Loop": ["trigger_orb_passive_extra"],
    "Self Repair": ["heal_at_end_of_combat"],
    "Static Discharge": ["channel_lightning_on_damage"],
    "Storm": ["channel_lightning_on_power_play"],

    # Rare Attacks
    "All For One": ["return_all_0_cost_from_discard"],
    "Core Surge": ["gain_artifact"],
    "Hyperbeam": ["lose_focus"],
    "Meteor Strike": ["channel_3_plasma"],
    "Thunder Strike": ["damage_per_lightning_channeled"],

    # Rare Skills
    "Amplify": ["next_power_plays_twice"],
    "Fission": ["remove_orbs_gain_energy_and_draw"],
    "Multi-Cast": ["evoke_first_orb_x_times"],
    "Rainbow": ["channel_lightning_frost_dark"],
    "Reboot": ["shuffle_hand_and_discard_draw"],
    "Seek": ["search_draw_pile"],

    # Rare Powers
    "Biased Cognition": ["gain_focus_lose_focus_each_turn"],
    "Buffer": ["prevent_next_hp_loss"],
    "Creative AI": ["add_random_power_each_turn"],
    "Echo Form": ["play_first_card_twice"],
    "Electrodynamics": ["lightning_hits_all", "channel_lightning_magic"],
    "Machine Learning": ["draw_extra_each_turn"],
}


def get_defect_card_effects(card_id: str) -> List[str]:
    """Get the effect names for a Defect card."""
    base_id = card_id.rstrip("+")
    return DEFECT_CARD_EFFECTS.get(base_id, [])


# =============================================================================
# Register all effects on module load
# =============================================================================

def _ensure_effects_registered():
    """Ensure all effects are registered. Called on module import."""
    pass


_ensure_effects_registered()
