//! Inserter: Every 2 turns, gain 1 orb slot.
//!
//! complex_hook needed: engine orb slot management requires engine access.
//! Old dispatch: increments INSERTER_COUNTER, adds ORB_SLOTS at threshold.

use crate::effects::declarative::{Effect, SimpleEffect, Target, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::AddStatus(Target::Player, sid::ORB_SLOTS, AmountSource::Fixed(1))),
];

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::TurnStart,
        condition: TriggerCondition::CounterReached,
        effects: &EFFECTS,
        counter: Some((sid::INSERTER_COUNTER, 2)),
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Inserter",
    name: "Inserter",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None, // TODO: wire complex_hook for engine orb slot management
    status_guard: None,
};
