use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Sunder: 3 cost, 24 dmg, gain 3 energy if this kills
    insert(cards, CardDef {
                id: "Sunder", name: "Sunder", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 3, base_damage: 24, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Conditional(Cond::EnemyKilled, &[E::Simple(SE::GainEnergy(A::Fixed(3)))], &[]),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Sunder+", name: "Sunder+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 3, base_damage: 32, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Conditional(Cond::EnemyKilled, &[E::Simple(SE::GainEnergy(A::Fixed(3)))], &[]),
                ], complex_hook: None,
            });
}
