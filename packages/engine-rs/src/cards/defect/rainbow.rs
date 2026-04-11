use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Rainbow: 2 cost, channel Lightning+Frost+Dark, exhaust (upgrade: no exhaust)
    insert(cards, CardDef {
                id: "Rainbow", name: "Rainbow", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effects: &["channel_lightning", "channel_frost", "channel_dark"], effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Lightning, A::Fixed(1))),
                    E::Simple(SE::ChannelOrb(OrbType::Frost, A::Fixed(1))),
                    E::Simple(SE::ChannelOrb(OrbType::Dark, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Rainbow+", name: "Rainbow+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["channel_lightning", "channel_frost", "channel_dark"], effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Lightning, A::Fixed(1))),
                    E::Simple(SE::ChannelOrb(OrbType::Frost, A::Fixed(1))),
                    E::Simple(SE::ChannelOrb(OrbType::Dark, A::Fixed(1))),
                ], complex_hook: None,
            });
}
