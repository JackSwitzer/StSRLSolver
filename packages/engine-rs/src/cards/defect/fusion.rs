use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Fusion loops magicNumber times to channel Plasma; upgrading changes
        // only its cost from 2 to 1.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Fusion.java
    insert(cards, CardDef {
                id: "Fusion", name: "Fusion", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Plasma, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Fusion+", name: "Fusion+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Plasma, A::Fixed(1))),
                ], complex_hook: None,
            });
}
