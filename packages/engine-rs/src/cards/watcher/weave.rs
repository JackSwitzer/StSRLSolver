use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Weave ---- (cost 0, 4 dmg, returns to hand on Scry; +2 dmg upgrade)
    insert(cards, CardDef {
                id: "Weave", name: "Weave", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 4, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["return_on_scry"], effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Weave+", name: "Weave+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 6, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["return_on_scry"], effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                ], complex_hook: None,
            });
}
