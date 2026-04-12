//! Ornamental Fan: Every 3 Attacks played, gain 4 Block.

use crate::effects::declarative::{Effect, SimpleEffect, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::GainBlock(AmountSource::Fixed(4))),
];

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::OnAttackPlayed,
        condition: TriggerCondition::CounterReached,
        effects: &EFFECTS,
        counter: Some((sid::ORNAMENTAL_FAN_COUNTER, 3)),
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Ornamental Fan",
    name: "Ornamental Fan",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
