use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Simmering Fury ---- (Java ID: Vengeance, cost 1, next turn enter Wrath + draw 2; +1 magic upgrade)
        // Sets SIMMERING_FURY status to magic (draw count). Turn-start hook enters Wrath + draws.
    insert(cards, CardDef {
                id: "Vengeance", name: "Simmering Fury", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::SetStatus(T::Player, sid::SIMMERING_FURY, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Vengeance+", name: "Simmering Fury+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::SetStatus(T::Player, sid::SIMMERING_FURY, A::Magic)),
                ], complex_hook: None,
            });
}
