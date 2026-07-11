use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // MakeTempCardInDrawPileAction(..., randomSpot=true) inserts Insight at a
    // cardRandomRng-selected index; it does not shuffle the existing pile.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Pray.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/MakeTempCardInDrawPileAction.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/CardGroup.java
    insert(cards, CardDef {
                id: "Pray", name: "Pray", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::GainMantra(A::Magic)),
                    E::Simple(SE::AddCardToRandomDrawSpot("Insight", A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Pray+", name: "Pray+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::GainMantra(A::Magic)),
                    E::Simple(SE::AddCardToRandomDrawSpot("Insight", A::Fixed(1))),
                ], complex_hook: None,
            });
}
