use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Recursion.java queues RedoAction: evoke the front orb, then re-channel
    // that same orb instance. The upgrade changes only the cost from 1 to 0.
    // Java: reference/extracted/methods/card/Recursion.java
    insert(cards, CardDef {
                id: "Redo", name: "Recursion", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::EvokeAndRechannelFrontOrb),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Redo+", name: "Recursion+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::EvokeAndRechannelFrontOrb),
                ], complex_hook: None,
            });
}
