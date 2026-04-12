//! Pen Nib: Every 10th Attack deals double damage.
//!
//! The actual double-damage logic is in the damage pipeline (complex_hook).
//! The counter tracking is declarative, but the effect (double damage) requires
//! hooking into the damage modifier pipeline, so this uses complex_hook.

use crate::effects::entity_def::{EntityDef, EntityKind};

pub static DEF: EntityDef = EntityDef {
    id: "Pen Nib",
    name: "Pen Nib",
    kind: EntityKind::Relic,
    triggers: &[],
    // Pen Nib's counter + double damage is handled in the damage pipeline
    // via check_pen_nib() in combat.rs. Cannot be pure declarative because
    // it modifies the damage multiplier, not an effect list.
    complex_hook: None,
    status_guard: None,
};
