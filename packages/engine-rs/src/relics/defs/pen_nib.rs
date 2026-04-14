//! Pen Nib: Every 10th Attack deals double damage.
//!
//! The actual double-damage logic is in the card damage pipeline.
//! The runtime-visible state for Pen Nib remains the player counter status,
//! while the damage doubling is consumed directly by the engine play path.

use crate::effects::entity_def::{EntityDef, EntityKind};

pub static DEF: EntityDef = EntityDef {
    id: "Pen Nib",
    name: "Pen Nib",
    kind: EntityKind::Relic,
    triggers: &[],
    // Pen Nib's counter + double damage is handled in the damage pipeline.
    // It cannot be pure declarative because it modifies the active attack's
    // damage multiplier rather than emitting a follow-up effect.
    complex_hook: None,
    status_guard: None,
};
