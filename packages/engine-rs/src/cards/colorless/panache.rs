use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Panache: 0 cost, power, deal 10 dmg to all every 5th card played per turn
    insert(cards, CardDef {
                id: "Panache", name: "Panache", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 10, exhaust: false, enter_stance: None,
                effects: &["panache"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Panache+", name: "Panache+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 14, exhaust: false, enter_stance: None,
                effects: &["panache"], effect_data: &[], complex_hook: None,
            });
}
