use super::prelude::*;
use crate::engine::CombatEngine;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn explosive_potion_hook(
    engine: &mut CombatEngine,
    owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    let amount = match owner {
        crate::effects::runtime::EffectOwner::PotionSlot { slot } => {
            let idx = slot as usize;
            if idx >= engine.state.potions.len() {
                return;
            }
            crate::potions::effective_potency_runtime(&engine.state, &engine.state.potions[idx])
        }
        _ => return,
    };

    // ExplosivePotion passes `true` to createDamageMatrix, so Strength,
    // Vulnerable, Slow, Flight reduction, and Intangible do not modify the
    // precomputed amount. DamageAllEnemiesAction still resolves NORMAL-damage
    // block and onAttacked hooks for each living enemy.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/ExplosivePotion.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/DamageInfo.java
    for idx in engine.state.living_enemy_indices() {
        engine.deal_pure_normal_damage_to_enemy(idx, amount);
    }
}

pub static DEF: EntityDef = EntityDef {
    id: "ExplosivePotion",
    name: "Explosive Potion",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(explosive_potion_hook),
    status_guard: None,
};
