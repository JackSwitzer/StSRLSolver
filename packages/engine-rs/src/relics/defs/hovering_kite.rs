//! Hovering Kite: first time you discard a card each turn, gain 1 Energy.
//! Stub: discard-trigger energy handled in engine discard pipeline.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Hovering Kite",
    name: "Hovering Kite",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
