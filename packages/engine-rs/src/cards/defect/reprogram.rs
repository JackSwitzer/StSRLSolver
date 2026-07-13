use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Reprogram.java queues negative Focus, then positive Strength and
    // Dexterity, all for magicNumber 1 (2 upgraded).
    // Java: reference/extracted/methods/card/Reprogram.java
    insert(cards, CardDef {
                id: "Reprogram", name: "Reprogram", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::FOCUS, A::Fixed(-1))),
                    E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic)),
                    E::Simple(SE::AddStatus(T::Player, sid::DEXTERITY, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Reprogram+", name: "Reprogram+", card_type: CardType::Skill,
                target: CardTarget::None, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::Player, sid::FOCUS, A::Fixed(-2))),
                    E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic)),
                    E::Simple(SE::AddStatus(T::Player, sid::DEXTERITY, A::Magic)),
                ], complex_hook: None,
            });
}
