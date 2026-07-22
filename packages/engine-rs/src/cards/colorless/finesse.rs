use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Finesse.java queues modified card Block, then a literal one-card draw;
    // the card has no magicNumber.
    // Java: reference/extracted/methods/card/Finesse.java
    insert(
        cards,
        CardDef {
            id: "Finesse",
            name: "Finesse",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: 2,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DrawCards(A::Fixed(1)))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Finesse+",
            name: "Finesse+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: 4,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DrawCards(A::Fixed(1)))],
            complex_hook: None,
        },
    );
}
