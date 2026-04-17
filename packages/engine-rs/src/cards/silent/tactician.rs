use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Tactician ---- runtime-trigger card; energy-on-discard lives in typed metadata. +1
    insert(cards, CardDef {
                id: "Tactician", name: "Tactician", card_type: CardType::Skill,
                target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Tactician+", name: "Tactician+", card_type: CardType::Skill,
                target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
            });
}
