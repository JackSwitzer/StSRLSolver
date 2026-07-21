//! Shovel's Dig campfire reward is handled by `RunEngine`.
//! Java: decompiled/java-src/com/megacrit/cardcrawl/relics/Shovel.java
//! Java: decompiled/java-src/com/megacrit/cardcrawl/vfx/campfire/CampfireDigEffect.java

use crate::effects::entity_def::{EntityDef, EntityKind};

pub static DEF: EntityDef = EntityDef {
    id: "Shovel",
    name: "Shovel",
    kind: EntityKind::Relic,
    triggers: &[],
    complex_hook: None,
    status_guard: None,
};
