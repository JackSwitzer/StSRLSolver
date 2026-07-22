use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // EndlessAgony.java deals 4, exhausts, and makes a stat-equivalent copy
    // when drawn; upgradeDamage(2) is its only upgrade.
    // Java: reference/extracted/methods/card/EndlessAgony.java
    insert(
        cards,
        CardDef {
            id: "Endless Agony",
            name: "Endless Agony",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 0,
            base_damage: 4,
            base_block: -1,
            base_magic: -1,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Endless Agony+",
            name: "Endless Agony+",
            card_type: CardType::Attack,
            target: CardTarget::Enemy,
            cost: 0,
            base_damage: 6,
            base_block: -1,
            base_magic: -1,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DealDamage(T::SelectedEnemy, A::Damage))],
            complex_hook: None,
        },
    );
}
