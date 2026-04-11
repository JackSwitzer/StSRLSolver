use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Phantasmal Killer", name: "Phantasmal Killer", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["phantasmal_killer", "ethereal"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::DOUBLE_DAMAGE, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Phantasmal Killer+", name: "Phantasmal Killer+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["phantasmal_killer"], effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::DOUBLE_DAMAGE, A::Fixed(1))),
                ], complex_hook: None,
            });
}
