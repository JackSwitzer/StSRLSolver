use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Common: Empty Fist ---- (cost 1, 9 dmg, exit stance; +5 upgrade)
    insert(cards, CardDef {
                id: "EmptyFist", name: "Empty Fist", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["exit_stance"], effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Simple(SE::ChangeStance(Stance::Neutral)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "EmptyFist+", name: "Empty Fist+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 14, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["exit_stance"], effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Simple(SE::ChangeStance(Stance::Neutral)),
                ], complex_hook: None,
            });
}
