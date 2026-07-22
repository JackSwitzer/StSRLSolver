use crate::cards::prelude::*;
use crate::effects::declarative::{
    AmountSource as A, Effect as E, SimpleEffect as SE, Target as T,
};

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // FTLAction subtracts the current FTL from cardsPlayedThisTurn, draws one
    // card when the prior count is below magicNumber, then deals damage.
    // Java: reference/extracted/methods/card/FTL.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/FTLAction.java
    insert(
        cards,
        CardDef {
            id: "FTL",
            name: "FTL",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 0,
            base_damage: 5,
            base_block: -1,
            base_magic: 3,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Conditional(
                    Cond::CardsPlayedThisTurnLessThan(4),
                    &[E::Simple(SE::DrawCards(A::Fixed(1)))],
                    &[],
                ),
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            ],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "FTL+",
            name: "FTL+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 0,
            base_damage: 6,
            base_block: -1,
            base_magic: 4,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Conditional(
                    Cond::CardsPlayedThisTurnLessThan(5),
                    &[E::Simple(SE::DrawCards(A::Fixed(1)))],
                    &[],
                ),
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
            ],
            complex_hook: None,
        },
    );
}
