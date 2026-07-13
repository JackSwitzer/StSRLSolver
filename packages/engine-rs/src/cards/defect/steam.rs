use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE};

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // use grants the card's current block, then ModifyBlockAction decrements
    // that exact combat instance's baseBlock without clamping at zero.
    // upgradeBlock(2) also applies to the current mutable value.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/blue/SteamBarrier.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/ModifyBlockAction.java
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
