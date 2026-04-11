use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Rebound: 1 cost, 9 dmg, next card drawn goes to top of draw pile
    insert(cards, CardDef {
                id: "Rebound", name: "Rebound", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["next_card_to_top"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Rebound+", name: "Rebound+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["next_card_to_top"], effect_data: &[], complex_hook: None,
            });
}
