use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // ThunderClap.java queues its 4-damage AoE before applying one
    // Vulnerable to every monster; upgradeDamage(3) changes only damage.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/red/ThunderClap.java
    insert(
        cards,
        CardDef {
            id: "Thunderclap",
            name: "Thunderclap",
            card_type: CardType::Attack,
            target: CardTarget::AllEnemy,
            cost: 1,
            base_damage: 4,
            base_block: -1,
            base_magic: 1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::AllEnemies,
                sid::VULNERABLE,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Thunderclap+",
            name: "Thunderclap+",
            card_type: CardType::Attack,
            target: CardTarget::AllEnemy,
            cost: 1,
            base_damage: 7,
            base_block: -1,
            base_magic: 1,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::AllEnemies,
                sid::VULNERABLE,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
}
