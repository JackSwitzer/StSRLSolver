use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Glass Knife", name: "Glass Knife", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 8, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["multi_hit", "glass_knife"],
                effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
                complex_hook: Some(crate::effects::hooks_complex::hook_glass_knife),
            });
    insert(cards, CardDef {
                id: "Glass Knife+", name: "Glass Knife+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 10, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effects: &["multi_hit", "glass_knife"],
                effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
                complex_hook: Some(crate::effects::hooks_complex::hook_glass_knife),
            });
}
