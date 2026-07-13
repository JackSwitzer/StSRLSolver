use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // RitualDaggerAction permanently raises the matching master-deck card and
    // its same-UUID combat instance by magicNumber on a non-minion kill. The
    // upgrade raises only that growth from 3 to 5.
    // Java: reference/extracted/methods/card/RitualDagger.java
    insert(cards, CardDef {
                id: "RitualDagger", name: "Ritual Dagger", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 15, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Conditional(
                        Cond::EnemyKilledNonMinion,
                        &[E::Simple(SE::ModifyPlayedCardDamage(A::Magic))],
                        &[],
                    ),
                ],
                complex_hook: None,
            });
    insert(cards, CardDef {
                id: "RitualDagger+", name: "Ritual Dagger+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 15, base_block: -1,
                base_magic: 5, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Conditional(
                        Cond::EnemyKilledNonMinion,
                        &[E::Simple(SE::ModifyPlayedCardDamage(A::Magic))],
                        &[],
                    ),
                ],
                complex_hook: None,
            });
}
