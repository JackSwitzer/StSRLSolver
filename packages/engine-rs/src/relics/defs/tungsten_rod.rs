//! Tungsten Rod: Reduce all HP loss by 1 (minimum 0).
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
    id: "TungstenRod",
    name: "Tungsten Rod",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
