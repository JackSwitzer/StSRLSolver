use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Mayhem.java applies one MayhemPower stack for cost 2; upgrade changes
    // only cost 2 -> 1. MayhemPower autoplays one top draw card per stack.
    insert(cards, CardDef {
        id: "Mayhem", name: "Mayhem", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
        base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::MAYHEM, A::Magic))],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Mayhem+", name: "Mayhem+", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
        base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::MAYHEM, A::Magic))],
        complex_hook: None,
    });
}
