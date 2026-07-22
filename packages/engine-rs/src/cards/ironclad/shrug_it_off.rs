use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ShrugItOff.java queues 8 Block followed by drawing exactly one card;
    // upgradeBlock(3) is its only upgrade change.
    // Java: reference/extracted/methods/card/ShrugItOff.java
    insert(
        cards,
        CardDef {
            id: "Shrug It Off",
            name: "Shrug It Off",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 8,
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
            id: "Shrug It Off+",
            name: "Shrug It Off+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 11,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DrawCards(A::Fixed(1)))],
            complex_hook: None,
        },
    );
}
