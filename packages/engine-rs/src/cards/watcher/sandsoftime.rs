use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Sands of Time ---- (cost 4, 20 dmg, retain, cost -1 each retain; +6 dmg upgrade)
    insert(cards, CardDef {
                id: "SandsOfTime", name: "Sands of Time", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 4, base_damage: 20, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "SandsOfTime+", name: "Sands of Time+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 4, base_damage: 26, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                ], complex_hook: None,
            });
}
