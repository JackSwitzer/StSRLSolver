//! Boot: Minimum 5 damage per hit (if unblocked damage is 1-4, becomes 5).
//!
//! Damage modifier: called from the damage pipeline inline, not via
//! dispatch_trigger. EntityDef serves as documentation for future migration.
//! Old dispatch: apply_boot() in run.rs checks unblocked_damage range.

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
    complex_hook: None, // Wired later when damage pipeline migrates
    status_guard: None,
};
