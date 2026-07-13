use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Predator.java queues 15 Damage before applying two stacks of the shared
    // DrawCardNextTurnPower; upgrading adds only 5 damage.
    // Source: reference/extracted/methods/card/Predator.java
    insert(cards, CardDef {
                id: "Predator", name: "Predator", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 15, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::DRAW_CARD, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Predator+", name: "Predator+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 20, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::DRAW_CARD, A::Magic)),
                ], complex_hook: None,
            });
}
