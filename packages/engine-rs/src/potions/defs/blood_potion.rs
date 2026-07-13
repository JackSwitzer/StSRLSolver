use super::prelude::*;
use crate::engine::CombatEngine;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn blood_potion_hook(
    engine: &mut CombatEngine,
    owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    let potency = match owner {
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

    // Source: reference/extracted/methods/potion/BloodPotion.java. Potency is
    // a percentage of max HP, floored before HealAction applies heal modifiers.
    let amount = (engine.state.player.max_hp * potency) / 100;
    engine.heal_player(amount);
}

pub static DEF: EntityDef = EntityDef {
    id: "BloodPotion",
    name: "Blood Potion",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(blood_potion_hook),
    status_guard: None,
};
