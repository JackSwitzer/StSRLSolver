//! Peace Pipe's Toke campfire option is handled by `RunEngine`.
//! Java: decompiled/java-src/com/megacrit/cardcrawl/relics/PeacePipe.java
//! Java: decompiled/java-src/com/megacrit/cardcrawl/vfx/campfire/CampfireTokeEffect.java

use crate::effects::entity_def::{EntityDef, EntityKind};

pub static DEF: EntityDef = EntityDef {
    id: "Peace Pipe",
    name: "Peace Pipe",
    kind: EntityKind::Relic,
    triggers: &[],
    complex_hook: None,
    status_guard: None,
};
