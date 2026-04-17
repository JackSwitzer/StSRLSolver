use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Common: Poisoned Stab ---- (cost 1, 6 dmg, 3 poison; +1/+1)
    insert(cards, CardDef {
                id: "Poisoned Stab", name: "Poisoned Stab", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::POISON, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Poisoned Stab+", name: "Poisoned Stab+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::POISON, A::Magic)),
                ], complex_hook: None,
            });
}
