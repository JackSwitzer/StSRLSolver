use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // MachineLearning.java applies one DrawPower stack; its upgrade changes only
    // isInnate and leaves cost/magic unchanged.
    insert(cards, CardDef {
        id: "Machine Learning", name: "Machine Learning", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
        base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::DRAW, A::Magic))],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Machine Learning+", name: "Machine Learning+", card_type: CardType::Power,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
        base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::DRAW, A::Magic))],
        complex_hook: None,
    });
}
