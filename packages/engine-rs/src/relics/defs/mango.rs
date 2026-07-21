//! Mango: on pickup, gain 14 Max HP and heal by the same amount.
//!
//! Source: `reference/extracted/methods/relic/Mango.java` (`onEquip` calls
//! `increaseMaxHp(14, true)`). Run-level acquisition is handled in `run.rs`.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Mango",
    name: "Mango",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
