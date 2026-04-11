use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Uncommon: Spot Weakness ---- (cost 1, +3 str if enemy attacking; +1 magic)
    insert(cards, CardDef {
                id: "Spot Weakness", name: "Spot Weakness", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 3, exhaust: false, enter_stance: None,
                effects: &["spot_weakness"], effect_data: &[
                    E::Conditional(
                        crate::effects::declarative::Condition::EnemyAttacking,
                        &[E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic))],
                        &[],
                    ),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Spot Weakness+", name: "Spot Weakness+", card_type: CardType::Skill,
                target: CardTarget::Enemy, cost: 1, base_damage: -1, base_block: -1,
                base_magic: 4, exhaust: false, enter_stance: None,
                effects: &["spot_weakness"], effect_data: &[
                    E::Conditional(
                        crate::effects::declarative::Condition::EnemyAttacking,
                        &[E::Simple(SE::AddStatus(T::Player, sid::STRENGTH, A::Magic))],
                        &[],
                    ),
                ], complex_hook: None,
            });
}
