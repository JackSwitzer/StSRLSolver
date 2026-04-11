use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Wraith Form", name: "Wraith Form", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &[], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::INTANGIBLE, A::Magic)),
                    E::Simple(SE::AddStatus(T::Player, sid::WRAITH_FORM, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Wraith Form+", name: "Wraith Form+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &[], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::INTANGIBLE, A::Magic)),
                    E::Simple(SE::AddStatus(T::Player, sid::WRAITH_FORM, A::Fixed(1))),
                ], complex_hook: None,
            });
}
