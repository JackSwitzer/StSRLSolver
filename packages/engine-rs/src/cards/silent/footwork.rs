use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Footwork.java applies DexterityPower(2); upgrading adds exactly 1
    // to that magic amount and changes no other stat.
    // Java: reference/extracted/methods/card/Footwork.java
    insert(
        cards,
        CardDef {
            id: "Footwork",
            name: "Footwork",
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
                sid::DEXTERITY,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Footwork+",
            name: "Footwork+",
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
                sid::DEXTERITY,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
}
