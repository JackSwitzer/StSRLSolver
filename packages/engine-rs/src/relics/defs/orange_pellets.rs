//! Orange Pellets: Initialize type tracking statuses at combat start.
//! Playing ATK + SKL + POW in same combat clears all debuffs.

use crate::effects::declarative::{Effect, SimpleEffect, Target, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static EFFECTS: [Effect; 3] = [
    Effect::Simple(SimpleEffect::SetStatus(Target::Player, sid::OP_ATTACK, AmountSource::Fixed(0))),
    Effect::Simple(SimpleEffect::SetStatus(Target::Player, sid::OP_SKILL, AmountSource::Fixed(0))),
    Effect::Simple(SimpleEffect::SetStatus(Target::Player, sid::OP_POWER, AmountSource::Fixed(0))),
];

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::CombatStart,
        condition: TriggerCondition::Always,
        effects: &EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "OrangePellets",
    name: "Orange Pellets",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
