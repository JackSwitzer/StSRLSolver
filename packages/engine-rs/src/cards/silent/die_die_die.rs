use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Source: reference/extracted/methods/card/DieDieDie.java deals 13 damage
    // to all enemies for 1 energy and exhausts; upgrading adds 4 damage only.
    insert(
        cards,
        CardDef {
            id: "Die Die Die",
            name: "Die Die Die",
            card_type: CardType::Attack,
            target: CardTarget::AllEnemy,
            cost: 1,
            base_damage: 13,
            base_block: -1,
            base_magic: -1,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Die Die Die+",
            name: "Die Die Die+",
            card_type: CardType::Attack,
            target: CardTarget::AllEnemy,
            cost: 1,
            base_damage: 17,
            base_block: -1,
            base_magic: -1,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))],
            complex_hook: None,
        },
    );
}
