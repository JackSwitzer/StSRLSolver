//! Incense Burner: Every 6 turns, gain 1 Intangible.

use crate::effects::declarative::{Effect, SimpleEffect, Target, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::AddStatus(Target::Player, sid::INTANGIBLE, AmountSource::Fixed(1))),
];

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::TurnStart,
        condition: TriggerCondition::CounterReached,
        effects: &EFFECTS,
        counter: Some((sid::INCENSE_BURNER_COUNTER, 6)),
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Incense Burner",
    name: "Incense Burner",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
};
