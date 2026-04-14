use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Silent Common: Slice ---- (cost 0, 6 dmg; +3 dmg)
    insert(cards, CardDef {
        id: "Slice", name: "Slice", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
        base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Slice+", name: "Slice+", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 0, base_damage: 9, base_block: -1,
        base_magic: -1, exhaust: false, enter_stance: None, effects: &[],
        effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))], complex_hook: None,
    });
}
