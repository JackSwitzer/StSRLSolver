use super::prelude::*;
use crate::engine::CombatEngine;
use crate::effects::trigger::TriggerContext;

/// Cunning Potion: Add Shiv+ cards to hand (3 base, 6 with Sacred Bark).
fn cunning_potion_hook(_engine: &mut CombatEngine, _ctx: &TriggerContext) {
    // Stub: actual logic adds upgraded Shivs to hand
}

pub static DEF: EntityDef = EntityDef {
    id: "CunningPotion",
    name: "Cunning Potion",
    kind: EntityKind::Potion,
    triggers: &[],
    complex_hook: Some(cunning_potion_hook),
    status_guard: None,
};
