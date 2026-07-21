//! Empty Cage: remove exactly two purgeable master-deck cards when possible.
//!
//! Source: `reference/extracted/methods/relic/EmptyCage.java` auto-removes a
//! pool of at most two cards and otherwise opens an exact two-card selection.

use crate::effects::entity_def::{EntityDef, EntityKind};

pub static DEF: EntityDef = EntityDef {
    id: "Empty Cage",
    name: "Empty Cage",
    kind: EntityKind::Relic,
    triggers: &[],
    complex_hook: None,
    status_guard: None,
};
