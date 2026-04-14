use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Melter", name: "Melter", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["remove_enemy_block"],
                effect_data: &[
                    E::Simple(SE::RemoveEnemyBlock(T::SelectedEnemy)),
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                ],
                complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Melter+", name: "Melter+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 14, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["remove_enemy_block"],
                effect_data: &[
                    E::Simple(SE::RemoveEnemyBlock(T::SelectedEnemy)),
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                ],
                complex_hook: None,
            });
}
