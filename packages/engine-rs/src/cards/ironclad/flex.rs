use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Flex.java applies Strength first, then debuff-typed
    // LoseStrengthPower for the same amount. Artifact can therefore block
    // only the delayed loss. Upgrading raises both amounts from 2 to 4.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/Flex.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/powers/LoseStrengthPower.java
    insert(
        cards,
        CardDef {
            id: "Flex",
            name: "Flex",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic)),
                E::Simple(SE::AddStatus(T::Player, sid::LOSE_STRENGTH, A::Magic)),
            ],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Flex+",
            name: "Flex+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 4,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic)),
                E::Simple(SE::AddStatus(T::Player, sid::LOSE_STRENGTH, A::Magic)),
            ],
            complex_hook: None,
        },
    );
}
