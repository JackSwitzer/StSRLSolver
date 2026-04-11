use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Metamorphosis: 2 cost, shuffle 3 random upgraded Attacks into draw pile, exhaust
    insert(cards, CardDef {
                id: "Metamorphosis", name: "Metamorphosis", card_type: CardType::Skill,
                target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["add_random_attacks_to_draw"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Metamorphosis+", name: "Metamorphosis+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 5, exhaust: true, enter_stance: None,
                effects: &["add_random_attacks_to_draw"], effect_data: &[], complex_hook: None,
            });
}
