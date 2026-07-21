use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/Sentinel.java
    // Grants 5 Block and unconditionally gains 2 energy when exhausted;
    // upgrading raises those values to 8 Block and 3 energy.
    insert(cards, CardDef {
        id: "Sentinel", name: "Sentinel", card_type: CardType::Skill,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 5,
        base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::GainBlock(A::Block))],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Sentinel+", name: "Sentinel+", card_type: CardType::Skill,
        target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 8,
        base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::GainBlock(A::Block))],
        complex_hook: None,
    });
}
