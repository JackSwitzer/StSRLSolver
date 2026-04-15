use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    insert(cards, CardDef {
                id: "Conclude", name: "Conclude", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 1, base_damage: 12, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::AllEnemies, A::Damage)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Conclude+", name: "Conclude+", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 1, base_damage: 16, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::AllEnemies, A::Damage)),
                ], complex_hook: None,
    });
}

#[cfg(test)]
#[path = "../../tests/test_card_runtime_watcher_wave5.rs"]
mod test_card_runtime_watcher_wave5;
