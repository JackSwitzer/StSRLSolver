use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Defragment: 1 cost, power, gain 1 focus
    insert(cards, CardDef {
                id: "Defragment", name: "Defragment", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::FOCUS, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Defragment+", name: "Defragment+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::FOCUS, A::Magic)),
                ], complex_hook: None,
            });
}
