//! on_retain hooks — fired at end of turn for cards that stay in hand.

use crate::cards::CardDef;
use crate::combat_types::CardInstance;
use crate::engine::CombatEngine;
use crate::status_ids::sid;

/// Sands of Time: reduce cost by 1 when retained.
pub fn hook_reduce_cost_on_retain(_engine: &mut CombatEngine, card_inst: &mut CardInstance, _card: &CardDef) {
    card_inst.cost = (card_inst.cost - 1).max(0);
}

/// Perseverance: grow block bonus when retained.
pub fn hook_grow_block_on_retain(engine: &mut CombatEngine, _card_inst: &mut CardInstance, card: &CardDef) {
    engine.state.player.add_status(sid::PERSEVERANCE_BONUS, card.base_magic);
}

/// Windmill Strike: grow damage bonus when retained.
pub fn hook_grow_damage_on_retain(engine: &mut CombatEngine, _card_inst: &mut CardInstance, card: &CardDef) {
    engine.state.player.add_status(sid::WINDMILL_STRIKE_BONUS, card.base_magic);
}
