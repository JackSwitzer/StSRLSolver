//! Dolly's Mirror deck-copy selection is run-level behavior in RunEngine.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/DollysMirror.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "DollysMirror",
    name: "Dolly's Mirror",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
