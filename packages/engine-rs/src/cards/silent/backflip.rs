use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Source: cards/green/Backflip.java costs 1, queues block before drawing
    // exactly 2 cards, and upgrades only its base block from 5 to 8.
    insert(
        cards,
        CardDef {
            id: "Backflip",
            name: "Backflip",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 5,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DrawCards(A::Magic))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Backflip+",
            name: "Backflip+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 8,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DrawCards(A::Magic))],
            complex_hook: None,
        },
    );
}
