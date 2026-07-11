use super::prelude::*;
use crate::engine::CombatEngine;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn fire_potion_hook(
    engine: &mut CombatEngine,
    owner: crate::effects::runtime::EffectOwner,
    event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    let amount = match owner {
        crate::effects::runtime::EffectOwner::PotionSlot { slot } => {
            let idx = slot as usize;
            if idx >= engine.state.potions.len() {
                return;
            }
            crate::potions::effective_potency_runtime(
                &engine.state,
                &engine.state.potions[idx],
            )
        }
        _ => return,
    };
    if event.target_idx < 0 {
        return;
    }

    // FirePotion applies target powers only to THORNS damage before queuing
    // DamageAction. The enemy damage path then applies block and on-attacked
    // hooks without NORMAL-only modifiers.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/FirePotion.java
    // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/DamageInfo.java
    engine.deal_thorns_damage_to_enemy(event.target_idx as usize, amount);
}

pub static DEF: EntityDef = EntityDef {
    id: "FirePotion",
    name: "Fire Potion",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(fire_potion_hook),
    status_guard: None,
};
