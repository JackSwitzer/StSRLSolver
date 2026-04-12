//! Tungsten Rod: Reduce all HP loss by 1 (minimum 0).
//!
//! Damage modifier: called from the damage pipeline inline, not via
//! dispatch_trigger. EntityDef serves as documentation for future migration.
//! Old dispatch: apply_tungsten_rod() in run.rs subtracts 1 from damage.

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
    complex_hook: None, // Wired later when damage pipeline migrates
    status_guard: None,
};
