use crate::cards::prelude::*;
use crate::effects::declarative::{
    AmountSource as A, CardFilter, Effect as E, Pile as P, SimpleEffect as SE,
};

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // DrawPileToHandAction moves three random Attacks from draw to hand (four
    // upgraded), discarding selected overflow at the ten-card hand cap.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Violence.java
    // and actions/utility/DrawPileToHandAction.java.
    insert(
        cards,
        CardDef {
            id: "Violence",
            name: "Violence",
            card_type: CardType::Skill,
            target: CardTarget::None,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 3,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DrawRandomCardsFromPileToHand(
                P::Draw,
                CardFilter::Attacks,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Violence+",
            name: "Violence+",
            card_type: CardType::Skill,
            target: CardTarget::None,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 4,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DrawRandomCardsFromPileToHand(
                P::Draw,
                CardFilter::Attacks,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
}
