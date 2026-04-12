//! Stone Calendar: Initialize counter at combat start. Deal 52 damage on turn 7.
//!
//! Counter increments each turn start. At turn end, when counter reaches 7,
//! deal 52 damage to ALL enemies.

use crate::effects::declarative::{Effect, SimpleEffect, Target, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static INIT_EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::SetStatus(Target::Player, sid::STONE_CALENDAR_COUNTER, AmountSource::Fixed(0))),
];

static FIRE_EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::DealDamage(Target::AllEnemies, AmountSource::Fixed(52))),
];

static TRIGGERS: [TriggeredEffect; 2] = [
    TriggeredEffect {
        trigger: Trigger::CombatStart,
        condition: TriggerCondition::Always,
        effects: &INIT_EFFECTS,
        counter: None,
    },
    TriggeredEffect {
        trigger: Trigger::TurnEnd,
        condition: TriggerCondition::CounterReached,
        effects: &FIRE_EFFECTS,
        counter: Some((sid::STONE_CALENDAR_COUNTER, 7)),
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "StoneCalendar",
    name: "Stone Calendar",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
