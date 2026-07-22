use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // HeadStompAction checks the card immediately before this Sash Whip and
    // applies Weak only when that previous card was an Attack.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/SashWhip.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/HeadStompAction.java
    insert(
        cards,
        CardDef {
            id: "SashWhip",
            name: "Sash Whip",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 8,
            base_block: -1,
            base_magic: 1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Conditional(
                Cond::LastCardType(CardType::Attack),
                &[E::Simple(SE::AddStatus(
                    T::SelectedEnemy,
                    sid::WEAKENED,
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
            id: "SashWhip+",
            name: "Sash Whip+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 10,
            base_block: -1,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Conditional(
                Cond::LastCardType(CardType::Attack),
                &[E::Simple(SE::AddStatus(
                    T::SelectedEnemy,
                    sid::WEAKENED,
                    A::Magic,
                ))],
                &[],
            )],
            complex_hook: None,
        },
    );
}
