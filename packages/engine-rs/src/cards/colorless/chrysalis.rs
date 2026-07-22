use crate::cards::prelude::*;
use crate::effects::declarative::{
    AmountSource as A, Effect as E, GeneratedCardPool, GeneratedCostRule,
};

static CHRYSALIS: [E; 1] = [E::GenerateRandomCardsToDraw {
    pool: GeneratedCardPool::Skill,
    count: A::Magic,
    cost_rule: GeneratedCostRule::ZeroIfPositiveThisTurn,
}];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Source: reference/extracted/methods/card/Chrysalis.java
    // Three random current-character Skills (five upgraded) are made zero-cost
    // when positive, inserted at random draw-pile positions, and exhaust Chrysalis.
    insert(
        cards,
        CardDef {
            id: "Chrysalis",
            name: "Chrysalis",
            card_type: CardType::Skill,
            target: CardTarget::None,
            cost: 2,
            base_damage: -1,
            base_block: -1,
            base_magic: 3,
            exhaust: true,
            enter_stance: None,
            effect_data: &CHRYSALIS,
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Chrysalis+",
            name: "Chrysalis+",
            card_type: CardType::Skill,
            target: CardTarget::None,
            cost: 2,
            base_damage: -1,
            base_block: -1,
            base_magic: 5,
            exhaust: true,
            enter_stance: None,
            effect_data: &CHRYSALIS,
            complex_hook: None,
        },
    );
}
