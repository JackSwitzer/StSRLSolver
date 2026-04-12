//! Hand Drill: Apply 2 Vulnerable when an enemy's block is broken.
//!
//! Block-break modifier: called inline in damage pipeline, not via
//! dispatch_trigger. EntityDef serves as documentation for future migration.
//! Old dispatch: hand_drill_on_block_break() in run.rs.

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
    complex_hook: None, // Wired later when damage pipeline migrates
    status_guard: None,
};
