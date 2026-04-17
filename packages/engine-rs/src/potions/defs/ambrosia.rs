use super::prelude::*;
use crate::engine::CombatEngine;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn ambrosia_hook(
    engine: &mut CombatEngine,
    _owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    crate::potions::apply_ambrosia_effect(&mut engine.state);
}

pub static DEF: EntityDef = EntityDef {
    id: "Ambrosia",
    name: "Ambrosia",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(ambrosia_hook),
    status_guard: None,
};
