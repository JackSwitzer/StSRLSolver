//! on_play hooks for orb-related effect tags (Defect orb system).

use crate::cards::CardType;
use crate::damage;
use crate::engine::CombatEngine;
use crate::orbs::{self, OrbType};
use crate::status_ids::sid;
use super::types::CardPlayContext;

// =========================================================================
// Channel orbs
// =========================================================================

/// Channel Lightning orb (Ball Lightning, Electrodynamics, etc.)
/// Channels `base_magic` Lightning orbs (min 1).
pub fn hook_channel_lightning(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let count = ctx.card.base_magic.max(1);
    for _ in 0..count {
        engine.channel_orb(OrbType::Lightning);
    }
}

/// Channel Frost orb (Cold Snap, Glacier, etc.)
/// Channels `base_magic` Frost orbs (min 1).
pub fn hook_channel_frost(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let count = ctx.card.base_magic.max(1);
    for _ in 0..count {
        engine.channel_orb(OrbType::Frost);
    }
}

/// Channel Dark orb (Doom and Gloom, etc.)
/// Channels `base_magic` Dark orbs (min 1).
pub fn hook_channel_dark(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let count = ctx.card.base_magic.max(1);
    for _ in 0..count {
        engine.channel_orb(OrbType::Dark);
    }
}

/// Channel Plasma orb (Fusion, Rainbow, etc.)
/// Channels `base_magic` Plasma orbs (min 1).
pub fn hook_channel_plasma(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let count = ctx.card.base_magic.max(1);
    for _ in 0..count {
        engine.channel_orb(OrbType::Plasma);
    }
}

/// Chill: channel one Frost orb per living enemy.
pub fn hook_channel_frost_per_enemy(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    let count = engine.state.living_enemy_indices().len();
    for _ in 0..count {
        engine.channel_orb(OrbType::Frost);
    }
}

// =========================================================================
// Evoke orbs
// =========================================================================

/// Dualcast: evoke the front orb. The tag can appear multiple times on a card
/// (e.g. Dualcast evokes twice), so we count occurrences.
pub fn hook_evoke_orb(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let evoke_count = ctx.card.effects.iter().filter(|&&e| e == "evoke_orb").count();
    if evoke_count > 0 {
        let focus = engine.state.player.focus();
        for _ in 0..evoke_count {
            let effect = engine.state.orb_slots.evoke_front(focus);
            engine.apply_evoke_effect(effect);
        }
    }
}

/// Multi-Cast: evoke front orb X times (X-cost card).
pub fn hook_evoke_orb_x(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let count = ctx.x_value as usize;
    engine.evoke_front_orb_n(count);
}

/// Multi-Cast+: evoke front orb X+1 times (X-cost card).
pub fn hook_evoke_orb_x_plus_1(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let count = (ctx.x_value + 1) as usize;
    engine.evoke_front_orb_n(count);
}

/// Evoke all orbs (e.g. from a card effect).
pub fn hook_evoke_all(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    engine.evoke_all_orbs();
}

// =========================================================================
// Focus manipulation
// =========================================================================

/// Defragment/Consume: gain Focus equal to `base_magic` (min 1).
pub fn hook_gain_focus(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let amount = ctx.card.base_magic.max(1);
    engine.state.player.add_status(sid::FOCUS, amount);
}

/// Hyperbeam: lose Focus equal to `base_magic` (min 1).
pub fn hook_lose_focus(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let amount = ctx.card.base_magic.max(1);
    engine.state.player.add_status(sid::FOCUS, -amount);
}

// =========================================================================
// Orb slot manipulation
// =========================================================================

/// Consume: lose one orb slot (evokes if full), paired with gain_focus.
pub fn hook_lose_orb_slot(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    let focus = engine.state.player.focus();
    let evoke = engine.state.orb_slots.remove_slot(focus);
    engine.apply_evoke_effect(evoke);
}

/// Capacitor (non-Power): gain orb slots equal to `base_magic` (min 1).
/// Power-type Capacitor is handled separately by install_power().
pub fn hook_gain_orb_slots(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    if ctx.card.card_type != CardType::Power {
        let amount = ctx.card.base_magic.max(1);
        for _ in 0..amount {
            engine.state.orb_slots.add_slot();
        }
    }
}

// =========================================================================
// X-cost channel
// =========================================================================

/// Tempest: channel X Lightning orbs (X-cost card).
pub fn hook_channel_lightning_x(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let count = ctx.x_value;
    for _ in 0..count {
        engine.channel_orb(OrbType::Lightning);
    }
}

/// Tempest+: channel X+1 Lightning orbs (X-cost card).
pub fn hook_channel_lightning_x_plus_1(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let count = ctx.x_value + 1;
    for _ in 0..count {
        engine.channel_orb(OrbType::Lightning);
    }
}

