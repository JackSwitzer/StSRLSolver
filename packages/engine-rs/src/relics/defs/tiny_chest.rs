//! Tiny Chest's fourth-mystery-room treasure override is handled by RunEngine.
//! Java: decompiled/java-src/com/megacrit/cardcrawl/relics/TinyChest.java
//! Java: decompiled/java-src/com/megacrit/cardcrawl/helpers/EventHelper.java

use crate::effects::entity_def::{EntityDef, EntityKind};

pub static DEF: EntityDef = EntityDef {
    id: "Tiny Chest",
    name: "Tiny Chest",
    kind: EntityKind::Relic,
    triggers: &[],
    complex_hook: None,
    status_guard: None,
};
