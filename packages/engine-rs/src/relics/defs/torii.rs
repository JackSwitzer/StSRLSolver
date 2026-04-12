//! Torii: Unblocked attack damage of 2-5 is reduced to 1.
//!
//! Damage modifier: called from the damage pipeline inline, not via
//! dispatch_trigger. EntityDef serves as documentation for future migration.
//! Old dispatch: apply_torii() in run.rs checks 1 < damage <= 5.

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
    id: "Torii",
    name: "Torii",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None, // Wired later when damage pipeline migrates
    status_guard: None,
};
