use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Electrodynamics.java applies its non-stacking ElectroPower before
    // channeling 2 Lightning (3 upgraded), so channel auto-evokes already hit
    // every enemy.
    // Java: reference/extracted/methods/card/Electrodynamics.java
    insert(cards, CardDef {
                id: "Electrodynamics", name: "Electrodynamics", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::SetStatus(T::Player, sid::ELECTRODYNAMICS, A::Fixed(1))),
                    E::Simple(SE::ChannelOrb(OrbType::Lightning, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Electrodynamics+", name: "Electrodynamics+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::SetStatus(T::Player, sid::ELECTRODYNAMICS, A::Fixed(1))),
                    E::Simple(SE::ChannelOrb(OrbType::Lightning, A::Magic)),
                ], complex_hook: None,
            });
}
