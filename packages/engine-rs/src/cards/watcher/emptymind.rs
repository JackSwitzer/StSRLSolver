use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java queues draw before changing to Neutral; upgrade raises draw 2 -> 3.
    // decompiled/java-src/com/megacrit/cardcrawl/cards/purple/EmptyMind.java
    insert(
        cards,
        CardDef {
            id: "EmptyMind",
            name: "Empty Mind",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
                E::Simple(SE::ChangeStance(Stance::Neutral)),
            ],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "EmptyMind+",
            name: "Empty Mind+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: -1,
            base_magic: 3,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::DrawCards(A::Magic)),
                E::Simple(SE::ChangeStance(Stance::Neutral)),
            ],
            complex_hook: None,
        },
    );
}
