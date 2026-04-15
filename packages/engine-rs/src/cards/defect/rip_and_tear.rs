use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Rip and Tear: 1 cost, deal 7 dmg twice to random enemies
    insert(cards, CardDef {
                id: "Rip and Tear", name: "Rip and Tear", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 1, base_damage: 7, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
                    E::ExtraHits(A::Magic),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Rip and Tear+", name: "Rip and Tear+", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 1, base_damage: 9, base_block: -1,
                base_magic: 2, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
                    E::ExtraHits(A::Magic),
                ], complex_hook: None,
            });
}
