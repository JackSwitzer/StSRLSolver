use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Fear No Evil ---- (cost 1, 8 dmg, enter Calm if enemy attacking; +3 dmg upgrade)
    insert(cards, CardDef {
                id: "FearNoEvil", name: "Fear No Evil", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["calm_if_enemy_attacking"], effect_data: &[
                    E::Conditional(Cond::EnemyAttacking, &[E::Simple(SE::ChangeStance(Stance::Calm))], &[]),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "FearNoEvil+", name: "Fear No Evil+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 11, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["calm_if_enemy_attacking"], effect_data: &[
                    E::Conditional(Cond::EnemyAttacking, &[E::Simple(SE::ChangeStance(Stance::Calm))], &[]),
                ], complex_hook: None,
            });
}
