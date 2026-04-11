//! Happy Flower: Every 3 turns, gain 1 Energy.

use crate::effects::declarative::{Effect, SimpleEffect, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::GainEnergy(AmountSource::Fixed(1))),
];

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::TurnStart,
        condition: TriggerCondition::CounterReached,
        effects: &EFFECTS,
        counter: Some((sid::HAPPY_FLOWER_COUNTER, 3)),
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Happy Flower",
    name: "Happy Flower",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
};
