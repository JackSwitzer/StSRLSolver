//! Snecko Skull (Snake Skull): +1 Poison per Poison application.
//!
//! Passive poison bonus: called inline when Poison is applied, not via
//! dispatch_trigger. EntityDef serves as documentation for future migration.
//! Old dispatch: snecko_skull_bonus() in run.rs returns +1 if owned.

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
    complex_hook: None, // Wired later when poison pipeline migrates
    status_guard: None,
};
