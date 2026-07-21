//! Toxic Egg 2's Skill-card acquisition upgrades are run-level behavior in
//! RunEngine.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/ToxicEgg2.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Toxic Egg 2",
    name: "Toxic Egg 2",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
