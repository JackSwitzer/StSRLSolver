use super::prelude::*;
use crate::engine::CombatEngine;
use crate::effects::trigger::TriggerContext;

/// Entropic Brew: fill all empty potion slots with Block Potion (MCTS proxy).
/// Irreducible -- iterates over potion slots directly.
fn entropic_brew_hook(engine: &mut CombatEngine, _ctx: &TriggerContext) {
    for slot in &mut engine.state.potions {
        if slot.is_empty() {
            *slot = "Block Potion".to_string();
        }
    }
}

pub static DEF: EntityDef = EntityDef {
    id: "EntropicBrew",
    name: "Entropic Brew",
    kind: EntityKind::Potion,
    triggers: &[],
    complex_hook: Some(entropic_brew_hook),
    status_guard: None,
};
