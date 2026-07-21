//! Bottled Tornado selection and innate marking are run/master-deck behavior
//! in RunEngine.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/BottledTornado.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Bottled Tornado",
    name: "Bottled Tornado",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
