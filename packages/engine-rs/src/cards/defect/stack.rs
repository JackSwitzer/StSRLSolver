use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Stack.applyPowers sets baseBlock to discard size, adds the upgraded +3,
    // then applies Dexterity/Frail once to that combined amount.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Stack.java
    insert(
        cards,
        CardDef {
            id: "Stack",
            name: "Stack",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 0,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::GainBlock(A::DiscardPileSizePlusBlock))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Stack+",
            name: "Stack+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 3,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::GainBlock(A::DiscardPileSizePlusBlock))],
            complex_hook: None,
        },
    );
}
