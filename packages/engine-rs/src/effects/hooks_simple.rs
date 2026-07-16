//! Small card hooks still used by the canonical runtime.

use crate::damage;
use crate::effects::types::CardPlayContext;
use crate::engine::CombatEngine;

/// Spirit Shield: gain block per card in hand.
pub fn hook_block_per_card_in_hand(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    let cards_in_hand = engine.state.hand.len() as i32;
    let per_card = ctx.card.base_magic.max(1);
    let dex = engine.state.player.dexterity();
    let frail = engine.state.player.is_frail();
    let block = damage::calculate_block(per_card * cards_in_hand, dex, frail);
    engine.gain_block_player(block);
}

/// Sentinel.triggerOnExhaust gains energy regardless of which effect exhausted it.
/// Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/Sentinel.java
pub fn hook_energy_on_exhaust(engine: &mut CombatEngine, ctx: &CardPlayContext) {
    engine.state.energy += ctx.card.base_magic;
}
