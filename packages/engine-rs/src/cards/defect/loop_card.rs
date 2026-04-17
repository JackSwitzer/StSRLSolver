use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Loop: 1 cost, power, trigger frontmost orb passive at start of turn
    insert(cards, CardDef {
        id: "Loop", name: "Loop", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
        base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::AddStatus(T::Player, sid::LOOP, A::Magic)),
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Loop+", name: "Loop+", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
        base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::AddStatus(T::Player, sid::LOOP, A::Magic)),
        ], complex_hook: None,
    });
}
