use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Boot Sequence: 0 cost, 10 block, innate, exhaust
    insert(cards, CardDef {
                id: "BootSequence", name: "Boot Sequence", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 10,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["innate"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "BootSequence+", name: "Boot Sequence+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 13,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["innate"], effect_data: &[], complex_hook: None,
            });
}
