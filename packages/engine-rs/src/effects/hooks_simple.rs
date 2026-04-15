//! on_play hooks — simple single-action effects.
//!
//! Each hook is an exact behavioral copy of the corresponding tag handler
//! in `card_effects.rs`. These hooks receive `&mut CombatEngine` and a
//! `CardPlayContext` and perform a single logical action (draw, block,
//! energy, heal, etc.).

use crate::damage;
use crate::engine::CombatEngine;
use crate::effects::types::CardPlayContext;
use crate::cards::CardType;
use crate::state::Stance;
use crate::status_ids::sid;

// =====================================================================
// Draw effects
// =====================================================================

/// Draw N cards (base_magic or 1).
pub fn hook_draw(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let count = if ctx.card.base_magic > 0 { ctx.card.base_magic } else { 1 };
    engine.draw_cards(count);
}

/// Scrawl: draw until hand is 10.
pub fn hook_draw_to_ten(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    let cards_to_draw = (10 - engine.state.hand.len() as i32).max(0);
    if cards_to_draw > 0 {
        engine.draw_cards(cards_to_draw);
    }
}

/// Predator: draw extra cards next turn.
pub fn hook_draw_next_turn(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    engine.state.player.add_status(sid::DRAW_CARD, ctx.card.base_magic);
}

/// Expertise: draw to N cards in hand.
pub fn hook_draw_to_n(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let target = ctx.card.base_magic;
    let to_draw = (target - engine.state.hand.len() as i32).max(0);
    if to_draw > 0 {
        engine.draw_cards(to_draw);
    }
}

/// FTL: draw if few cards played this turn.
pub fn hook_draw_if_few_cards_played(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if engine.state.cards_played_this_turn < 3 {
        engine.draw_cards(ctx.card.base_magic);
    }
}

/// Calculated Gamble: discard hand, draw same count.
pub fn hook_calculated_gamble(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    let hand_count = engine.state.hand.len() as i32;
    let discarded: Vec<_> = engine.state.hand.drain(..).collect();
    engine.state.discard_pile.extend(discarded);
    if hand_count > 0 {
        engine.draw_cards(hand_count);
    }
}

// =====================================================================
// Mantra / Scry
// =====================================================================

/// Gain mantra.
pub fn hook_mantra(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if ctx.card.base_magic > 0 {
        engine.gain_mantra(ctx.card.base_magic);
    }
}

/// Do scry (may trigger AwaitingChoice).
pub fn hook_scry(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if ctx.card.base_magic > 0 {
        engine.do_scry(ctx.card.base_magic);
        // Scry triggers AwaitingChoice; the normal card-resolution loop
        // will check engine.phase and return early if needed.
    }
}

// =====================================================================
// Energy effects
// =====================================================================

/// Gain energy from base_magic (Miracle).
pub fn hook_gain_energy(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if ctx.card.base_magic > 0 {
        engine.state.energy += ctx.card.base_magic;
    }
}

/// Gain exactly 1 energy (Adrenaline).
pub fn hook_gain_energy_1(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    engine.state.energy += 1;
}

/// Double current energy.
pub fn hook_double_energy(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    engine.state.energy *= 2;
}

/// Conserve Battery / Outmaneuver / Flying Knee: gain energy next turn.
pub fn hook_next_turn_energy(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    engine.state.player.add_status(sid::ENERGIZED, ctx.card.base_magic);
}

/// Bloodletting: lose HP, gain 2 energy.
pub fn hook_lose_hp_gain_energy(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    engine.player_lose_hp(ctx.card.base_magic);
    engine.state.energy += 2;
}

/// Aggregate: gain 1 energy per 4 cards in draw pile.
pub fn hook_energy_per_cards_in_draw(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    engine.state.energy += engine.state.draw_pile.len() as i32 / 4;
}

/// Sneaky Strike: refund 2 energy if player discarded this turn.
pub fn hook_refund_energy_on_discard(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    if engine.state.player.status(sid::DISCARDED_THIS_TURN) > 0 {
        engine.state.energy += 2;
    }
}

// =====================================================================
// Vigor
// =====================================================================

/// Wreath of Flame: gain vigor status.
pub fn hook_vigor(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if ctx.card.base_magic > 0 {
        engine.state.player.add_status(sid::VIGOR, ctx.card.base_magic);
    }
}

// =====================================================================
// Block effects
// =====================================================================

/// Escape Plan: if the last drawn card is a Skill, gain block.
/// NOTE: The "draw" tag already drew a card before this hook fires.
pub fn hook_block_if_skill(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if !engine.state.hand.is_empty() {
        let last = engine.state.hand.last().unwrap();
        let last_type = engine.card_registry.card_def_by_id(last.def_id).card_type;
        if last_type == CardType::Skill {
            let dex = engine.state.player.dexterity();
            let frail = engine.state.player.is_frail();
            let block = damage::calculate_block(ctx.card.base_block.max(0), dex, frail);
            engine.gain_block_player(block);
        }
    }
}

/// Reinforced Body: gain block X times.
pub fn hook_block_x_times(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let dex = engine.state.player.dexterity();
    let frail = engine.state.player.is_frail();
    let block = damage::calculate_block(ctx.card.base_block, dex, frail);
    engine.gain_block_player(block * ctx.x_value);
}

