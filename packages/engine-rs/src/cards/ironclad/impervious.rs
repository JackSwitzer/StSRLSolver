use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Impervious queues one GainBlockAction using 30 card Block and Exhausts.
    // Upgrading adds exactly 10 Block without changing cost or Exhaust.
    // Java: reference/extracted/methods/card/Impervious.java
    insert(
        cards,
        CardDef {
            id: "Impervious",
            name: "Impervious",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 2,
            base_damage: -1,
            base_block: 30,
            base_magic: -1,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::GainBlock(A::Block))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Impervious+",
            name: "Impervious+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 2,
            base_damage: -1,
            base_block: 40,
            base_magic: -1,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::GainBlock(A::Block))],
            complex_hook: None,
        },
    );
}
