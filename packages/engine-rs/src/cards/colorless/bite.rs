use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Source: cards/colorless/Bite.java costs 1, queues 7 damage then a
    // 2 HP HealAction, and upgrades both values by 1.
    insert(
        cards,
        CardDef {
            id: "Bite",
            name: "Bite",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 7,
            base_block: -1,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::HealHp(T::Player, A::Magic))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Bite+",
            name: "Bite+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 8,
            base_block: -1,
            base_magic: 3,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::HealHp(T::Player, A::Magic))],
            complex_hook: None,
        },
    );
}
