use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Melter: 1 cost, 10 dmg, remove all enemy block
    insert(cards, CardDef {
                id: "Melter", name: "Melter", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["remove_enemy_block"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Melter+", name: "Melter+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 14, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["remove_enemy_block"], effect_data: &[], complex_hook: None,
            });
}
