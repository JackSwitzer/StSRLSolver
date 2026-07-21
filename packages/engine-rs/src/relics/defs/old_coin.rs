//! Old Coin's 300-gold on-equip effect is handled by `RunEngine`.
//! Java: decompiled/java-src/com/megacrit/cardcrawl/relics/OldCoin.java

use crate::effects::entity_def::{EntityDef, EntityKind};

pub static DEF: EntityDef = EntityDef {
    id: "Old Coin",
    name: "Old Coin",
    kind: EntityKind::Relic,
    triggers: &[],
    complex_hook: None,
    status_guard: None,
};
