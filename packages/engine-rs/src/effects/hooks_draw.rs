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
    } else {
        // EndlessAgony.triggerWhenDrawn queues MakeTempCardInHandAction, which
        // puts copies beyond the ten-card hand limit in the discard pile.
        // Java: reference/extracted/methods/card/EndlessAgony.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/MakeTempCardInHandAction.java
        engine.state.discard_pile.push(card_inst);
    }
}

/// Deus Ex Machina: when drawn, add N Miracles to hand and exhaust self.
/// N = base_magic of the card (2 base, 3 upgraded).
/// Routed through the canonical on_draw registry dispatch.
pub fn hook_deus_ex_machina_on_draw(engine: &mut CombatEngine, card_inst: CardInstance) {
    let card_def = engine.card_registry.card_def_by_id(card_inst.def_id);
    let miracle_count = card_def.base_magic.max(1);

    // DeusExMachina.java adds the exhaust action to the top after adding the
    // creation action to the top, so exhaustion resolves first.
    if let Some(pos) = engine.state.hand.iter().rposition(|c| c.def_id == card_inst.def_id) {
        let removed = engine.state.hand.remove(pos);
        engine.state.exhaust_pile.push(removed);
        engine.trigger_card_on_exhaust(removed);
    }

    // MakeTempCardInHandAction sends generated cards beyond the hand limit to
    // discard rather than deleting them.
    // decompiled/java-src/com/megacrit/cardcrawl/actions/common/MakeTempCardInHandAction.java
    engine.add_temp_cards_to_hand("Miracle", miracle_count);
}
