use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "RitualDagger", name: "Ritual Dagger", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 15, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Conditional(
                        Cond::EnemyKilled,
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
                        Cond::EnemyKilled,
                        &[E::Simple(SE::ModifyPlayedCardDamage(A::Magic))],
                        &[],
                    ),
                ],
                complex_hook: None,
            });
}
