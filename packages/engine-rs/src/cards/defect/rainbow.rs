use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Rainbow channels Lightning, then Frost, then Dark. The upgrade changes
    // only exhaust=true to false; cost and the three-action order stay fixed.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Rainbow.java
    insert(cards, CardDef {
                id: "Rainbow", name: "Rainbow", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Lightning, A::Fixed(1))),
                    E::Simple(SE::ChannelOrb(OrbType::Frost, A::Fixed(1))),
                    E::Simple(SE::ChannelOrb(OrbType::Dark, A::Fixed(1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Rainbow+", name: "Rainbow+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::ChannelOrb(OrbType::Lightning, A::Fixed(1))),
                    E::Simple(SE::ChannelOrb(OrbType::Frost, A::Fixed(1))),
                    E::Simple(SE::ChannelOrb(OrbType::Dark, A::Fixed(1))),
                ], complex_hook: None,
            });
}