// =========================================================================
// Passive/Dark triggers
// =========================================================================

/// Darkness/Darkness+: trigger dark orb passives (accumulate evoke damage).
pub fn hook_trigger_dark_passive(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    let focus = engine.state.player.focus();
    for orb in engine.state.orb_slots.slots.iter_mut() {
        if orb.orb_type == OrbType::Dark {
            let gain = (orb.base_passive + focus).max(0);
            orb.evoke_amount += gain;
        }
    }
}

/// Trigger all orb passives (Loop, etc.)
/// Lightning: damage random enemy. Frost: gain block. Plasma: gain energy.
/// Dark: accumulate evoke amount.
pub fn hook_trigger_all_passives(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    let focus = engine.state.player.focus();
    for i in 0..engine.state.orb_slots.slots.len() {
        let orb = &engine.state.orb_slots.slots[i];
        if orb.is_empty() {
            continue;
        }
        let passive_val = orb.passive_with_focus(focus);
        match orb.orb_type {
            OrbType::Frost => {
                engine.state.player.block += passive_val;
            }
            OrbType::Lightning => {
                let living = engine.state.living_enemy_indices();
                if let Some(&idx) = living.first() {
                    engine.deal_damage_to_enemy(idx, passive_val);
                }
            }
            OrbType::Plasma => {
                engine.state.energy += passive_val;
            }
            OrbType::Dark => {
                // Dark passive increases its own evoke amount
                engine.state.orb_slots.slots[i].evoke_amount += passive_val;
            }
            _ => {}
        }
    }
}

// =========================================================================
// Fission
// =========================================================================

/// Fission: remove all orbs (without evoking), gain energy + draw per orb.
pub fn hook_fission(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    let orb_count = engine.state.orb_slots.occupied_count() as i32;
    engine.state.orb_slots.slots =
        vec![orbs::Orb::new(OrbType::Empty); engine.state.orb_slots.max_slots];
    if orb_count > 0 {
        engine.state.energy += orb_count;
        engine.draw_cards(orb_count);
    }
}

/// Fission+: evoke all orbs, then gain energy + draw per orb.
pub fn hook_fission_evoke(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    let orb_count = engine.state.orb_slots.occupied_count() as i32;
    let focus = engine.state.player.focus();
    let effects = engine.state.orb_slots.evoke_all(focus);
    for effect in effects {
        engine.apply_evoke_effect(effect);
    }
    if orb_count > 0 {
        engine.state.energy += orb_count;
        engine.draw_cards(orb_count);
    }
}

// =========================================================================
// Channel random
// =========================================================================

/// Chaos: channel a random orb type.
pub fn hook_channel_random(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    let orb_types = [OrbType::Lightning, OrbType::Frost, OrbType::Dark, OrbType::Plasma];
    let idx = engine.rng_gen_range(0..orb_types.len());
    let focus = engine.state.player.focus();
    let evoke = engine.state.orb_slots.channel(orb_types[idx], focus);
    engine.apply_evoke_effect(evoke);
}

// =========================================================================
// Orb-dependent damage/draw
// =========================================================================

/// Barrage: deal damage once per channeled orb (beyond the first hit from
/// the generic damage loop). The first hit is handled by the base damage
/// path, so we deal `(orb_count - 1)` additional hits here.
pub fn hook_damage_per_orb(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let orb_count = engine.state.orb_slots.occupied_count() as i32;
    let target_idx = ctx.target_idx;
    if orb_count > 1 && target_idx >= 0 && (target_idx as usize) < engine.state.enemies.len() {
        let tidx = target_idx as usize;
        let player_strength = engine.state.player.strength();
        let player_weak = engine.state.player.is_weak();
        let stance_mult = engine.state.stance.outgoing_mult();
        let enemy_vuln = engine.state.enemies[tidx].entity.is_vulnerable();
        let dmg = damage::calculate_damage(
            ctx.card.base_damage,
            player_strength + ctx.vigor,
            player_weak,
            stance_mult,
            enemy_vuln,
            false,
        );
        for _ in 0..(orb_count - 1) {
            if engine.state.enemies[tidx].entity.is_dead() {
                break;
            }
            engine.deal_damage_to_enemy(tidx, dmg);
        }
    }
}

/// Compile Driver: draw one card per unique orb type channeled.
pub fn hook_draw_per_unique_orb(engine: &mut CombatEngine, _ctx: &CardPlayContext) {
    let mut types = std::collections::HashSet::new();
    for orb in &engine.state.orb_slots.slots {
        if !orb.is_empty() {
            types.insert(orb.orb_type);
        }
    }
    let draw_count = types.len() as i32;
    if draw_count > 0 {
        engine.draw_cards(draw_count);
    }
}
