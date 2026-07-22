use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Hologram queues GainBlockAction before the mandatory one-card
    // BetterDiscardPileToHandAction. The latter is a no-op on an empty pile
    // and directly moves a singleton without opening grid selection. Upgrade
    // adds 2 Block and removes Exhaust.
    // Java: reference/extracted/methods/card/Hologram.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/BetterDiscardPileToHandAction.java
    insert(
        cards,
        CardDef {
            id: "Hologram",
            name: "Hologram",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 3,
            base_magic: -1,
            exhaust: true,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::GainBlock(A::Block)),
                E::ChooseCards {
                    source: P::Discard,
                    filter: CardFilter::All,
                    action: ChoiceAction::MoveToHand,
                    min_picks: A::Fixed(1),
                    max_picks: A::Fixed(1),
                    post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
                },
            ],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Hologram+",
            name: "Hologram+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 5,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::GainBlock(A::Block)),
                E::ChooseCards {
                    source: P::Discard,
                    filter: CardFilter::All,
                    action: ChoiceAction::MoveToHand,
                    min_picks: A::Fixed(1),
                    max_picks: A::Fixed(1),
                    post_choice_draw: crate::effects::declarative::AmountSource::Fixed(0),
                },
            ],
            complex_hook: None,
        },
    );
}
