use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Java: DeusExMachina.java is unplayable and, when drawn, queues its
        // own exhaustion before creating 2 Miracles (3 when upgraded).
        // decompiled/java-src/com/megacrit/cardcrawl/cards/purple/DeusExMachina.java
        // ---- Rare: Deus Ex Machina ---- runtime-trigger card; on-draw miracle/exhaust lives in typed metadata. +1 magic upgrade
    insert(cards, CardDef {
                id: "DeusExMachina", name: "Deus Ex Machina", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -2, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "DeusExMachina+", name: "Deus Ex Machina+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -2, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effect_data: &[], complex_hook: None,
            });
}
