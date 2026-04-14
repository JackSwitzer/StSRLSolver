use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Common: Bane ---- (cost 1, 7 dmg, double if poisoned; +3 dmg)
    insert(cards, CardDef {
                id: "Bane", name: "Bane", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 7, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["double_if_poisoned"],
                effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
                complex_hook: Some(crate::effects::hooks_complex::hook_double_if_poisoned),
            });
    insert(cards, CardDef {
                id: "Bane+", name: "Bane+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["double_if_poisoned"],
                effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
                complex_hook: Some(crate::effects::hooks_complex::hook_double_if_poisoned),
            });
}
