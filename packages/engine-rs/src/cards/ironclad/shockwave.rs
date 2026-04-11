use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Shockwave ---- (cost 2, 3 weak+vuln to all, exhaust; +2 magic)
    insert(cards, CardDef {
                id: "Shockwave", name: "Shockwave", card_type: CardType::Skill,
                target: CardTarget::AllEnemy, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: true, enter_stance: None,
                effects: &["weak_all", "vulnerable_all"], effect_data: &[
                    E::Simple(SE::AddStatus(T::AllEnemies, sid::WEAKENED, A::Magic)),
                    E::Simple(SE::AddStatus(T::AllEnemies, sid::VULNERABLE, A::Magic)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Shockwave+", name: "Shockwave+", card_type: CardType::Skill,
                target: CardTarget::AllEnemy, cost: 2, base_damage: -1, base_block: -1,
                base_magic: 5, exhaust: true, enter_stance: None,
                effects: &["weak_all", "vulnerable_all"], effect_data: &[
                    E::Simple(SE::AddStatus(T::AllEnemies, sid::WEAKENED, A::Magic)),
                    E::Simple(SE::AddStatus(T::AllEnemies, sid::VULNERABLE, A::Magic)),
                ], complex_hook: None,
            });
}
