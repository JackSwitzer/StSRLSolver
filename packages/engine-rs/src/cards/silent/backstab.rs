use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ---- Silent Uncommon: Backstab ---- (cost 0, 11 dmg, innate, exhaust; +4 dmg)
    insert(cards, CardDef {
        id: "Backstab", name: "Backstab", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 0, base_damage: 11, base_block: -1,
        base_magic: -1, exhaust: true, enter_stance: None,
        effects: &["innate"], effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))], complex_hook: None,
    });
    insert(cards, CardDef {
        id: "Backstab+", name: "Backstab+", card_type: CardType::Attack,
        target: CardTarget::Enemy, cost: 0, base_damage: 15, base_block: -1,
        base_magic: -1, exhaust: true, enter_stance: None,
        effects: &["innate"], effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))], complex_hook: None,
    });
}
