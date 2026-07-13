use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ReinforcedBody.java passes its modified Block to ReinforcedBodyAction,
    // which queues one GainBlockAction per energy used. The upgrade adds 2.
    // Java: reference/extracted/methods/card/ReinforcedBody.java
    insert(cards, CardDef {
                id: "Reinforced Body", name: "Reinforced Body", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: 7,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::GainBlock(A::Block)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Reinforced Body+", name: "Reinforced Body+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: 9,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::GainBlock(A::Block)),
                ], complex_hook: None,
            });
}
