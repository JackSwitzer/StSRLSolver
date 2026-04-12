//! Captain's Wheel: Initialize counter at combat start. Gain 18 Block on turn 3.

use crate::effects::declarative::{Effect, SimpleEffect, Target, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static INIT_EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::SetStatus(Target::Player, sid::CAPTAINS_WHEEL_COUNTER, AmountSource::Fixed(0))),
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
    id: "CaptainsWheel",
    name: "Captain's Wheel",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
