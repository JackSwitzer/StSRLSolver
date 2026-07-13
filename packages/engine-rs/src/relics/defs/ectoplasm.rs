//! Ectoplasm's master-energy and gold-gain effects are run-level behavior in
//! RunEngine.
//! Sources: decompiled/java-src/com/megacrit/cardcrawl/relics/Ectoplasm.java
//! and characters/AbstractPlayer.java::gainGold.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Ectoplasm",
    name: "Ectoplasm",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
