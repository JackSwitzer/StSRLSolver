//! Black Star has no combat mutation; RunEngine adds the extra elite relic
//! reward through its canonical relic flag.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/BlackStar.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Black Star",
    name: "Black Star",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
