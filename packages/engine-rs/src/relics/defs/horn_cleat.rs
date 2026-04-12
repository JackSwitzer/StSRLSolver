//! Horn Cleat: Initialize counter at combat start. Gain 14 Block on turn 2.

use crate::effects::declarative::{Effect, SimpleEffect, Target, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static INIT_EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::SetStatus(Target::Player, sid::HORN_CLEAT_COUNTER, AmountSource::Fixed(0))),
];

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::CombatStart,
        condition: TriggerCondition::Always,
        effects: &INIT_EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "HornCleat",
    name: "Horn Cleat",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
