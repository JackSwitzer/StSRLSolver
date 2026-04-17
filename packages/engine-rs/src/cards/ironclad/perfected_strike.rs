use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Ironclad Common: Perfected Strike ---- (cost 2, 6 dmg + 2/strike in deck; +1 magic)
    insert(cards, CardDef {
        id: "Perfected Strike", name: "Perfected Strike", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 2, base_damage: 6, base_block: -1,
        base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Perfected Strike+", name: "Perfected Strike+", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 2, base_damage: 6, base_block: -1,
        base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
        complex_hook: None,
    });
}
