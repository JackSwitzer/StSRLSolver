//! Calipers loses 15 Block instead of all Block at the normal turn-start clear;
//! the timing is handled in CombatEngine.
//! Sources: decompiled/java-src/com/megacrit/cardcrawl/relics/Calipers.java and
//! decompiled/java-src/com/megacrit/cardcrawl/actions/GameActionManager.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Calipers",
    name: "Calipers",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
