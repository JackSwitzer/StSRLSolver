//! Hand Drill: Apply 2 Vulnerable when an enemy's block is broken.
//!
//! Block-break modifier: owned by the canonical inline damage pipeline.
//! This EntityDef records the trigger surface for export/runtime snapshots.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::OnBlockBroken,
        condition: TriggerCondition::Always,
        effects: &[], // Handled inline in damage pipeline
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "HandDrill",
    name: "Hand Drill",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
