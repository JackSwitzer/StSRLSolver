use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Forethought: 0 cost, put card from hand to bottom of draw pile at 0 cost
    insert(cards, CardDef {
                id: "Forethought", name: "Forethought", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["forethought"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Forethought+", name: "Forethought+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["forethought_all"], effect_data: &[], complex_hook: None,
            });
}
