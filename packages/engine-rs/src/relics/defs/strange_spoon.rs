//! Strange Spoon: 50% chance exhausted non-Power cards follow their normal
//! post-use destination instead.
//! Java: decompiled/java-src/com/megacrit/cardcrawl/relics/StrangeSpoon.java
//! Java: decompiled/java-src/com/megacrit/cardcrawl/actions/utility/UseCardAction.java

use crate::effects::entity_def::{EntityDef, EntityKind};

pub static DEF: EntityDef = EntityDef {
    id: "Strange Spoon",
    name: "Strange Spoon",
    kind: EntityKind::Relic,
    triggers: &[],
    complex_hook: None,
    status_guard: None,
};
