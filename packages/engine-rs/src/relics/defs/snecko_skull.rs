//! Snecko Skull (Snake Skull): +1 Poison per Poison application.
//!
//! Passive poison bonus: owned by the canonical inline poison-application path.
//! This EntityDef records the trigger surface for export/runtime snapshots.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::OnPoisonApplied,
        condition: TriggerCondition::Always,
        effects: &[], // Handled inline in Poison application
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "SneckoSkull",
    name: "Snecko Skull",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
