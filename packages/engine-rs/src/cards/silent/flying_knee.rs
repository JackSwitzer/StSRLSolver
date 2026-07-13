use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // FlyingKnee.java deals 8 damage, then applies EnergizedPower with a
        // literal amount of 1; the card has no magicNumber. Upgrading adds
        // only 3 damage.
        // Java: reference/extracted/methods/card/FlyingKnee.java
    insert(cards, CardDef {
                id: "Flying Knee", name: "Flying Knee", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::ENERGIZED, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Flying Knee+", name: "Flying Knee+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 11, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::ENERGIZED, A::Fixed(1))),
                ], complex_hook: None,
            });
}
