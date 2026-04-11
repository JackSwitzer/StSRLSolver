use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Rare: Wraith Form ---- (cost 3, power, +2 intangible, -1 dex/turn; +1 intangible)
    insert(cards, CardDef {
                id: "Wraith Form", name: "Wraith Form", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["wraith_form"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Wraith Form+", name: "Wraith Form+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["wraith_form"], effect_data: &[], complex_hook: None,
            });
}
