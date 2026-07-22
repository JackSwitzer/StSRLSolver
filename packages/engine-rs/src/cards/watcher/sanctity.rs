use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // SanctityAction checks cardsPlayedThisCombat[size - 2], because the
    // current Sanctity is already recorded, and draws only after its block.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Sanctity.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/SanctityAction.java
    insert(
        cards,
        CardDef {
            id: "Sanctity",
            name: "Sanctity",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 6,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Conditional(
                Cond::LastCardType(CardType::Skill),
                &[E::Simple(SE::DrawCards(A::Magic))],
                &[],
            )],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Sanctity+",
            name: "Sanctity+",
            card_type: CardType::Skill,
            target: CardTarget::SelfTarget,
            cost: 1,
            base_damage: -1,
            base_block: 9,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Conditional(
                Cond::LastCardType(CardType::Skill),
                &[E::Simple(SE::DrawCards(A::Magic))],
                &[],
            )],
            complex_hook: None,
        },
    );
}
