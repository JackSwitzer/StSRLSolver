use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Rare: A Thousand Cuts ---- (cost 2, power, deal 1 dmg per card played; +1)
    insert(cards, CardDef {
                id: "A Thousand Cuts", name: "A Thousand Cuts", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["thousand_cuts"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "A Thousand Cuts+", name: "A Thousand Cuts+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["thousand_cuts"], effect_data: &[], complex_hook: None,
            });
}
