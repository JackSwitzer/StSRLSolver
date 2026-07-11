//! Runic Dome: +1 energy each turn, but enemy intents are hidden.
//!
//! Source: `reference/extracted/methods/relic/RunicDome.java` increments
//! energyMaster; `AbstractMonster.java` suppresses intent rendering.

use crate::effects::entity_def::{EntityDef, EntityKind};

pub static DEF: EntityDef = EntityDef {
    id: "Runic Dome",
    name: "Runic Dome",
    kind: EntityKind::Relic,
    triggers: &[],
    complex_hook: None,
    status_guard: None,
};
