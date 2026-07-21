use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java queues block before changing to Neutral; upgrade adds 3 block.
    // decompiled/java-src/com/megacrit/cardcrawl/cards/purple/EmptyBody.java
    insert(cards, CardDef {
                id: "EmptyBody", name: "Empty Body", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 7,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::GainBlock(A::Block)),
                    E::Simple(SE::ChangeStance(Stance::Neutral)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "EmptyBody+", name: "Empty Body+", card_type: CardType::Skill,
                target: CardTarget::SelfTarget, cost: 1, base_damage: -1, base_block: 10,
                base_magic: -1, exhaust: false, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::GainBlock(A::Block)),
                    E::Simple(SE::ChangeStance(Stance::Neutral)),
                ], complex_hook: None,
            });
}
