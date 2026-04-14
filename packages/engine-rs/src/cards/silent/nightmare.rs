use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Still hook-backed: Java's delayed copy/install path is in `NightmareAction`
    // and `NightmarePower`, which we cannot express without a runtime primitive.
    insert(cards, CardDef {
                id: "Nightmare", name: "Nightmare", card_type: CardType::Skill,
                target: CardTarget::None, cost: 3, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["nightmare"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_nightmare),
            });
    insert(cards, CardDef {
                id: "Nightmare+", name: "Nightmare+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["nightmare"], effect_data: &[], complex_hook: Some(crate::effects::hooks_complex::hook_nightmare),
            });
}
