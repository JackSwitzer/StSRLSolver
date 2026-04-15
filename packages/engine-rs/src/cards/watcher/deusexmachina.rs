use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Deus Ex Machina ---- (cleanup-only empty shell; on-draw runtime hook covers the miracle/exhaust behavior. +1 magic upgrade)
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
