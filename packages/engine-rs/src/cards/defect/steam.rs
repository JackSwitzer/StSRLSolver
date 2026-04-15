use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE};

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Steam Barrier (SteamBarrier): 0 cost, 6 block, loses 1 block each play
    insert(cards, CardDef {
                id: "Steam", name: "Steam Barrier", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 6,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::GainBlock(A::Block)),
                    E::Simple(SE::ModifyPlayedCardBlock(A::Fixed(-1))),
                ],
                complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Steam+", name: "Steam Barrier+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 0, base_damage: -1, base_block: 8,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::GainBlock(A::Block)),
                    E::Simple(SE::ModifyPlayedCardBlock(A::Fixed(-1))),
                ],
                complex_hook: None,
            });
}
