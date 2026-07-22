use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // FearNoEvilAction.java checks the four attacking intent variants, queues
    // damage first, then enters Calm. Upgrade adds 3 damage.
    // decompiled/java-src/com/megacrit/cardcrawl/cards/purple/FearNoEvil.java
    // decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/FearNoEvilAction.java
    insert(
        cards,
        CardDef {
            id: "FearNoEvil",
            name: "Fear No Evil",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 8,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                E::Conditional(
                    Cond::EnemyAttacking,
                    &[E::Simple(SE::ChangeStance(Stance::Calm))],
                    &[],
                ),
            ],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "FearNoEvil+",
            name: "Fear No Evil+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 1,
            base_damage: 11,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                E::Conditional(
                    Cond::EnemyAttacking,
                    &[E::Simple(SE::ChangeStance(Stance::Calm))],
                    &[],
                ),
            ],
            complex_hook: None,
        },
    );
}
