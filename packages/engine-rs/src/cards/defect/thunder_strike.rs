use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE, Target as T};
use crate::status_ids::sid;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Thunder Strike: 3 cost, deal 7 dmg for each Lightning channeled this combat
    insert(cards, CardDef {
                id: "Thunder Strike", name: "Thunder Strike", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 3, base_damage: 7, base_block: -1,
                base_magic: 0, exhaust: false, enter_stance: None,
                effects: &[], effect_data: &[
                    E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
                    E::ExtraHits(A::StatusValue(sid::LIGHTNING_CHANNELED)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Thunder Strike+", name: "Thunder Strike+", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 3, base_damage: 9, base_block: -1,
                base_magic: 0, exhaust: false, enter_stance: None,
                effects: &[], effect_data: &[
                    E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
                    E::ExtraHits(A::StatusValue(sid::LIGHTNING_CHANNELED)),
                ], complex_hook: None,
            });
}
