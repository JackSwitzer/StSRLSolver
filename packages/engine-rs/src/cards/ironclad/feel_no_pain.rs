use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // FeelNoPain.java installs magicNumber stacks; FeelNoPainPower.onExhaust
    // queues a GainBlockAction for that raw amount on every exhausted card.
    // Java: reference/extracted/methods/card/FeelNoPain.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/FeelNoPainPower.java
    insert(
        cards,
        CardDef {
            id: "Feel No Pain",
            name: "Feel No Pain",
            card_type: CardType::Power,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: 3,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::Player,
                sid::FEEL_NO_PAIN,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Feel No Pain+",
            name: "Feel No Pain+",
            card_type: CardType::Power,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: 4,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::Player,
                sid::FEEL_NO_PAIN,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
}
