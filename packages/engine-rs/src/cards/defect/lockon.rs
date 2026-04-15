use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Lock-On (Java ID: Lockon): 1 cost, 8 dmg, apply 2 Lock-On
    insert(cards, CardDef {
                id: "Lockon", name: "Lock-On", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::LOCK_ON, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Lockon+", name: "Lock-On+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 11, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::LOCK_ON, A::Magic)),
                ], complex_hook: None,
            });
}
