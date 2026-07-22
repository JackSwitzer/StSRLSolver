use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Seek.java
    // BetterDrawPileToHandAction mandatorily moves one card (two when upgraded)
    // from the draw pile, and Seek always Exhausts.
    insert(
        cards,
        CardDef {
            id: "Seek",
            name: "Seek",
            card_type: CardType::Skill,
            target: CardTarget::None,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 1,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::ChooseCards {
                source: P::Draw,
                filter: CardFilter::All,
                action: ChoiceAction::MoveToHand,
                min_picks: A::Magic,
                max_picks: A::Magic,
                post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
            }],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Seek+",
            name: "Seek+",
            card_type: CardType::Skill,
            target: CardTarget::None,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 2,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::ChooseCards {
                source: P::Draw,
                filter: CardFilter::All,
                action: ChoiceAction::MoveToHand,
                min_picks: A::Magic,
                max_picks: A::Magic,
                post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
            }],
            complex_hook: None,
        },
    );
}
