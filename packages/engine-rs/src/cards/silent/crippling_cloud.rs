use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Uncommon: Crippling Cloud (CripplingPoison) ---- (cost 2, 4 poison + 2 weak to all; +3/+1)
    insert(cards, CardDef {
                id: "Crippling Cloud", name: "Crippling Cloud", card_type: CardType::Skill,
                target: CardTarget::AllEnemy, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: true, enter_stance: None,
                effects: &["poison_all", "weak_all"], effect_data: &[
                    E::Simple(SE::AddStatus(T::AllEnemies, sid::POISON, A::Magic)),
                    E::Simple(SE::AddStatus(T::AllEnemies, sid::WEAKENED, A::Fixed(2))),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Crippling Cloud+", name: "Crippling Cloud+", card_type: CardType::Skill,
                target: CardTarget::AllEnemy, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 7, exhaust: true, enter_stance: None,
                effects: &["poison_all", "weak_all"], effect_data: &[
                    E::Simple(SE::AddStatus(T::AllEnemies, sid::POISON, A::Magic)),
                    E::Simple(SE::AddStatus(T::AllEnemies, sid::WEAKENED, A::Fixed(3))),
                ], complex_hook: None,
            });
}
