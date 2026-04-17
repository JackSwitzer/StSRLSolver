use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Tempest: X cost, channel X Lightning orbs, exhaust (upgrade: +1)
    insert(cards, CardDef {
                id: "Tempest", name: "Tempest", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Lightning, A::XCost)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Tempest+", name: "Tempest+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: -1, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Lightning, A::XCost)),
                    E::Simple(SE::ChannelOrb(OrbType::Lightning, A::Fixed(1))),
                ], complex_hook: None,
            });
}
