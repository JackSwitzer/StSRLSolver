use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Envenom.java has no magic number and applies a literal one stack of
    // EnvenomPower. Upgrade changes only cost from 2 to 1.
    // Java: reference/extracted/methods/card/Envenom.java
    insert(cards, CardDef {
                id: "Envenom", name: "Envenom", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::ENVENOM, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Envenom+", name: "Envenom+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::ENVENOM, A::Fixed(1))),
                ], complex_hook: None,
            });
}
