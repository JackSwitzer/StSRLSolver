use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Hello World: 1 cost, power, add random common card to hand each turn (upgrade: innate)
    insert(cards, CardDef {
                id: "Hello World", name: "Hello World", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["hello_world"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Hello World+", name: "Hello World+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["hello_world", "innate"], effect_data: &[], complex_hook: None,
            });
}
