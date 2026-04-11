//! Brimstone: +2 Strength to player, +1 Strength to all enemies at turn start.

use crate::effects::declarative::{Effect, SimpleEffect, Target, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static EFFECTS: [Effect; 2] = [
    Effect::Simple(SimpleEffect::AddStatus(Target::Player, sid::STRENGTH, AmountSource::Fixed(2))),
    Effect::Simple(SimpleEffect::AddStatus(Target::AllEnemies, sid::STRENGTH, AmountSource::Fixed(1))),
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
    id: "Brimstone",
    name: "Brimstone",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
};
