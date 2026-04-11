use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Searing Blow ---- (cost 2, 12 dmg, can upgrade infinitely; +4+N per upgrade)
    insert(cards, CardDef {
                id: "Searing Blow", name: "Searing Blow", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 12, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["searing_blow"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Searing Blow+", name: "Searing Blow+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 16, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["searing_blow"], effect_data: &[], complex_hook: None,
            });
}
