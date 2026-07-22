use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
    // Intimidate.java iterates every monster and applies magicNumber Weak (1)
    // for zero energy, then exhausts; upgradeMagicNumber(1) raises Weak to 2.
    insert(
        cards,
        CardDef {
            id: "Intimidate",
            name: "Intimidate",
            card_type: CardType::Skill,
            target: CardTarget::AllEnemy,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 1,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::AllEnemies,
                sid::WEAKENED,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
    insert(
        cards,
        CardDef {
            id: "Intimidate+",
            name: "Intimidate+",
            card_type: CardType::Skill,
            target: CardTarget::AllEnemy,
            cost: 0,
            base_damage: -1,
            base_block: -1,
            base_magic: 2,
            exhaust: true,
            enter_stance: None,
            effect_data: &[E::Simple(SE::AddStatus(
                T::AllEnemies,
                sid::WEAKENED,
                A::Magic,
            ))],
            complex_hook: None,
        },
    );
}
