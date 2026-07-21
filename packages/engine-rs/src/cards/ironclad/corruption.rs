use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Source: reference/extracted/methods/card/Corruption.java applies the
    // non-stacking power at cost 3 (2 upgraded); CorruptionPower.java makes
    // Skills free when drawn and marks every used Skill for Exhaust.
    insert(cards, CardDef {
                id: "Corruption", name: "Corruption", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 3, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::SetStatus(T::Player, sid::CORRUPTION, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Corruption+", name: "Corruption+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::SetStatus(T::Player, sid::CORRUPTION, A::Fixed(1))),
                ], complex_hook: None,
            });
}
