use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Glacier: 2 cost, 7 block, channel 2 Frost
    insert(cards, CardDef {
                id: "Glacier", name: "Glacier", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 7,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Frost, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Glacier+", name: "Glacier+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 10,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Frost, A::Magic)),
                ], complex_hook: None,
            });
}
