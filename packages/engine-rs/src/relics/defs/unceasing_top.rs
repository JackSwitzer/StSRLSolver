//! Unceasing Top's empty-hand draw is handled in the card post-play pipeline.
//! Java: decompiled/java-src/com/megacrit/cardcrawl/relics/UnceasingTop.java

use crate::effects::entity_def::{EntityDef, EntityKind};

pub static DEF: EntityDef = EntityDef {
    id: "Unceasing Top",
    name: "Unceasing Top",
    kind: EntityKind::Relic,
    triggers: &[],
    complex_hook: None,
    status_guard: None,
};
