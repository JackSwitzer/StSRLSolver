use crate::cards::prelude::*;

// Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Halt.java
// Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/HaltAction.java
// Halt.applyPowers temporarily adds 6 (10 upgraded) to baseBlock, calculates
// the modified Wrath component into magicNumber, restores baseBlock, and then
// calculates the normal component. The two declarative block gains reproduce
// those independently modified values.
pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(
        cards,
        CardDef {
            id: "Halt",
            name: "Halt",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: 3,
            base_magic: 9,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Conditional(
                Cond::InStance(Stance::Wrath),
                &[E::Simple(SE::GainBlock(A::Magic))],
                &[],
            )],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Halt+",
            name: "Halt+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: 4,
            base_magic: 14,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Conditional(
                Cond::InStance(Stance::Wrath),
                &[E::Simple(SE::GainBlock(A::Magic))],
                &[],
            )],
            complex_hook: None,
        },
    );
}
