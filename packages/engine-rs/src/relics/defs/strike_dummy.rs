//! Strike Dummy: +3 damage per Strike card in deck.
//!
//! Passive damage bonus: called inline from damage calculation, not via
//! dispatch_trigger. EntityDef serves as documentation for future migration.
//! Old dispatch: strike_dummy_bonus() in run.rs returns +3 if owned.

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
    id: "StrikeDummy",
    name: "Strike Dummy",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None, // Wired later when damage pipeline migrates
    status_guard: None,
};
