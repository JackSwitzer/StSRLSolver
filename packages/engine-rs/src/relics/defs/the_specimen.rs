//! The Specimen: Transfer Poison from dead enemy to random living enemy.
//!
//! complex_hook needed: requires reading dead enemy's Poison stacks
//! and applying them to a random living enemy.
//! Old dispatch: reads dead_enemy.status(POISON), finds first alive
//! enemy, applies Poison stacks.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::OnEnemyDeath,
        condition: TriggerCondition::Always,
        effects: &[], // complex_hook handles Poison transfer
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "The Specimen",
    name: "The Specimen",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None, // TODO: wire complex_hook for Poison transfer
    status_guard: None,
};
