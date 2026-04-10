//! on_draw hooks — fired when a card is drawn into hand.

use crate::combat_types::CardInstance;
use crate::engine::CombatEngine;

/// Void: lose 1 energy when drawn.
pub fn hook_lose_energy_on_draw(engine: &mut CombatEngine, _card_inst: CardInstance) {
    engine.state.energy = (engine.state.energy - 1).max(0);
}

/// Endless Agony: add a copy to hand when drawn.
pub fn hook_copy_on_draw(engine: &mut CombatEngine, card_inst: CardInstance) {
    if engine.state.hand.len() < 10 {
        engine.state.hand.push(card_inst);
    }
}
