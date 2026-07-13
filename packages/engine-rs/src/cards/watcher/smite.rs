use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Smite.java is a one-cost, 12-damage self-retaining Attack that exhausts;
    // upgradeDamage(4) is its only upgrade change.
    // Java: reference/extracted/methods/card/Smite.java
    insert(cards, CardDef {
        id: "Smite", name: "Smite", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 1, base_damage: 12, base_block: -1,
        base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Smite+", name: "Smite+", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 1, base_damage: 16, base_block: -1,
        base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
        ], complex_hook: None,
    });
}
