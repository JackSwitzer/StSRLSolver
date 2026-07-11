//! Astrolabe has no combat hook; its on-equip deck transformation is handled by
//! RunEngine.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/Astrolabe.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Astrolabe",
    name: "Astrolabe",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
