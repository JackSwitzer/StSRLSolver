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

/// Deus Ex Machina: when drawn, add N Miracles to hand and exhaust self.
/// N = base_magic of the card (2 base, 3 upgraded).
/// Routed through the canonical on_draw registry dispatch.
pub fn hook_deus_ex_machina_on_draw(engine: &mut CombatEngine, card_inst: CardInstance) {
    let card_def = engine.card_registry.card_def_by_id(card_inst.def_id);
    let miracle_count = card_def.base_magic.max(1);

    // Remove this card from hand (it was just drawn) and exhaust it
    if let Some(pos) = engine.state.hand.iter().rposition(|c| c.def_id == card_inst.def_id) {
        let removed = engine.state.hand.remove(pos);
        engine.state.exhaust_pile.push(removed);
        engine.trigger_on_exhaust();
    }

    // Add Miracles to hand
    for _ in 0..miracle_count {
        if engine.state.hand.len() < 10 {
            let miracle = engine.temp_card("Miracle");
            engine.state.hand.push(miracle);
        }
    }
}
