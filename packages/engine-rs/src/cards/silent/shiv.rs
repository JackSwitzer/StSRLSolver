use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Shiv.java constructs a zero-cost, 4-damage exhausting Attack;
    // upgradeDamage(2) is its only upgrade change.
    // Java: reference/extracted/methods/card/Shiv.java
    insert(cards, CardDef {
        id: "Shiv", name: "Shiv", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
        base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
        ], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Shiv+", name: "Shiv+", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
        base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[
            E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
        ], complex_hook: None,
    });
}
