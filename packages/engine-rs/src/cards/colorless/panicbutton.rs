use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // PanicButton.java gains 30 block before applying two NoBlockPower for zero
    // energy and exhausts. Upgrade adds 10 block only.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/PanicButton.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/NoBlockPower.java
    insert(
        cards,
        CardDef {
            id: "PanicButton",
            name: "Panic Button",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: 30,
            base_magic: 2,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::NO_BLOCK, A::Magic))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "PanicButton+",
            name: "Panic Button+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: 40,
            base_magic: 2,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(T::Player, sid::NO_BLOCK, A::Magic))],
            complex_hook: None,
        },
    );
}
