//! Busted Crown master-energy and card-reward effects are run-level behavior in
//! RunEngine.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/BustedCrown.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Busted Crown",
    name: "Busted Crown",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
