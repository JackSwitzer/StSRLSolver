use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Defect Common Cards ----
        // Ball Lightning: 1 cost, 7 dmg, channel 1 Lightning
    insert(cards, CardDef {
                id: "Ball Lightning", name: "Ball Lightning", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["channel_lightning"], effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Lightning, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Ball Lightning+", name: "Ball Lightning+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effects: &["channel_lightning"], effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Lightning, A::Magic)),
                ], complex_hook: None,
            });
}
