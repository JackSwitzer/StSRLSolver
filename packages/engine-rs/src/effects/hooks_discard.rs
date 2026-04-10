//! on_discard hooks — fired when a card is manually discarded from hand.

use crate::combat_types::CardInstance;
use crate::engine::CombatEngine;
use super::types::OnDiscardEffect;

/// Reflex: draw cards when discarded.
pub fn hook_draw_on_discard(engine: &mut CombatEngine, card_inst: CardInstance) -> OnDiscardEffect {
    let card_def = engine.card_registry.card_def_by_id(card_inst.def_id);
    OnDiscardEffect {
        draw: card_def.base_magic,
        energy: 0,
    }
}

/// Tactician: gain energy when discarded.
pub fn hook_energy_on_discard(engine: &mut CombatEngine, card_inst: CardInstance) -> OnDiscardEffect {
    let card_def = engine.card_registry.card_def_by_id(card_inst.def_id);
    OnDiscardEffect {
        draw: 0,
        energy: card_def.base_magic,
    }
}
