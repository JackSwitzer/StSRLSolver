use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // PoisonedStab.java queues DamageAction before ApplyPowerAction. Upgrade
    // adds 2 damage and 1 Poison, from 6/3 to 8/4.
    // Source: reference/extracted/methods/card/PoisonedStab.java
    insert(
        cards,
        CardDef {
            id: "Poisoned Stab",
            name: "Poisoned Stab",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 6,
            base_block: -1,
            base_magic: 3,
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
            id: "Poisoned Stab+",
            name: "Poisoned Stab+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 8,
            base_block: -1,
            base_magic: 4,
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
