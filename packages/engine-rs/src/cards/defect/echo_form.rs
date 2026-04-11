use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Echo Form: 3 cost, power, ethereal, first card each turn played twice (upgrade: no ethereal)
    insert(cards, CardDef {
                id: "Echo Form", name: "Echo Form", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["echo_form", "ethereal"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::ECHO_FORM, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Echo Form+", name: "Echo Form+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["echo_form"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::ECHO_FORM, A::Fixed(1))),
                ], complex_hook: None,
            });
}
