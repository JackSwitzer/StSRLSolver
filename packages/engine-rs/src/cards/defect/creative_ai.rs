use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Creative AI: 3 cost, power, add random Power to hand each turn (upgrade: cost 2)
    insert(cards, CardDef {
                id: "Creative AI", name: "Creative AI", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::CREATIVE_AI, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Creative AI+", name: "Creative AI+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::CREATIVE_AI, A::Magic)),
                ], complex_hook: None,
            });
}
