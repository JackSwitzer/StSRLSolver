use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Transmutation: X cost, add X random colorless cards to hand, exhaust
    insert(cards, CardDef {
                id: "Transmutation", name: "Transmutation", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["transmutation"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Transmutation+", name: "Transmutation+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["transmutation"], effect_data: &[], complex_hook: None,
            });
}
