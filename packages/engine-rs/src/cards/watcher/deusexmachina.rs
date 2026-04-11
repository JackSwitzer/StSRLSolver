use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Deus Ex Machina ---- (cost -2 (unplayable), skill, exhaust, on draw: add 2 Miracles to hand; +1 magic upgrade)
    insert(cards, CardDef {
                id: "DeusExMachina", name: "Deus Ex Machina", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -2, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effects: &["unplayable", "deus_ex_machina"], effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "DeusExMachina+", name: "Deus Ex Machina+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -2, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["unplayable", "deus_ex_machina"], effect_data: &[], complex_hook: None,
            });
}
