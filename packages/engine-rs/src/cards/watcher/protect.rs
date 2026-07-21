use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Protect owns ordinary modified block and selfRetain; the upgrade adds 4.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Protect.java
    insert(cards, CardDef {
        id: "Protect", name: "Protect", card_type: CardType::Skill,
        target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 12,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::GainBlock(A::Block)),
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Protect+", name: "Protect+", card_type: CardType::Skill,
        target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: 16,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
            E::Simple(SE::GainBlock(A::Block)),
        ], complex_hook: None,
    });
}
