//! Bottled Lightning selection and innate marking are run/master-deck behavior
//! in RunEngine.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/BottledLightning.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Bottled Lightning",
    name: "Bottled Lightning",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
