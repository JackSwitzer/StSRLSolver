use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Rare: Tools of the Trade ---- (cost 1, power, draw 1 + discard 1 at turn start; upgrade: cost 0)
    insert(cards, CardDef {
                id: "Tools of the Trade", name: "Tools of the Trade", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::TOOLS_OF_THE_TRADE, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Tools of the Trade+", name: "Tools of the Trade+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::TOOLS_OF_THE_TRADE, A::Magic)),
                ], complex_hook: None,
            });
}
