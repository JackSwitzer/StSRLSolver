//! Violet Lotus: +1 energy when exiting Calm stance.
//!
//! Java: decompiled/java-src/com/megacrit/cardcrawl/relics/VioletLotus.java
//! The relic has no combat-start hook; `CombatEngine::change_stance` applies its
//! passive bonus only when the previous stance is Calm and the stance changes.

use crate::effects::entity_def::{EntityDef, EntityKind};

pub static DEF: EntityDef = EntityDef {
    id: "VioletLotus",
    name: "Violet Lotus",
    kind: EntityKind::Relic,
    triggers: &[],
    complex_hook: None,
    status_guard: None,
};
