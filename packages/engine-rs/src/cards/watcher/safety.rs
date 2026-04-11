use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Safety (from Deceive Reality): cost 1, 12 block, retain, exhaust
    insert(cards, CardDef {
                id: "Safety", name: "Safety", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 12,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["retain"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Safety+", name: "Safety+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 16,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["retain"], effect_data: &[], complex_hook: None,
            });
}
