use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Intimidate ---- (cost 0, 1 weak to all, exhaust; +1 magic)
    insert(cards, CardDef {
                id: "Intimidate", name: "Intimidate", card_type: CardType::Skill,
                target: CardTarget::AllEnemy, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 1, exhaust: true, enter_stance: None,
                effects: &["weak_all"], effect_data: &[
                    E::Simple(SE::AddStatus(T::AllEnemies, sid::WEAKENED, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Intimidate+", name: "Intimidate+", card_type: CardType::Skill,
                target: CardTarget::AllEnemy, cost: 0, base_damage: -1, base_block: -1,
                base_magic: 2, exhaust: true, enter_stance: None,
                effects: &["weak_all"], effect_data: &[
                    E::Simple(SE::AddStatus(T::AllEnemies, sid::WEAKENED, A::Magic)),
                ], complex_hook: None,
            });
}
