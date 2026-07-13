use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Tactician.java has an empty use(), is permanently unplayable, and its
        // manual-discard hook gains magicNumber energy (1, upgraded to 2).
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/green/Tactician.java
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
