use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Common: Heavy Blade ---- (cost 2, 14 dmg, 3x str scaling; upgrade: 5x str)
    insert(cards, CardDef {
                id: "Heavy Blade", name: "Heavy Blade", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 14, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["heavy_blade"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Heavy Blade+", name: "Heavy Blade+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 14, base_block: -1,
                base_magic: 5, exhaust: false, enter_stance: None,
                effects: &["heavy_blade"], effect_data: &[], complex_hook: None,
            });
}
