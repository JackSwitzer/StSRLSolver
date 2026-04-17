//! Boot: Minimum 5 damage per hit (if unblocked damage is 1-4, becomes 5).
//!
//! Damage modifier: owned by the canonical inline damage pipeline.
//! This EntityDef records the trigger surface for export/runtime snapshots.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::DamageResolved,
        condition: TriggerCondition::Always,
        effects: &[], // Handled inline in damage pipeline
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Boot",
    name: "Boot",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
