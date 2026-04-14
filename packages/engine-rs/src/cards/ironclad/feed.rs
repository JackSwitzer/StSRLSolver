use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Rare: Feed ---- (cost 1, 10 dmg, exhaust, +3 max HP on kill; +2/+1)
    insert(cards, CardDef {
                id: "Feed", name: "Feed", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &[],
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Conditional(
                        Cond::EnemyKilled,
                        &[E::Simple(SE::ModifyMaxHp(A::Magic))],
                        &[],
                    ),
                ],
                complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Feed+", name: "Feed+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
                base_magic: 4, exhaust: true, enter_stance: None,
                effects: &[],
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Conditional(
                        Cond::EnemyKilled,
                        &[E::Simple(SE::ModifyMaxHp(A::Magic))],
                        &[],
                    ),
                ],
                complex_hook: None,
            });
}
