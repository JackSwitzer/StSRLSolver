use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Choke.java queues 12 damage, then applies ChokePower for 3 (5 upgraded).
        // ChokePower's on-use and start-turn behavior is resolved by the engine.
        // Source: reference/extracted/methods/card/Choke.java
    insert(cards, CardDef {
                id: "Choke", name: "Choke", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 12, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::CONSTRICTED, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Choke+", name: "Choke+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 12, base_block: -1,
                base_magic: 5, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::SelectedEnemy, sid::CONSTRICTED, A::Magic)),
                ], complex_hook: None,
            });
}
