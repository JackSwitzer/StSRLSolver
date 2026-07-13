use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Clash.canUse rejects any non-Attack remaining in hand; upgrade adds 4 damage.
        // Source: reference/extracted/methods/card/Clash.java
    insert(cards, CardDef {
                id: "Clash", name: "Clash", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 14, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Clash+", name: "Clash+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 18, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))], complex_hook: None,
            });
}
