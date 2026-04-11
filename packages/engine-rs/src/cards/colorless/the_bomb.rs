use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // The Bomb: 2 cost, deal 40 dmg to all enemies in 3 turns
    insert(cards, CardDef {
                id: "The Bomb", name: "The Bomb", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 40, exhaust: false, enter_stance: None,
                effects: &["the_bomb"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "The Bomb+", name: "The Bomb+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 50, exhaust: false, enter_stance: None,
                effects: &["the_bomb"], effect_data: &[], complex_hook: None,
            });
}
