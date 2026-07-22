use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Safety.java costs 1, gains 12 block, self-retains, and exhausts;
    // upgradeBlock(4) raises only its block to 16.
    // Java: reference/extracted/methods/card/Safety.java
    insert(
        cards,
        CardDef {
            id: "Safety",
            name: "Safety",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 12,
            base_magic: -1,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::GainBlock(A::Block))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Safety+",
            name: "Safety+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 16,
            base_magic: -1,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::GainBlock(A::Block))],
            complex_hook: None,
        },
    );
}
