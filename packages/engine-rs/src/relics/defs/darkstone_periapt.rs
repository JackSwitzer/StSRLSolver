//! Darkstone Periapt's curse-obtain effect is run-level behavior in RunEngine.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/DarkstonePeriapt.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Darkstone Periapt",
    name: "Darkstone Periapt",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
