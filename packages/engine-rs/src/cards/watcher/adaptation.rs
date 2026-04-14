use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Power Cards ----
    insert(cards, CardDef {
        id: "Adaptation", name: "Rushdown", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
        base_magic: 2, exhaust: false, enter_stance: None,
        effects: &[], effect_data: &[
            E::Simple(SE::AddStatus(T::Player, sid::RUSHDOWN, A::Magic)),
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Adaptation+", name: "Rushdown+", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
        base_magic: 2, exhaust: false, enter_stance: None,
        effects: &[], effect_data: &[
            E::Simple(SE::AddStatus(T::Player, sid::RUSHDOWN, A::Magic)),
        ], complex_hook: None,
    });
}
