use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // EscapePlan.java draws one card, then EscapePlanAction checks that draw
    // action's actual drawnCards list for a Skill before gaining this.block.
    // Java: reference/extracted/methods/card/EscapePlan.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/EscapePlanAction.java
    insert(cards, CardDef {
        id: "Escape Plan", name: "Escape Plan", card_type: CardType::Skill,
        target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 3,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::DrawCards(A::Fixed(1))),
            E::Simple(SE::GainBlockIfLastDrawnCardType(CardType::Skill, A::Block)),
        ],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Escape Plan+", name: "Escape Plan+", card_type: CardType::Skill,
        target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 5,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::DrawCards(A::Fixed(1))),
            E::Simple(SE::GainBlockIfLastDrawnCardType(CardType::Skill, A::Block)),
        ],
        complex_hook: None,
    });
}
