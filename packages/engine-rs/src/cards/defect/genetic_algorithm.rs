use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // GainBlockAction captures `block` while use() is enqueuing actions; the
    // earlier IncreaseMiscAction changes only this instance's future plays and
    // its matching master-deck card.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/GeneticAlgorithm.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/IncreaseMiscAction.java
    insert(
        cards,
        CardDef {
            id: "Genetic Algorithm",
            name: "Genetic Algorithm",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 1,
            base_magic: 2,
            exhaust: true,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::GainBlock(A::Block)),
                E::Simple(SE::ModifyPlayedCardBlock(A::Magic)),
            ],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Genetic Algorithm+",
            name: "Genetic Algorithm+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 1,
            base_magic: 3,
            exhaust: true,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::GainBlock(A::Block)),
                E::Simple(SE::ModifyPlayedCardBlock(A::Magic)),
            ],
            complex_hook: None,
        },
    );
}
