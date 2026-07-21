//! Orrery's five card-reward choices are run-level behavior in RunEngine.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/Orrery.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Orrery",
    name: "Orrery",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
