use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Fusion: 2 cost, channel 1 Plasma (upgrade: cost 1)
    insert(cards, CardDef {
                id: "Fusion", name: "Fusion", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["channel_plasma"], effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Plasma, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Fusion+", name: "Fusion+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["channel_plasma"], effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Plasma, A::Magic)),
                ], complex_hook: None,
            });
}
