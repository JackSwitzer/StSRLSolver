use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Doom and Gloom: 2 cost, 10 dmg AoE, channel 1 Dark
    insert(cards, CardDef {
                id: "Doom and Gloom", name: "Doom and Gloom", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 2, base_damage: 10, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["channel_dark"], effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Dark, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Doom and Gloom+", name: "Doom and Gloom+", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 2, base_damage: 14, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["channel_dark"], effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Dark, A::Magic)),
                ], complex_hook: None,
            });
}
