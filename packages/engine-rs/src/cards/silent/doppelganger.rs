use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Doppelganger", name: "Doppelganger", card_type: CardType::Skill,
                target: CardTarget::None, cost: -1, base_damage: -1, base_block: -1,
                base_magic: 0, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::DOPPELGANGER_ENERGY, A::XCost)),
                    E::Simple(SE::AddStatus(T::Player, sid::DOPPELGANGER_DRAW, A::XCost)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Doppelganger+", name: "Doppelganger+", card_type: CardType::Skill,
                target: CardTarget::None, cost: -1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::DOPPELGANGER_ENERGY, A::MagicPlusX)),
                    E::Simple(SE::AddStatus(T::Player, sid::DOPPELGANGER_DRAW, A::MagicPlusX)),
                ], complex_hook: None,
            });
}
