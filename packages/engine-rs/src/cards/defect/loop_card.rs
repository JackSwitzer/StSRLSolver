use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Loop.java applies one LoopPower stack for one energy; upgradeMagicNumber(1).
    // LoopPower.atStartOfTurn calls the front orb's start and end callbacks once
    // per stack.
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
