use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // FeedAction grants max HP only when its damage kills a target that is
    // neither half-dead nor a Minion.
    // Java: reference/extracted/methods/card/Feed.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/FeedAction.java
    insert(
        cards,
        CardDef {
            id: "Feed",
            name: "Feed",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 10,
            base_block: -1,
            base_magic: 3,
            exhaust: true,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                E::Conditional(
                    Cond::EnemyKilledNonMinion,
                    &[E::Simple(SE::ModifyMaxHp(A::Magic))],
                    &[],
                ),
            ],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Feed+",
            name: "Feed+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 12,
            base_block: -1,
            base_magic: 4,
            exhaust: true,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                E::Conditional(
                    Cond::EnemyKilledNonMinion,
                    &[E::Simple(SE::ModifyMaxHp(A::Magic))],
                    &[],
                ),
            ],
            complex_hook: None,
        },
    );
}
