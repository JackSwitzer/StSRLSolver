use crate::cards::prelude::*;

pub fn register(cards: &mut HashMap<&'static str, CardDef>) {
        // ---- Ironclad Rare: Reaper ---- (cost 2, 4 AoE dmg, heal for unblocked, exhaust; +1 dmg)
    insert(cards, CardDef {
                id: "Reaper", name: "Reaper", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 2, base_damage: 4, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &[], effect_data: &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))],
                complex_hook: Some(crate::effects::hooks_complex::hook_reaper),
            });
    insert(cards, CardDef {
                id: "Reaper+", name: "Reaper+", card_type: CardType::Attack,
                target: CardTarget::AllEnemy, cost: 2, base_damage: 5, base_block: -1,
                base_magic: -1, exhaust: false, enter_stance: None,
                effects: &[], effect_data: &[E::Simple(SE::DealDamage(T::AllEnemies, A::Damage))],
                complex_hook: Some(crate::effects::hooks_complex::hook_reaper),
            });
}
