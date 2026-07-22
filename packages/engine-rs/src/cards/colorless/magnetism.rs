use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Magnetism.java applies one MagnetismPower stack. Its upgrade changes only
    // base cost 2 -> 1; MagnetismPower generates one random Colorless per stack.
    insert(
        cards,
        CardDef {
            id: "Magnetism",
            name: "Magnetism",
            card_type: CardType::Power,
            target: CardTarget::SelfTarget,
            cost: 2,
            base_damage: -1,
            base_block: -1,
            base_magic: 1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::Player,
                sid::MAGNETISM,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Magnetism+",
            name: "Magnetism+",
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
                sid::MAGNETISM,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
}
