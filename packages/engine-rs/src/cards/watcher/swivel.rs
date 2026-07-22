use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Swivel.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/FreeAttackPower.java
    // ApplyPowerAction stacks one FreeAttackPower charge per copy played.
    insert(
        cards,
        CardDef {
            id: "Swivel",
            name: "Swivel",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 2,
            base_damage: -1,
            base_block: 8,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::SetFlag(BF::NextAttackFree))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Swivel+",
            name: "Swivel+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 2,
            base_damage: -1,
            base_block: 11,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::SetFlag(BF::NextAttackFree))],
            complex_hook: None,
        },
    );
}
