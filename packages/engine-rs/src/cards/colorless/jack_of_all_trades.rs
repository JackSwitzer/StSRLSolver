use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Jack of All Trades: 0 cost, add 1 random colorless card to hand, exhaust
    insert(cards, CardDef {
                id: "Jack Of All Trades", name: "Jack Of All Trades", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: true, enter_stance: None,
                effects: &["add_random_colorless"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Jack Of All Trades+", name: "Jack Of All Trades+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effects: &["add_random_colorless"], effect_data: &[], complex_hook: None,
            });
}
