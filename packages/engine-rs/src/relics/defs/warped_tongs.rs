//! Warped Tongs: upgrade a random card in hand at start of each turn.
//! Stub: card upgrade logic handled in engine turn-start pipeline.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "WarpedTongs",
    name: "Warped Tongs",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
