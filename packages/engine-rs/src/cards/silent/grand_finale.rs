use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Silent Rare: Grand Finale ---- (cost 0, 50 dmg AoE, only if draw pile empty; +10 dmg)
    insert(cards, CardDef {
                id: "Grand Finale", name: "Grand Finale", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 0, base_damage: 50, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["only_empty_draw"], effect_data: &[
                    E::Simple(SE::DealDamage(T::AllEnemies, A::Damage)),
                ], complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Grand Finale+", name: "Grand Finale+", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 0, base_damage: 60, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &["only_empty_draw"], effect_data: &[
                    E::Simple(SE::DealDamage(T::AllEnemies, A::Damage)),
                ], complex_hook: None,
            });
}
