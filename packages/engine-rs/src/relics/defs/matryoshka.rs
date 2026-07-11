//! Matryoshka: the next two non-boss chests add one extra relic reward.
//!
//! Source: `reference/extracted/methods/relic/Matryoshka.java`. The constructor
//! starts at two charges; `onChestOpen` consumes one and rolls COMMON with 75%
//! probability, otherwise UNCOMMON. Run-level resolution lives in `run.rs`.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Matryoshka",
    name: "Matryoshka",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
