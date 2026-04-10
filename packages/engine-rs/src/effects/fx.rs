//! Helper API for card effect implementations.
//! Provides clean functions that route through proper engine methods.
//!
//! These helpers are thin wrappers that enforce correct ordering
//! (Artifact checks, dex/frail pipeline, hand size limits, etc.)
//! so card implementations don't need to repeat the boilerplate.

use crate::combat_types::CardInstance;
use crate::damage;
use crate::engine::CombatEngine;
use crate::ids::StatusId;
use crate::powers;
use crate::status_ids::sid;

// ===========================================================================
// Debuff helpers (route through Artifact check)
// ===========================================================================

/// Apply a debuff to a single enemy, respecting Artifact.
/// Returns true if the debuff was actually applied (not blocked by Artifact).
pub fn apply_debuff(engine: &mut CombatEngine, enemy_idx: usize, status: StatusId, amount: i32) -> bool {
    if enemy_idx < engine.state.enemies.len() {
        powers::apply_debuff(&mut engine.state.enemies[enemy_idx].entity, status, amount)
    } else {
        false
    }
}

/// Apply a debuff to all living enemies, respecting Artifact on each.
pub fn apply_debuff_all(engine: &mut CombatEngine, status: StatusId, amount: i32) {
    let living = engine.state.living_enemy_indices();
    for idx in living {
        powers::apply_debuff(&mut engine.state.enemies[idx].entity, status, amount);
    }
}

// ===========================================================================
// Block helpers (route through dex/frail pipeline)
// ===========================================================================

/// Gain block for the player, applying dexterity and frail modifiers.
/// Routes through `engine.gain_block_player()` which fires Juggernaut
/// and Wave of the Hand reactions.
pub fn gain_block(engine: &mut CombatEngine, base_block: i32) {
    let dex = engine.state.player.dexterity();
    let frail = engine.state.player.is_frail();
    let block = damage::calculate_block(base_block, dex, frail);
    engine.gain_block_player(block);
}

/// Gain raw block for the player (no dex/frail modification).
/// Still routes through `engine.gain_block_player()` for Juggernaut/WotH.
pub fn gain_block_raw(engine: &mut CombatEngine, amount: i32) {
    engine.gain_block_player(amount);
}

// ===========================================================================
// Card creation helpers
// ===========================================================================

/// Create a temp card instance by name, respecting Master Reality (auto-upgrade).
pub fn temp_card(engine: &CombatEngine, name: &str) -> CardInstance {
    engine.temp_card(name)
}

/// Add a temp card to the player's hand (respects hand size limit of 10).
/// Returns true if the card was added.
pub fn add_card_to_hand(engine: &mut CombatEngine, name: &str) -> bool {
    if engine.state.hand.len() < 10 {
        let card = engine.temp_card(name);
        engine.state.hand.push(card);
        true
    } else {
        false
    }
}

/// Add a temp card to the draw pile and shuffle.
pub fn add_card_to_draw(engine: &mut CombatEngine, name: &str) {
    let card = engine.temp_card(name);
    engine.state.draw_pile.push(card);
    engine.shuffle_draw_pile();
}

/// Add a temp card to the discard pile.
pub fn add_card_to_discard(engine: &mut CombatEngine, name: &str) {
    let card = engine.temp_card(name);
    engine.state.discard_pile.push(card);
}

// ===========================================================================
// Status helpers
// ===========================================================================

/// Add a player status (buff or debuff stacks).
pub fn add_player_status(engine: &mut CombatEngine, status: StatusId, amount: i32) {
    engine.state.player.add_status(status, amount);
}

/// Set a player status to an exact value.
pub fn set_player_status(engine: &mut CombatEngine, status: StatusId, value: i32) {
    engine.state.player.set_status(status, value);
}

/// Get the current value of a player status.
pub fn player_status(engine: &CombatEngine, status: StatusId) -> i32 {
    engine.state.player.status(status)
}

/// Add a status to a single enemy (not a debuff — no Artifact check).
/// Use `apply_debuff` for Weak/Vuln/Frail/Poison.
pub fn add_enemy_status(engine: &mut CombatEngine, enemy_idx: usize, status: StatusId, amount: i32) {
    if enemy_idx < engine.state.enemies.len() {
        engine.state.enemies[enemy_idx].entity.add_status(status, amount);
    }
}

// ===========================================================================
// Common debuff shortcuts
// ===========================================================================

/// Apply Weak to a single enemy (handles Artifact).
pub fn apply_weak(engine: &mut CombatEngine, enemy_idx: usize, amount: i32) -> bool {
    apply_debuff(engine, enemy_idx, sid::WEAKENED, amount)
}

/// Apply Vulnerable to a single enemy (handles Artifact).
pub fn apply_vulnerable(engine: &mut CombatEngine, enemy_idx: usize, amount: i32) -> bool {
    apply_debuff(engine, enemy_idx, sid::VULNERABLE, amount)
}

/// Apply Weak to all living enemies.
pub fn apply_weak_all(engine: &mut CombatEngine, amount: i32) {
    apply_debuff_all(engine, sid::WEAKENED, amount);
}

/// Apply Vulnerable to all living enemies.
pub fn apply_vulnerable_all(engine: &mut CombatEngine, amount: i32) {
    apply_debuff_all(engine, sid::VULNERABLE, amount);
}

// ===========================================================================
// Target validation
// ===========================================================================

/// Check if a target_idx is valid for the current combat.
pub fn valid_enemy_target(engine: &CombatEngine, target_idx: i32) -> bool {
    target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len()
}
