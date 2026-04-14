use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Blocked on a card-owned current-block seeding primitive. Java's
        // IncreaseMiscAction mutates the played instance before future plays.
    insert(cards, CardDef {
                id: "Genetic Algorithm", name: "Genetic Algorithm", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effects: &["genetic_algorithm"], effect_data: &[],
                complex_hook: Some(crate::effects::hooks_complex::hook_genetic_algorithm),
            });
    insert(cards, CardDef {
                id: "Genetic Algorithm+", name: "Genetic Algorithm+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 0,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["genetic_algorithm"], effect_data: &[],
                complex_hook: Some(crate::effects::hooks_complex::hook_genetic_algorithm),
            });
}
