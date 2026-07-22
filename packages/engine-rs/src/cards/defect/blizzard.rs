use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Source: cards/blue/Blizzard.java counts every Frost channeled this
    // combat, multiplies by magic 2, then runs normal AoE damage powers;
    // the upgrade raises only that multiplier to 3.
    insert(
        cards,
        CardDef {
            id: "Blizzard",
            name: "Blizzard",
            card_type: CardType::Attack,
            target: CardTarget::AllEnemy,
            cost: 1,
            base_damage: 0,
            base_block: -1,
            base_magic: 2,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DealDamage(
                T::AllEnemies,
                A::StatusValueTimesMagic(sid::FROST_CHANNELED),
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Blizzard+",
            name: "Blizzard+",
            card_type: CardType::Attack,
            target: CardTarget::AllEnemy,
            cost: 1,
            base_damage: 0,
            base_block: -1,
            base_magic: 3,
            exhaust: false,
            enter_stance: None,
            effect_data: &[E::Simple(SE::DealDamage(
                T::AllEnemies,
                A::StatusValueTimesMagic(sid::FROST_CHANNELED),
            ))],
            complex_hook: None,
        },
    );
}
