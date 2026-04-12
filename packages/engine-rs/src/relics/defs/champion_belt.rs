//! Champion Belt: Apply 1 Weak whenever applying Vulnerable.
//!
//! Debuff modifier: called inline when Vulnerable is applied, not via
//! dispatch_trigger. EntityDef serves as documentation for future migration.
//! Old dispatch: champion_belt_on_vulnerable() returns bool in run.rs.

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
    complex_hook: None, // Wired later when debuff pipeline migrates
    status_guard: None,
};
