//! Bloody Idol heals on run-level gold gain; RunEngine owns that callback.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/BloodyIdol.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Bloody Idol",
    name: "Bloody Idol",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
