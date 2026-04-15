use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Silent Basic: Strike_G ----
    insert(cards, CardDef {
        id: "Strike_G", name: "Strike", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Strike_G+", name: "Strike+", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))], complex_hook: None,
    });
}
