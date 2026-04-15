use crate::cards::prelude::*;

#[cfg(test)]
#[path = "../../tests/test_card_runtime_defect_wave6.rs"]
mod test_card_runtime_defect_wave6;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Defect Basic Cards ----
    insert(cards, CardDef {
        id: "Strike_B", name: "Strike", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 1, base_damage: 6, base_block: -1,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
        complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Strike_B+", name: "Strike+", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
        base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
        complex_hook: None,
    });
}
