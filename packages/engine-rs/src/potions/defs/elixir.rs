use super::prelude::*;
use crate::engine::CombatEngine;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

/// Elixir: exhaust entire hand. Irreducible -- needs engine access.
fn elixir_hook(
    engine: &mut CombatEngine,
    _owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    engine.state.exhaust_pile.extend(engine.state.hand.drain(..));
}

pub static DEF: EntityDef = EntityDef {
    id: "Elixir",
    name: "Elixir",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(elixir_hook),
    status_guard: None,
};
