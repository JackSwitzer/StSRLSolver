use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Expunger (from Conjure Blade): cost 1, deal 9 dmg X times
    insert(cards, CardDef {
                id: "Expunger", name: "Expunger", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 9, base_block: -1,
                base_magic: 0, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::ExtraHits(A::CardMisc),
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Expunger+", name: "Expunger+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 1, base_damage: 15, base_block: -1,
                base_magic: 0, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::ExtraHits(A::CardMisc),
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                ], complex_hook: None,
            });
}
