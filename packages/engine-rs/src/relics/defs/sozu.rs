//! Sozu's master-energy and potion-obtain restrictions are run-level behavior
//! in RunEngine.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/Sozu.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Sozu",
    name: "Sozu",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
