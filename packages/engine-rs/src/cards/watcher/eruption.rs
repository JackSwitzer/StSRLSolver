use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Eruption", name: "Eruption", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 9, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: Some("Wrath"), effects: &[], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Eruption+", name: "Eruption+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: Some("Wrath"), effects: &[], effect_data: &[], complex_hook: None,
            });
}
