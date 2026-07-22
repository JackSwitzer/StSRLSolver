use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Source: reference/extracted/methods/card/DeadlyPoison.java applies 5
    // Poison for 1 energy; upgrading adds 2 Poison and changes nothing else.
    insert(
        cards,
        CardDef {
            id: "Deadly Poison",
            name: "Deadly Poison",
            card_type: CardType::Skill,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: 5,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::SelectedEnemy,
                sid::POISON,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Deadly Poison+",
            name: "Deadly Poison+",
            card_type: CardType::Skill,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: 7,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::SelectedEnemy,
                sid::POISON,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
}
