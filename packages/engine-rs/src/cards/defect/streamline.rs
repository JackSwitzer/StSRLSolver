use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Streamline: 2 cost, 15 dmg, costs 1 less each play.
        //
        // The Java game uses card-instance UUID targeting here. Our current
        // runtime does not carry UUIDs on CardInstance, so we can only narrow
        // the mutation to a single surviving copy instead of broadcasting to
        // every Streamline copy. The defect test file records that blocker.
    insert(cards, CardDef {
                id: "Streamline", name: "Streamline", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 15, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Simple(SE::ModifyPlayedCardCost(A::Fixed(-1))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Streamline+", name: "Streamline+", card_type: CardType::Attack,
                target: CardTarget::Enemy, cost: 2, base_damage: 20, base_block: -1,
                base_magic: 1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                    E::Simple(SE::ModifyPlayedCardCost(A::Fixed(-1))),
                ], complex_hook: None,
            });
}

#[cfg(test)]
#[path = "../../tests/test_defect_java_wave1.rs"]
mod test_defect_java_wave1;

#[cfg(test)]
#[path = "../../tests/test_played_card_instance_state.rs"]
mod test_played_card_instance_state;

#[cfg(test)]
#[path = "../../tests/test_orb_runtime_java_wave1.rs"]
mod test_orb_runtime_java_wave1;
