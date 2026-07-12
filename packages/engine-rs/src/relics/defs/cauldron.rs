//! Cauldron's five-potion shop reward is run-level behavior in RunEngine.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/Cauldron.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Cauldron",
    name: "Cauldron",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
