use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // MakeTempCardInDrawPileAction inserts base Through Violence at a
    // cardRandomRng-selected draw index after damage; upgrade adds 5 damage.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/ReachHeaven.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/MakeTempCardInDrawPileAction.java
    insert(
        cards,
        CardDef {
            id: "ReachHeaven",
            name: "Reach Heaven",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 2,
            base_damage: 10,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddCardToRandomDrawSpot(
                "ThroughViolence",
                A::Fixed(1),
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "ReachHeaven+",
            name: "Reach Heaven+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 2,
            base_damage: 15,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddCardToRandomDrawSpot(
                "ThroughViolence",
                A::Fixed(1),
            ))],
            complex_hook: None,
        },
    );
}
