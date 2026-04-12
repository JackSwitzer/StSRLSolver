//! Lizard Tail: when you would die, heal to 50% HP (once per combat).
//! Stub: death prevention handled in check_fairy_revive pipeline.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Lizard Tail",
    name: "Lizard Tail",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
