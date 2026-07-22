use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/ThirdEye.java
    // Generic base-block resolution runs before this declared Scry effect,
    // matching GainBlockAction followed by ScryAction.
    insert(
        cards,
        CardDef {
            id: "ThirdEye",
            name: "Third Eye",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 7,
            base_magic: 3,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::Scry(A::Magic))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "ThirdEye+",
            name: "Third Eye+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 9,
            base_magic: 5,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::Scry(A::Magic))],
            complex_hook: None,
        },
    );
}
