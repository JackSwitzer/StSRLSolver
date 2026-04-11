use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Genetic Algorithm: 1 cost, block from misc (starts 0), grows +2 per combat, exhaust
    insert(cards, CardDef {
                id: "Genetic Algorithm", name: "Genetic Algorithm", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 0,
                base_magic: 2, exhaust: true, enter_stance: None,
                effects: &["genetic_algorithm"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Genetic Algorithm+", name: "Genetic Algorithm+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 0,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["genetic_algorithm"], effect_data: &[], complex_hook: None,
            });
}
