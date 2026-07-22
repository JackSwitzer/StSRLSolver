use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Sources: cards/red/Barricade.java costs 3 (2 upgraded) and applies
    // BarricadePower only when absent; powers/BarricadePower.java is a
    // non-stacking amount -1 power that prevents normal block removal.
    insert(
        cards,
        CardDef {
            id: "Barricade",
            name: "Barricade",
            card_type: CardType::Power,
            target: CardTarget::SelfTarget,
            cost: 3,
            base_damage: -1,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::SetStatus(
                T::Player,
                sid::BARRICADE,
                A::Fixed(1),
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Barricade+",
            name: "Barricade+",
            card_type: CardType::Power,
            target: CardTarget::SelfTarget,
            cost: 2,
            base_damage: -1,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::SetStatus(
                T::Player,
                sid::BARRICADE,
                A::Fixed(1),
            ))],
            complex_hook: None,
        },
    );
}
