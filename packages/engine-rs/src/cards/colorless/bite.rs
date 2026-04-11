use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Bite: 1 cost, 7 dmg, heal 2
    insert(cards, CardDef {
                id: "Bite", name: "Bite", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["heal_on_play"], effect_data: &[
                    E::Simple(SE::HealHp(T::Player, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Bite+", name: "Bite+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["heal_on_play"], effect_data: &[
                    E::Simple(SE::HealHp(T::Player, A::Magic)),
                ], complex_hook: None,
            });
}
