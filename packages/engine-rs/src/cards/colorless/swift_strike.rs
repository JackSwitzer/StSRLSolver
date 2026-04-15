use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Swift Strike: 0 cost, 7 dmg
    insert(cards, CardDef {
        id: "Swift Strike", name: "Swift Strike", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 0, base_damage: 7, base_block: -1,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Swift Strike+", name: "Swift Strike+", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 0, base_damage: 10, base_block: -1,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))], complex_hook: None,
    });
}
