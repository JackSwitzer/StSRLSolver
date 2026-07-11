//! Cursed Key master-energy and chest-open effects are run-level behavior in
//! RunEngine.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/CursedKey.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Cursed Key",
    name: "Cursed Key",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
