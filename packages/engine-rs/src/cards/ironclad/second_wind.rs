use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // SecondWind.java costs 1 and passes card block 5 (7 upgraded) to
    // BlockPerNonAttackAction, which snapshots every non-Attack in hand, queues
    // their exhausts, then one GainBlockAction per snapshotted card.
    // Java: reference/extracted/methods/card/SecondWind.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/
    // BlockPerNonAttackAction.java
    insert(cards, CardDef {
        id: "Second Wind",
        name: "Second Wind",
        card_type: CardType::Skill,
        target: CardTarget::SelfTarget,
        cost: 1,
        base_damage: -1,
        base_block: 5,
        base_magic: -1,
        exhaust: false,
        enter_stance: None,
                effect_data: &[
            E::ForEachInPile {
                pile: P::Hand,
                filter: CardFilter::NonAttacks,
                action: crate::effects::declarative::BulkAction::Exhaust,
            },
            E::Simple(SE::GainBlock(A::LastBulkCountTimesBlock)),
        ],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Second Wind+",
        name: "Second Wind+",
        card_type: CardType::Skill,
        target: CardTarget::SelfTarget,
        cost: 1,
        base_damage: -1,
        base_block: 7,
        base_magic: -1,
        exhaust: false,
        enter_stance: None,
                effect_data: &[
            E::ForEachInPile {
                pile: P::Hand,
                filter: CardFilter::NonAttacks,
                action: crate::effects::declarative::BulkAction::Exhaust,
            },
            E::Simple(SE::GainBlock(A::LastBulkCountTimesBlock)),
        ],
        complex_hook: None,
    });
}
