use super::prelude::*;
use crate::engine::CombatEngine;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

/// Entropic Brew: fill all empty potion slots with Block Potion (MCTS proxy).
/// Irreducible -- iterates over potion slots directly.
fn entropic_brew_hook(
    engine: &mut CombatEngine,
    _owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    crate::potions::fill_empty_potion_slots_with_proxy_rewards(&mut engine.state, "Block Potion");
}

pub static DEF: EntityDef = EntityDef {
    id: "EntropicBrew",
    name: "Entropic Brew",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(entropic_brew_hook),
    status_guard: None,
};
