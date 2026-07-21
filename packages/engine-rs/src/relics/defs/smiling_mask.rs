//! Smiling Mask's flat 50-gold shop removal price is handled by `RunEngine`.
//! Java: decompiled/java-src/com/megacrit/cardcrawl/relics/SmilingMask.java
//! Java: decompiled/java-src/com/megacrit/cardcrawl/shop/ShopScreen.java

use crate::effects::entity_def::{EntityDef, EntityKind};

pub static DEF: EntityDef = EntityDef {
    id: "Smiling Mask",
    name: "Smiling Mask",
    kind: EntityKind::Relic,
    triggers: &[],
    complex_hook: None,
    status_guard: None,
};
