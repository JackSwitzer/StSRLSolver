//! Dream Catcher's post-rest card reward is run-level behavior in RunEngine.
//! Sources: decompiled/java-src/com/megacrit/cardcrawl/relics/DreamCatcher.java
//! and vfx/campfire/CampfireSleepEffect.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Dream Catcher",
    name: "Dream Catcher",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
