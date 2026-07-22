use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // InfiniteBlades.java applies one stack of InfiniteBladesPower; its upgrade
    // keeps cost 1 and only sets isInnate. InfiniteBladesPower.java creates one
    // Shiv per stacked amount in atStartOfTurn.
    insert(
        cards,
        CardDef {
            id: "Infinite Blades",
            name: "Infinite Blades",
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
                sid::INFINITE_BLADES,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Infinite Blades+",
            name: "Infinite Blades+",
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
                sid::INFINITE_BLADES,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
}
