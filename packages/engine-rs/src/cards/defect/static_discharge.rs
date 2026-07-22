use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // StaticDischarge.use applies magicNumber stacks; the power channels that
    // many Lightning after positive non-THORNS/non-HP_LOSS damage from another
    // owner survives block and Buffer.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/StaticDischarge.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/StaticDischargePower.java
    insert(
        cards,
        CardDef {
            id: "Static Discharge",
            name: "Static Discharge",
            card_type: CardType::Power,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: 1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::Player,
                sid::STATIC_DISCHARGE,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Static Discharge+",
            name: "Static Discharge+",
            card_type: CardType::Power,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::Player,
                sid::STATIC_DISCHARGE,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
}
