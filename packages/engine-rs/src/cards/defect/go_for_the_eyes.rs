use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Go for the Eyes: 0 cost, 3 dmg, apply Weak if attacking
    insert(cards, CardDef {
                id: "Go for the Eyes", name: "Go for the Eyes", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 3, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["weak_if_attacking"], effect_data: &[
                    E::Conditional(Cond::EnemyAttacking, &[E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic))], &[]),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Go for the Eyes+", name: "Go for the Eyes+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["weak_if_attacking"], effect_data: &[
                    E::Conditional(Cond::EnemyAttacking, &[E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic))], &[]),
                ], complex_hook: None,
            });
}
