use super::prelude::*;
use crate::engine::CombatEngine;
use crate::effects::trigger::TriggerContext;

/// Power Potion: Add random Power card to hand (cost 0 this turn).
/// complex_hook because it requires RNG to pick a random power.
fn power_potion_hook(_engine: &mut CombatEngine, _ctx: &TriggerContext) {
    // Stub: actual logic picks random power from class pool, adds at cost 0
}

pub static DEF: EntityDef = EntityDef {
    id: "PowerPotion",
    name: "Power Potion",
    kind: EntityKind::Potion,
    triggers: &[],
    complex_hook: Some(power_potion_hook),
    status_guard: None,
};
