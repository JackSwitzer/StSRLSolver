//! Bottled Flame selection and innate marking are run/master-deck behavior in
//! RunEngine.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/BottledFlame.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Bottled Flame",
    name: "Bottled Flame",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
