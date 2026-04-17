use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Uncommon: Wallop ---- (cost 2, 9 dmg, gain block equal to unblocked damage; +3 dmg upgrade)
    insert(cards, CardDef {
                id: "Wallop", name: "Wallop", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 9, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Simple(SE::GainBlock(A::TotalUnblockedDamage)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Wallop+", name: "Wallop+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 12, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Simple(SE::GainBlock(A::TotalUnblockedDamage)),
                ], complex_hook: None,
            });
}
