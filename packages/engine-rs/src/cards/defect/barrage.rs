use crate::cards::prelude::*;
use crate::effects::declarative::{
    AmountSource as A, Effect as E, SimpleEffect as SE, Target as T,
};

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Sources: cards/blue/Barrage.java supplies 4 damage at cost 1;
    // actions/defect/BarrageAction.java queues one hit per non-empty orb.
    // The card upgrade adds 2 damage without changing the hit count.
    insert(
        cards,
        CardDef {
            id: "Barrage",
            name: "Barrage",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 4,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                E::ExtraHits(A::OrbCount),
            ],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Barrage+",
            name: "Barrage+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 6,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                E::ExtraHits(A::OrbCount),
            ],
            complex_hook: None,
        },
    );
}
