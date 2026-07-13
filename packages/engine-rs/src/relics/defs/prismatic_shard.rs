//! Prismatic Shard: all-color card rewards and one orb slot for Watcher.
//!
//! Source: `reference/extracted/methods/relic/PrismaticShard.java` — onEquip
//! sets masterMaxOrbs to one for a non-Defect character that had zero slots.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "PrismaticShard",
    name: "Prismatic Shard",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
