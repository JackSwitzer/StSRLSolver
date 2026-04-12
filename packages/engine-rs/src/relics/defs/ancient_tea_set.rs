//! Ancient Tea Set: if you rested at a campfire last, gain 2 energy on turn 1.
//! Stub: room tracking handled externally (Python/run layer).

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static TRIGGERS: [TriggeredEffect; 0] = [];

pub static DEF: EntityDef = EntityDef {
    id: "Ancient Tea Set",
    name: "Ancient Tea Set",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
