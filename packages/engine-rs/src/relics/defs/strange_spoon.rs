//! Strange Spoon: 50% chance exhausted cards go to discard instead.
//! Passive check via has_relic in exhaust pipeline.

use crate::effects::entity_def::{EntityDef, EntityKind};

pub static DEF: EntityDef = EntityDef {
    id: "Strange Spoon",
    name: "Strange Spoon",
    kind: EntityKind::Relic,
    triggers: &[],
    complex_hook: None,
    status_guard: None,
};
