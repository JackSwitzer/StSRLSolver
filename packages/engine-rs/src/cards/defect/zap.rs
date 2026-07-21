use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Zap.use queues one Lightning ChannelAction from magicNumber 1. The
    // upgrade changes only base cost from one to zero.
    // Java: reference/extracted/methods/card/Zap.java
    insert(cards, CardDef {
                id: "Zap", name: "Zap", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Lightning, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Zap+", name: "Zap+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Lightning, A::Magic)),
                ], complex_hook: None,
            });
}
