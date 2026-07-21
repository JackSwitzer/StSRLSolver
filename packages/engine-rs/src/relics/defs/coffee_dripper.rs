//! Coffee Dripper master-energy and campfire effects are run-level behavior in
//! RunEngine.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/CoffeeDripper.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Coffee Dripper",
    name: "Coffee Dripper",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