/// Spirit Shield: gain block per card in hand.
pub fn hook_block_per_card_in_hand(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let cards_in_hand = engine.state.hand.len() as i32;
    let per_card = ctx.card.base_magic.max(1);
    let dex = engine.state.player.dexterity();
    let frail = engine.state.player.is_frail();
    let block = damage::calculate_block(per_card * cards_in_hand, dex, frail);
    engine.gain_block_player(block);
}

/// Halt: extra block in Wrath stance.
pub fn hook_extra_block_in_wrath(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if engine.state.stance == Stance::Wrath && ctx.card.base_magic > 0 {
        let dex = engine.state.player.dexterity();
        let frail = engine.state.player.is_frail();
        let extra = damage::calculate_block(ctx.card.base_magic, dex, frail);
        engine.gain_block_player(extra);
    }
}

/// Entrench: double current player block.
pub fn hook_double_block(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    engine.state.player.block *= 2;
}

/// Dodge and Roll: gain block next turn.
pub fn hook_next_turn_block(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    engine.state.player.add_status(sid::NEXT_TURN_BLOCK, ctx.card.base_magic);
}

/// Stack: gain block equal to discard pile size.
pub fn hook_block_from_discard(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    let block = engine.state.discard_pile.len() as i32;
    engine.gain_block_player(block);
}

/// Auto Shields: gain block only if player has no block.
pub fn hook_block_if_no_block(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if engine.state.player.block == 0 {
        engine.gain_block_player(ctx.card.base_block);
    }
}

// =====================================================================
// HP effects
// =====================================================================

/// Heal player (Bandage Up).
pub fn hook_heal(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let amount = ctx.card.base_magic.max(1);
    engine.heal_player(amount);
}

/// Bite: heal on play.
pub fn hook_heal_on_play(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let amount = ctx.card.base_magic.max(1);
    engine.heal_player(amount);
}

/// Offering: lose 6 HP, gain 2 energy, draw N cards.
pub fn hook_offering(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    engine.state.player.hp = (engine.state.player.hp - 6).max(0);
    engine.state.energy += 2;
    let draw_count = ctx.card.base_magic.max(3);
    engine.draw_cards(draw_count);
}

/// Lose HP (Hemokinesis).
pub fn hook_lose_hp(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    engine.player_lose_hp(ctx.card.base_magic);
}

/// J.A.X.: lose HP, gain equal Strength.
pub fn hook_lose_hp_gain_str(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    engine.player_lose_hp(ctx.card.base_magic);
    engine.state.player.add_status(sid::STRENGTH, ctx.card.base_magic);
}

// =====================================================================
// Enemy manipulation
// =====================================================================

/// Melter: remove all block from target enemy.
pub fn hook_remove_enemy_block(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if ctx.target_idx >= 0 && (ctx.target_idx as usize) < engine.state.enemies.len() {
        engine.state.enemies[ctx.target_idx as usize].entity.block = 0;
    }
}

// =====================================================================
// Pile manipulation
// =====================================================================

/// Deep Breath: shuffle discard pile into draw pile.
pub fn hook_shuffle_discard_into_draw(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    let mut cards = std::mem::take(&mut engine.state.discard_pile);
    engine.state.draw_pile.append(&mut cards);
    engine.shuffle_draw_pile();
}

/// All-Out Attack: discard 1 random card from hand.
pub fn hook_discard_random(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    if !engine.state.hand.is_empty() {
        let idx = engine.rng_gen_range(0..engine.state.hand.len());
        let card = engine.state.hand.remove(idx);
        engine.state.discard_pile.push(card);
    }
}

// =====================================================================
// Status flag setters
// =====================================================================

/// Battle Trance: no more draw this turn.
pub fn hook_no_draw(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    engine.state.player.set_status(sid::NO_DRAW, 1);
}

/// Blur: retain block at end of turn.
pub fn hook_retain_block(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    engine.state.player.add_status(sid::BLUR, ctx.card.base_magic.max(1));
}

/// Equilibrium: retain entire hand this turn.
pub fn hook_retain_hand(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    engine.state.player.set_status(sid::RETAIN_HAND_FLAG, 1);
}

/// Phantasmal Killer: double damage next turn.
pub fn hook_phantasmal_killer(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    engine.state.player.add_status(sid::DOUBLE_DAMAGE, 1);
}

/// Sentinel: gain energy when exhausted (only under Corruption).
pub fn hook_energy_on_exhaust(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if engine.state.player.status(sid::CORRUPTION) > 0 {
        let amount = ctx.card.base_magic.max(2);
        engine.state.energy += amount;
    }
}

// =====================================================================
// Conditional energy + draw
// =====================================================================

/// Dropkick: if target is vulnerable, gain 1 energy + draw 1.
pub fn hook_if_vulnerable_energy_draw(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if ctx.target_idx >= 0 && (ctx.target_idx as usize) < engine.state.enemies.len() {
        if engine.state.enemies[ctx.target_idx as usize].entity.is_vulnerable() {
            engine.state.energy += 1;
            engine.draw_cards(1);
        }
    }
}

/// Heel Hook: if target is weak, gain 1 energy + draw 1.
pub fn hook_if_weak_energy_draw(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if ctx.target_idx >= 0 && (ctx.target_idx as usize) < engine.state.enemies.len() {
        if engine.state.enemies[ctx.target_idx as usize].entity.status(sid::WEAKENED) > 0 {
            engine.state.energy += 1;
            engine.draw_cards(1);
        }
    }
}
