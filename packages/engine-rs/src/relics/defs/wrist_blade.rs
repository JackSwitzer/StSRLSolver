//! Wrist Blade: +4 damage for 0-cost attacks.
//!
//! Passive damage bonus: owned by the canonical inline damage calculation path.
//! This EntityDef records the trigger surface for export/runtime snapshots.

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
    complex_hook: None,
    status_guard: None,
};
