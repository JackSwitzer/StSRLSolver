use super::prelude::*;
use crate::engine::CombatEngine;
use crate::effects::trigger::TriggerContext;

/// Potion of Capacity: Gain orb slots (2 base, 4 with Sacred Bark).
/// complex_hook because it must call engine.add_orb_slot() repeatedly.
fn potion_of_capacity_hook(_engine: &mut CombatEngine, _ctx: &TriggerContext) {
    // Stub: actual logic adds potency orb slots via orb_slots.add_slot()
}

pub static DEF: EntityDef = EntityDef {
    id: "PotionOfCapacity",
    name: "Potion of Capacity",
    kind: EntityKind::Potion,
    triggers: &[],
    complex_hook: Some(potion_of_capacity_hook),
    status_guard: None,
};
