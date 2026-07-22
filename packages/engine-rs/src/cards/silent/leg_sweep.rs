use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // LegSweep.java queues Weak before GainBlockAction: 2 Weak and 11 Block
    // for two energy; upgradeMagicNumber(1) and upgradeBlock(3).
    insert(
        cards,
        CardDef {
            id: "Leg Sweep",
            name: "Leg Sweep",
            card_type: CardType::Skill,
            target: CardTarget::Enemy,
            cost: 2,
            base_damage: -1,
            base_block: 11,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
                E::Simple(SE::GainBlock(A::Block)),
            ],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Leg Sweep+",
            name: "Leg Sweep+",
            card_type: CardType::Skill,
            target: CardTarget::Enemy,
            cost: 2,
            base_damage: -1,
            base_block: 14,
            base_magic: 3,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::AddStatus(T::SelectedEnemy, sid::WEAKENED, A::Magic)),
                E::Simple(SE::GainBlock(A::Block)),
            ],
            complex_hook: None,
        },
    );
}
