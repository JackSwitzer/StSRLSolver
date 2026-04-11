//! Mercury Hourglass: Deal 3 damage to ALL enemies at start of turn.

use crate::effects::declarative::{Effect, SimpleEffect, Target, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::DealDamage(Target::AllEnemies, AmountSource::Fixed(3))),
];

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::TurnStart,
        condition: TriggerCondition::Always,
        effects: &EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Mercury Hourglass",
    name: "Mercury Hourglass",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
};
