//! Lizard Tail: when you would die, heal 50% max HP (once per run).
//! Death prevention is handled in the centralized revive pipeline.
//!
//! Source: `reference/extracted/methods/relic/LizardTail.java` (`onTrigger`
//! heals `maxHealth / 2`, clamped to at least 1, then marks the relic used).

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Lizard Tail",
    name: "Lizard Tail",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
