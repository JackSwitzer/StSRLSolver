use super::prelude::*;
use crate::engine::CombatEngine;
use crate::effects::trigger::TriggerContext;

/// Ambrosia: Enter Divinity stance.
/// complex_hook because stance changes have side-effects (Calm exit energy,
/// Divinity +3 energy, relic triggers).
fn ambrosia_hook(_engine: &mut CombatEngine, _ctx: &TriggerContext) {
    // Stub: actual logic enters Divinity stance with full side-effects
}

pub static DEF: EntityDef = EntityDef {
    id: "Ambrosia",
    name: "Ambrosia",
    kind: EntityKind::Potion,
    triggers: &[],
    complex_hook: Some(ambrosia_hook),
    status_guard: None,
};
