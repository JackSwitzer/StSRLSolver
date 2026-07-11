//! Juzu Bracelet: mystery rooms cannot resolve into monster rooms.
//!
//! Source: decompiled `helpers/EventHelper.java`: after a MONSTER result is
//! rolled, owning canonical relic ID `Juzu Bracelet` converts it to EVENT.
//! Run-level mystery-room resolution lives in `run.rs`.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Juzu Bracelet",
    name: "Juzu Bracelet",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
