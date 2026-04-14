use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Heatsinks: 1 cost, power, whenever you play a power draw 1 card
    insert(cards, CardDef {
        id: "Heatsinks", name: "Heatsinks", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
        base_magic: 1, exhaust: false, enter_stance: None,
        effects: &[], effect_data: &[
            E::Simple(SE::AddStatus(T::Player, sid::HEATSINK, A::Magic)),
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Heatsinks+", name: "Heatsinks+", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
        base_magic: 2, exhaust: false, enter_stance: None,
        effects: &[], effect_data: &[
            E::Simple(SE::AddStatus(T::Player, sid::HEATSINK, A::Magic)),
        ], complex_hook: None,
    });
}
