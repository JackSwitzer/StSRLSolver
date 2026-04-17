use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, Effect as E, SimpleEffect as SE};

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Rare: Spirit Shield ---- (cost 2, skill, gain 3 block per card in hand; +1 magic upgrade)
    insert(cards, CardDef {
                id: "SpiritShield", name: "Spirit Shield", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::GainBlock(A::HandSize)),
                    E::Simple(SE::GainBlock(A::HandSize)),
                    E::Simple(SE::GainBlock(A::HandSize)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "SpiritShield+", name: "Spirit Shield+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::GainBlock(A::HandSize)),
                    E::Simple(SE::GainBlock(A::HandSize)),
                    E::Simple(SE::GainBlock(A::HandSize)),
                    E::Simple(SE::GainBlock(A::HandSize)),
                ], complex_hook: None,
            });
}
