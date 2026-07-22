use crate::cards::prelude::*;
use crate::effects::declarative::{
    AmountSource as A, Effect as E, SimpleEffect as SE, Target as T,
};

static RAGNAROK: [E; 5] = [
    E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
    E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
    E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
    E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
    E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
];

static RAGNAROK_PLUS: [E; 6] = [
    E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
    E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
    E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
    E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
    E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
    E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Each AttackDamageRandomEnemyAction recalculates damage for its chosen
    // target and consumes one cardRandomRng selection. Upgrade adds one
    // damage and one queued hit.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Ragnarok.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/AttackDamageRandomEnemyAction.java
    insert(
        cards,
        CardDef {
            id: "Ragnarok",
            name: "Ragnarok",
            card_type: CardType::Attack,
            target: CardTarget::AllEnemy,
            cost: 3,
            base_damage: 5,
            base_block: -1,
            base_magic: 5,
            exhaust: false,
            enter_stance: None,
            effect_data: &RAGNAROK,
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Ragnarok+",
            name: "Ragnarok+",
            card_type: CardType::Attack,
            target: CardTarget::AllEnemy,
            cost: 3,
            base_damage: 6,
            base_block: -1,
            base_magic: 6,
            exhaust: false,
            enter_stance: None,
            effect_data: &RAGNAROK_PLUS,
            complex_hook: None,
        },
    );
}
