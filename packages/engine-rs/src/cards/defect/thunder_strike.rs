use crate::cards::prelude::*;
use crate::effects::declarative::{
    AmountSource as A, Effect as E, SimpleEffect as SE, Target as T,
};
use crate::status_ids::sid;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ThunderStrike.java queues one random-enemy action per Lightning orb
    // channeled this combat, carries the STRIKE tag, and upgrades 7 -> 9.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/ThunderStrike.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/defect/NewThunderStrikeAction.java
    insert(
        cards,
        CardDef {
            id: "Thunder Strike",
            name: "Thunder Strike",
            card_type: CardType::Attack,
            target: CardTarget::AllEnemy,
            cost: 3,
            base_damage: 7,
            base_block: -1,
            base_magic: 0,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
                E::ExtraHits(A::StatusValue(sid::LIGHTNING_CHANNELED)),
            ],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Thunder Strike+",
            name: "Thunder Strike+",
            card_type: CardType::Attack,
            target: CardTarget::AllEnemy,
            cost: 3,
            base_damage: 9,
            base_block: -1,
            base_magic: 0,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::DealDamage(T::RandomEnemy, A::Damage)),
                E::ExtraHits(A::StatusValue(sid::LIGHTNING_CHANNELED)),
            ],
            complex_hook: None,
        },
    );
}
