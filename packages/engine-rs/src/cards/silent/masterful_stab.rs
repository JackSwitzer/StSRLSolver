use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // MasterfulStab.java starts at cost 0/damage 12. tookDamage() calls
    // updateCost(1) once per damage event; upgradeDamage(4) is the only upgrade.
    insert(cards, CardDef {
                id: "Masterful Stab", name: "Masterful Stab", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 12, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Masterful Stab+", name: "Masterful Stab+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 0, base_damage: 16, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                ], complex_hook: None,
            });
}

#[cfg(test)]
#[path = "../../tests/test_card_runtime_silent_wave3.rs"]
mod test_card_runtime_silent_wave3;
