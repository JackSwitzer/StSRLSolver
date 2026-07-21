use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Slice.java deals 6 damage for zero energy; upgradeDamage(3) is its only
    // upgrade change.
    // Java: reference/extracted/methods/card/Slice.java
    insert(cards, CardDef {
        id: "Slice", name: "Slice", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Slice+", name: "Slice+", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 0, base_damage: 9, base_block: -1,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))], complex_hook: None,
    });
}
