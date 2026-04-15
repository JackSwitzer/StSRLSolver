use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Dramatic Entrance: 0 cost, 8 dmg AoE, innate, exhaust
    insert(cards, CardDef {
        id: "Dramatic Entrance", name: "Dramatic Entrance", card_type: CardType::Attack,
        target: CardTarget::AllEnemy, cost: 0, base_damage: 8, base_block: -1,
        base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Dramatic Entrance+", name: "Dramatic Entrance+", card_type: CardType::Attack,
        target: CardTarget::AllEnemy, cost: 0, base_damage: 12, base_block: -1,
        base_magic: -1, exhaust: true, enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))],
        complex_hook: None,
    });
}
