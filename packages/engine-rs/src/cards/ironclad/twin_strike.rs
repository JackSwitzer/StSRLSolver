use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Common: Twin Strike ---- (cost 1, 5 dmg x2; +2 dmg)
    insert(cards, CardDef {
                id: "Twin Strike", name: "Twin Strike", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 5, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["multi_hit"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Twin Strike+", name: "Twin Strike+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["multi_hit"], effect_data: &[], complex_hook: None,
            });
}
