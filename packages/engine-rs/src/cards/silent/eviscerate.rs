use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Eviscerate.java queues three damage actions, reduces its current cost on
    // each discard, and upgrades damage by two.
    // Java: reference/extracted/methods/card/Eviscerate.java
    insert(
        cards,
        CardDef {
            id: "Eviscerate",
            name: "Eviscerate",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 3,
            base_damage: 7,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::ExtraHits(A::Fixed(3))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Eviscerate+",
            name: "Eviscerate+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 3,
            base_damage: 9,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::ExtraHits(A::Fixed(3))],
            complex_hook: None,
        },
    );
}
