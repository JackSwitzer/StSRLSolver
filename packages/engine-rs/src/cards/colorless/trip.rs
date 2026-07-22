use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Trip.java applies two Vulnerable to one enemy for free; upgrading
    // changes only its target to ALL_ENEMY.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/colorless/Trip.java
    insert(
        cards,
        CardDef {
            id: "Trip",
            name: "Trip",
            card_type: CardType::Skill,
            target: CardTarget::Enemy,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::SelectedEnemy,
                sid::VULNERABLE,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Trip+",
            name: "Trip+",
            card_type: CardType::Skill,
            target: CardTarget::AllEnemy,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 2,
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
