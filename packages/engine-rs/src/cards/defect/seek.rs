use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Seek: 0 cost, choose 1 card from draw pile and put into hand, exhaust
    insert(cards, CardDef {
                id: "Seek", name: "Seek", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: true, enter_stance: None,
                effects: &["seek"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Seek+", name: "Seek+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effects: &["seek"], effect_data: &[], complex_hook: None,
            });
}
