use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/Brutality.java
    // Cost 0 and applies one Brutality stack; the upgrade changes only isInnate.
    insert(
        cards,
        CardDef {
            id: "Brutality",
            name: "Brutality",
            card_type: CardType::Power,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::Player,
                sid::BRUTALITY,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Brutality+",
            name: "Brutality+",
            card_type: CardType::Power,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::Player,
                sid::BRUTALITY,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
}
