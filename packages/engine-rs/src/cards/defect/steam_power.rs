use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Overclock.use queues DrawCardAction before creating one base Burn in
    // discard; upgradeMagicNumber(1) changes only the draw count from 2 to 3.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Overclock.java
    insert(
        cards,
        CardDef {
            id: "Steam Power",
            name: "Overclock",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
                E::Simple(SE::AddCard("Burn", P::Discard, A::Fixed(1))),
            ],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Steam Power+",
            name: "Overclock+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 3,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
                E::Simple(SE::AddCard("Burn", P::Discard, A::Fixed(1))),
            ],
            complex_hook: None,
        },
    );
}
