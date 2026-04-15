//! Champion Belt: Apply 1 Weak whenever applying Vulnerable.
//!
//! Debuff modifier: owned by the canonical inline debuff-application pipeline.
//! This EntityDef records the trigger surface for export/runtime snapshots.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::OnDebuffApplied,
        condition: TriggerCondition::Always,
        effects: &[], // Handled inline in debuff application
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Champion Belt",
    name: "Champion Belt",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
