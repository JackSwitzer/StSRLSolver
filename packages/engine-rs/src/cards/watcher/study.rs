use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Uncommon: Study ---- (cost 2, power, add Insight to draw at end of turn; upgrade: cost 1)
    insert(cards, CardDef {
        id: "Study", name: "Study", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
        base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::AddStatus(T::Player, sid::STUDY, A::Magic)),
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Study+", name: "Study+", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
        base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::AddStatus(T::Player, sid::STUDY, A::Magic)),
        ], complex_hook: None,
    });
}
