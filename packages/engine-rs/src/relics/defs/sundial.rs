//! Sundial: Every 3 shuffles, gain 2 Energy.

use crate::effects::declarative::{Effect, SimpleEffect, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::GainEnergy(AmountSource::Fixed(2))),
];

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::OnShuffle,
        condition: TriggerCondition::CounterReached,
        effects: &EFFECTS,
        counter: Some((sid::SUNDIAL_COUNTER, 3)),
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Sundial",
    name: "Sundial",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
