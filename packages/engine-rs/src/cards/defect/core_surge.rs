use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Source: reference/extracted/methods/card/CoreSurge.java queues 11 damage
    // before 1 Artifact and Exhausts; its upgrade changes only damage by +4.
    insert(
        cards,
        CardDef {
            id: "Core Surge",
            name: "Core Surge",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 11,
            base_block: -1,
            base_magic: 1,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::ARTIFACT, A::Magic))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Core Surge+",
            name: "Core Surge+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 15,
            base_block: -1,
            base_magic: 1,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::ARTIFACT, A::Magic))],
            complex_hook: None,
        },
    );
}
