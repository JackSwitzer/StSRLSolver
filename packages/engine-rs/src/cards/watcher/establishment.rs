use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Establishment ---- (cost 1, power, retained cards cost 1 less; upgrade: innate)
    insert(cards, CardDef {
                id: "Establishment", name: "Establishment", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["establishment"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Establishment+", name: "Establishment+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["establishment", "innate"], effect_data: &[], complex_hook: None,
            });
}
