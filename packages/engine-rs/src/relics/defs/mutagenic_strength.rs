//! Mutagenic Strength: +3 Strength and -3 at end of turn at combat start.

use crate::effects::declarative::{Effect, SimpleEffect, Target, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static EFFECTS: [Effect; 2] = [
    Effect::Simple(SimpleEffect::AddStatus(Target::Player, sid::STRENGTH, AmountSource::Fixed(3))),
    Effect::Simple(SimpleEffect::AddStatus(Target::Player, sid::LOSE_STRENGTH, AmountSource::Fixed(3))),
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
    id: "MutagenicStrength",
    name: "Mutagenic Strength",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
};
