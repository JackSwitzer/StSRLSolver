use crate::cards::prelude::*;
use crate::effects::declarative::{AmountSource as A, Effect, SimpleEffect as SE, Target as T};

static REAPER_EFFECT: [Effect; 2] = [
    Effect::Simple(SE::DealDamage(T::AllEnemies, A::Damage)),
    Effect::Simple(SE::HealHp(T::Player, A::TotalUnblockedDamage)),
];

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Rare: Reaper ---- (cost 2, 4 AoE dmg, heal for unblocked, exhaust; +1 dmg)
    insert(cards, CardDef {
                id: "Reaper", name: "Reaper", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 2, base_damage: 4, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &[], effect_data: &REAPER_EFFECT, complex_hook: None,
            });
    insert(cards, CardDef {
                id: "Reaper+", name: "Reaper+", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 2, base_damage: 5, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &[], effect_data: &REAPER_EFFECT, complex_hook: None,
            });
}
