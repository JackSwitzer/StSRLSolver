use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Meteor Strike: 5 cost, 24 dmg, channel 3 Plasma
    insert(cards, CardDef {
                id: "Meteor Strike", name: "Meteor Strike", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 5, base_damage: 24, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["channel_plasma"], effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Plasma, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Meteor Strike+", name: "Meteor Strike+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 5, base_damage: 30, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["channel_plasma"], effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Plasma, A::Magic)),
                ], complex_hook: None,
            });
}
