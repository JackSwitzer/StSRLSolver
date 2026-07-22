use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // TrueGrit.java gains 7 Block then randomly exhausts one hand card. Upgrade
    // raises Block to 9 and changes only the exhaust from random to selected.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/TrueGrit.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/ExhaustAction.java
    insert(
        cards,
        CardDef {
            id: "True Grit",
            name: "True Grit",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 7,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::GainBlock(A::Block)),
                E::Simple(SE::ExhaustRandomCardFromHand),
            ],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "True Grit+",
            name: "True Grit+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 9,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::GainBlock(A::Block)),
                E::ChooseCards {
                    source: P::Hand,
                    filter: crate::effects::declarative::CardFilter::All,
                    action: crate::effects::declarative::ChoiceAction::Exhaust,
                    min_picks: A::Fixed(1),
                    max_picks: A::Fixed(1),
                    post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
                },
            ],
            complex_hook: None,
        },
    );
}
