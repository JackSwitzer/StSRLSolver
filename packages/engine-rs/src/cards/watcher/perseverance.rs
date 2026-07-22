use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // onRetained upgrades this card instance's block by magicNumber; the
    // upgrade changes both its starting block and per-retain growth.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Perseverance.java
    insert(
        cards,
        CardDef {
            id: "Perseverance",
            name: "Perseverance",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 5,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::GainBlock(A::Block))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Perseverance+",
            name: "Perseverance+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 7,
            base_magic: 3,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::GainBlock(A::Block))],
            complex_hook: None,
        },
    );
}
