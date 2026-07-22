use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // MentalFortressPower stacks its amount and queues raw GainBlockAction
    // only when the old and new stance IDs differ.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/MentalFortress.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/MentalFortressPower.java
    insert(
        cards,
        CardDef {
            id: "MentalFortress",
            name: "Mental Fortress",
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
                sid::MENTAL_FORTRESS,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "MentalFortress+",
            name: "Mental Fortress+",
            card_type: CardType::Power,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: 6,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::Player,
                sid::MENTAL_FORTRESS,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
}
