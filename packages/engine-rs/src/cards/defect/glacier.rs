use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Glacier gains block before queueing exactly magicNumber (2) Frost
        // channels; upgrading adds 3 block and changes nothing else.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Glacier.java
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
