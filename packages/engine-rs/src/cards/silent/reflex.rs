use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Reflex.java makes the card unplayable at cost -2 and draws its magic
    // number only from triggerOnManualDiscard (2, or 3 when upgraded).
    // Java: reference/extracted/methods/card/Reflex.java
    insert(cards, CardDef {
                id: "Reflex", name: "Reflex", card_type: CardType::Skill,
                target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Reflex+", name: "Reflex+", card_type: CardType::Skill,
                target: CardTarget::None, cost: -2, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[], complex_hook: None,
            });
}
