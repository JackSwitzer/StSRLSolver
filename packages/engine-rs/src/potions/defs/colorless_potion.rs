use super::prelude::*;
use crate::engine::CombatEngine;
use crate::effects::trigger::TriggerContext;

/// Colorless Potion: Add random Colorless card to hand (cost 0 this turn).
/// complex_hook because it requires RNG to pick a random colorless card.
fn colorless_potion_hook(_engine: &mut CombatEngine, _ctx: &TriggerContext) {
    // Stub: actual logic picks random colorless card, adds at cost 0
}

pub static DEF: EntityDef = EntityDef {
    id: "ColorlessPotion",
    name: "Colorless Potion",
    kind: EntityKind::Potion,
    triggers: &[],
    complex_hook: Some(colorless_potion_hook),
    status_guard: None,
};
