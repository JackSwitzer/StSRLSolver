//! Wrist Blade: +4 damage for 0-cost attacks.
//!
//! Passive damage bonus: called inline from damage calculation, not via
//! dispatch_trigger. EntityDef serves as documentation for future migration.
//! Old dispatch: wrist_blade_bonus() in run.rs returns +4 if owned.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::DamageCalculation,
        condition: TriggerCondition::Always,
        effects: &[], // Handled inline in damage calculation
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "WristBlade",
    name: "Wrist Blade",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None, // Wired later when damage pipeline migrates
    status_guard: None,
};
