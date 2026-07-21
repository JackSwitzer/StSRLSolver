//! Calling Bell curse confirmation and tiered relic rewards are handled by
//! RunEngine.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/CallingBell.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Calling Bell",
    name: "Calling Bell",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
