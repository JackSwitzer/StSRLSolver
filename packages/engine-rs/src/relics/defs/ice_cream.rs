//! Ice Cream's energy carryover is handled by CombatEngine's turn recharge.
//! Sources: decompiled/java-src/com/megacrit/cardcrawl/relics/IceCream.java and
//! core/EnergyManager.java::recharge.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Ice Cream",
    name: "Ice Cream",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
