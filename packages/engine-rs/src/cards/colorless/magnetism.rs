use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Magnetism: 2 cost, power, add random colorless card to hand each turn (upgrade: cost 1)
    insert(cards, CardDef {
        id: "Magnetism", name: "Magnetism", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
        base_magic: 1, exhaust: false, enter_stance: None,
        effects: &[],
        effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::MAGNETISM, A::Magic))],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Magnetism+", name: "Magnetism+", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
        base_magic: 1, exhaust: false, enter_stance: None,
        effects: &[],
        effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::MAGNETISM, A::Magic))],
        complex_hook: None,
    });
}
