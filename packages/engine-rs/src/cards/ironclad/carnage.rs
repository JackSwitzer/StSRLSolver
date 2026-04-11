use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Carnage ---- (cost 2, 20 dmg, ethereal; +8 dmg)
    insert(cards, CardDef {
                id: "Carnage", name: "Carnage", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 20, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["ethereal"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Carnage+", name: "Carnage+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 28, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["ethereal"], effect_data: &[], complex_hook: None,
            });
}
