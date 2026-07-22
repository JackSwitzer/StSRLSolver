use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Good Instincts costs 0 and queues one GainBlockAction for 6 block;
    // upgrading adds exactly 3 block.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/GoodInstincts.java
    insert(
        cards,
        CardDef {
            id: "Good Instincts",
            name: "Good Instincts",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: 6,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::GainBlock(A::Block))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Good Instincts+",
            name: "Good Instincts+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: 9,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::GainBlock(A::Block))],
            complex_hook: None,
        },
    );
}
