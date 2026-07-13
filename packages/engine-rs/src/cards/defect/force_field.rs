use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ForceField.java costs 4, grants 12 Block, and permanently reduces each
    // copy by one per Power card played this combat. New copies replay that
    // combat history; upgrading adds 4 Block only.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/ForceField.java
    insert(cards, CardDef {
        id: "Force Field", name: "Force Field", card_type: CardType::Skill,
        target: CardTarget::SelfTarget, cost: 4, base_damage: -1, base_block: 12,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::GainBlock(A::Block))],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Force Field+", name: "Force Field+", card_type: CardType::Skill,
        target: CardTarget::SelfTarget, cost: 4, base_damage: -1, base_block: 16,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::GainBlock(A::Block))],
        complex_hook: None,
    });
}
