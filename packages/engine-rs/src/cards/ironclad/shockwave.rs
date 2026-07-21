use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // Shockwave.java queues Weak then Vulnerable for every monster, each
        // for 3 turns, and exhausts; upgradeMagicNumber(2) makes both 5.
        // Java: reference/extracted/methods/card/Shockwave.java
    insert(cards, CardDef {
                id: "Shockwave", name: "Shockwave", card_type: CardType::Skill,
                target: CardTarget::AllEnemy, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::AllEnemies, sid::WEAKENED, A::Magic)),
                    E::Simple(SE::AddStatus(T::AllEnemies, sid::VULNERABLE, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Shockwave+", name: "Shockwave+", card_type: CardType::Skill,
                target: CardTarget::AllEnemy, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 5, exhaust: true, enter_stance: None,
                effect_data: &[
                    E::Simple(SE::AddStatus(T::AllEnemies, sid::WEAKENED, A::Magic)),
                    E::Simple(SE::AddStatus(T::AllEnemies, sid::VULNERABLE, A::Magic)),
                ], complex_hook: None,
            });
}
