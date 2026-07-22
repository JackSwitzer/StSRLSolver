use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // PommelStrike.java queues 9 Damage before drawing 1; upgrade adds one to
    // both values. Its Java STRIKE tag is represented by the canonical name.
    // Source: reference/extracted/methods/card/PommelStrike.java
    insert(
        cards,
        CardDef {
            id: "Pommel Strike",
            name: "Pommel Strike",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 9,
            base_block: -1,
            base_magic: 1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DrawCards(A::Magic))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Pommel Strike+",
            name: "Pommel Strike+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 10,
            base_block: -1,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DrawCards(A::Magic))],
            complex_hook: None,
        },
    );
}
