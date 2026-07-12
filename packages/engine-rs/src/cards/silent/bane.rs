use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Source: cards/green/Bane.java queues 7 damage, then BaneAction queues
        // the same damage again only if the living target has Poison; +3 damage.
    insert(cards, CardDef {
                id: "Bane", name: "Bane", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Conditional(
                        Cond::EnemyAlive,
                        &[E::Conditional(
                            Cond::EnemyHasStatus(sid::POISON),
                            &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
                            &[],
                        )],
                        &[],
                    ),
                ],
                complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Bane+", name: "Bane+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Conditional(
                        Cond::EnemyAlive,
                        &[E::Conditional(
                            Cond::EnemyHasStatus(sid::POISON),
                            &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
                            &[],
                        )],
                        &[],
                    ),
                ],
                complex_hook: None,
            });
}
