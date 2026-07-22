use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/purple/Wallop.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/watcher/WallopAction.java
    // GainBlock uses the target's final lastDamageTaken after the attack.
    insert(
        cards,
        CardDef {
            id: "Wallop",
            name: "Wallop",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 2,
            base_damage: 9,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                E::Simple(SE::GainBlock(A::TotalUnblockedDamage)),
            ],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Wallop+",
            name: "Wallop+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 2,
            base_damage: 12,
            base_block: -1,
            base_magic: -1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[
                E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage)),
                E::Simple(SE::GainBlock(A::TotalUnblockedDamage)),
            ],
            complex_hook: None,
        },
    );
}
