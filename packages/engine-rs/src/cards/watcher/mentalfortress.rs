use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
        id: "MentalFortress", name: "Mental Fortress", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
        base_magic: 4, exhaust: false, enter_stance: None,
        effects: &[], effect_data: &[
            E::Simple(SE::AddStatus(T::Player, sid::MENTAL_FORTRESS, A::Magic)),
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "MentalFortress+", name: "Mental Fortress+", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
        base_magic: 6, exhaust: false, enter_stance: None,
        effects: &[], effect_data: &[
            E::Simple(SE::AddStatus(T::Player, sid::MENTAL_FORTRESS, A::Magic)),
        ], complex_hook: None,
    });
}
