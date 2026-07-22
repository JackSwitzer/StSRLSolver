use crate::cards::prelude::*;
pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/SpiritShield.java
    // applyPowers sets one baseBlock equal to the remaining hand size times
    // magicNumber, then applies block modifiers once to that total.
    insert(
        cards,
        CardDef {
            id: "SpiritShield",
            name: "Spirit Shield",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 2,
            base_damage: -1,
            base_block: -1,
            base_magic: 3,
            exhaust: false,
            enter_stance: None,
            effect_data: &[],
            complex_hook: Some(crate::effects::hooks_simple::hook_block_per_card_in_hand),
        },
    );
    insert(
        cards,
        CardDef {
            id: "SpiritShield+",
            name: "Spirit Shield+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 2,
            base_damage: -1,
            base_block: -1,
            base_magic: 4,
            exhaust: false,
            enter_stance: None,
            effect_data: &[],
            complex_hook: Some(crate::effects::hooks_simple::hook_block_per_card_in_hand),
        },
    );
}
