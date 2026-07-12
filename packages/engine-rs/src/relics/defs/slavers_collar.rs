//! SlaversCollar.beforeEnergyPrep temporarily changes energyMaster for elite
//! and boss combats. RunEngine applies that encounter-scoped behavior before
//! constructing CombatState.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/SlaversCollar.java.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "SlaversCollar",
    name: "Slaver's Collar",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
