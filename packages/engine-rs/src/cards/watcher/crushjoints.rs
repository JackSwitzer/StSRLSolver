use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java: CrushJoints.java queues damage, then CrushJointsAction.java checks
    // cardsPlayedThisCombat[-2] for SKILL before applying Vulnerable.
    // decompiled/java-src/com/megacrit/cardcrawl/cards/purple/CrushJoints.java
    // decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/CrushJointsAction.java
    insert(
        cards,
        CardDef {
            id: "CrushJoints",
            name: "Crush Joints",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 8,
            base_block: -1,
            base_magic: 1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Conditional(
                Cond::LastCardType(CardType::Skill),
                &[E::Simple(SE::AddStatus(
                    T::SelectedEnemy,
                    sid::VULNERABLE,
                    A::Magic,
                ))],
                &[],
            )],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "CrushJoints+",
            name: "Crush Joints+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 10,
            base_block: -1,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Conditional(
                Cond::LastCardType(CardType::Skill),
                &[E::Simple(SE::AddStatus(
                    T::SelectedEnemy,
                    sid::VULNERABLE,
                    A::Magic,
                ))],
                &[],
            )],
            complex_hook: None,
        },
    );
}
