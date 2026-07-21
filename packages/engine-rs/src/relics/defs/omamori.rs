//! Omamori: negates the next two curses obtained.
//!
//! Source: `reference/extracted/methods/relic/Omamori.java` initializes the
//! counter to 2. The run-level obtain path owns the counter mutation.

use crate::effects::entity_def::{EntityDef, EntityKind};

pub static DEF: EntityDef = EntityDef {
    id: "Omamori",
    name: "Omamori",
    kind: EntityKind::Relic,
    triggers: &[],
    complex_hook: None,
    status_guard: None,
};
