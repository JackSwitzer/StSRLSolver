//! Pandora's Box starter-card replacement is run-level behavior in RunEngine.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/PandorasBox.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Pandora's Box",
    name: "Pandora's Box",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
