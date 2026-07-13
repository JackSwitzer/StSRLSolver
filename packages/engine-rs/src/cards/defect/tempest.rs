use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // TempestAction channels X Lightning orbs, with +2 from Chemical X and
        // +1 when upgraded, then spends energy unless the play is free.
        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Tempest.java
        // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/unique/TempestAction.java
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
