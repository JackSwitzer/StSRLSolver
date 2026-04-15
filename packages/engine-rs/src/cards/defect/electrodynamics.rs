use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Electrodynamics: 2 cost, power, Lightning hits all enemies, channel 2 Lightning
    insert(cards, CardDef {
                id: "Electrodynamics", name: "Electrodynamics", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Lightning, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Electrodynamics+", name: "Electrodynamics+", card_type: CardType::Power,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Lightning, A::Magic)),
                ], complex_hook: None,
            });
}
