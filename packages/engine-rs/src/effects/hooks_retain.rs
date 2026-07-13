//! on_retain hooks — fired at end of turn for cards that stay in hand.

use crate::cards::CardDef;
use crate::combat_types::CardInstance;
use crate::engine::CombatEngine;

/// Sands of Time: reduce cost by 1 when retained.
pub fn hook_reduce_cost_on_retain(_engine: &mut CombatEngine, card_inst: &mut CardInstance, _card: &CardDef) {
    let current_cost = if card_inst.cost >= 0 {
        card_inst.cost
    } else {
        _card.cost as i8
    };
    card_inst.set_permanent_cost((current_cost - 1).max(0));
}

/// Perseverance: grow block bonus when retained.
pub fn hook_grow_block_on_retain(_engine: &mut CombatEngine, card_inst: &mut CardInstance, card: &CardDef) {
    // Perseverance.onRetained calls upgradeBlock on this exact AbstractCard.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Perseverance.java
    let current_block = if card_inst.misc >= 0 {
        card_inst.misc
    } else {
        card.base_block as i16
    };
    card_inst.misc = current_block + card.base_magic as i16;
}

/// Windmill Strike: grow damage bonus when retained.
pub fn hook_grow_damage_on_retain(_engine: &mut CombatEngine, card_inst: &mut CardInstance, card: &CardDef) {
    let current_damage = if card_inst.misc >= 0 {
        card_inst.misc
    } else {
        card.base_damage as i16
    };
    card_inst.misc = current_damage + card.base_magic as i16;
}
