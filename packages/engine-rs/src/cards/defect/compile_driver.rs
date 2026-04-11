use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Compile Driver: 1 cost, 7 dmg, draw 1 per unique orb
    insert(cards, CardDef {
                id: "Compile Driver", name: "Compile Driver", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["draw_per_unique_orb"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Compile Driver+", name: "Compile Driver+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["draw_per_unique_orb"], effect_data: &[], complex_hook: None,
            });
}
