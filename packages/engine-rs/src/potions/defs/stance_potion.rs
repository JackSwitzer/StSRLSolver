use super::prelude::*;
use crate::engine::CombatEngine;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn stance_potion_hook(
    engine: &mut CombatEngine,
    _owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    crate::potions::apply_stance_potion_effect(&mut engine.state);
}

/// Stance Potion: Enter Wrath or Calm stance.
pub static DEF: EntityDef = EntityDef {
    id: "StancePotion",
    name: "Stance Potion",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(stance_potion_hook),
    status_guard: None,
};
