use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // SearingBlow.java costs 2 and starts at 12 damage. canUpgrade() always
        // returns true; upgrade n adds 4 + the previous timesUpgraded, producing
        // 12, 16, 21, 27, ... damage while preserving the exact counter.
        // Java: reference/extracted/methods/card/SearingBlow.java
    insert(cards, CardDef {
                id: "Searing Blow", name: "Searing Blow", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 12, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Searing Blow+", name: "Searing Blow+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 16, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))], complex_hook: None,
            });
}
