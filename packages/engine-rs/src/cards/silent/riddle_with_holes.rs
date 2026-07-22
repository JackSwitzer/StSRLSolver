use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // RiddleWithHoles.java uses canonical ID "Riddle With Holes" and queues
    // five DamageActions at 3 damage; upgrading adds 1 damage to every hit.
    // Java: reference/extracted/methods/card/RiddleWithHoles.java
    insert(
        cards,
        CardDef {
            id: "Riddle With Holes",
            name: "Riddle with Holes",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 2,
            base_damage: 3,
            base_block: -1,
            base_magic: 5,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::ExtraHits(A::Magic)],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Riddle With Holes+",
            name: "Riddle with Holes+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 2,
            base_damage: 4,
            base_block: -1,
            base_magic: 5,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::ExtraHits(A::Magic)],
            complex_hook: None,
        },
    );
}
