use super::prelude::*;
use crate::engine::CombatEngine;
use crate::effects::trigger::TriggerContext;

/// Stance Potion: Enter Wrath or Calm stance.
/// complex_hook because the choice depends on current stance and has
/// side-effects (Calm exit energy, etc.).
fn stance_potion_hook(_engine: &mut CombatEngine, _ctx: &TriggerContext) {
    // Stub: actual logic toggles between Calm and Wrath
}

pub static DEF: EntityDef = EntityDef {
    id: "StancePotion",
    name: "Stance Potion",
    kind: EntityKind::Potion,
    triggers: &[],
    complex_hook: Some(stance_potion_hook),
    status_guard: None,
};
