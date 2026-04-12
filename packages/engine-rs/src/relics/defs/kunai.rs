//! Kunai: Every 3 Attacks played, gain 1 Dexterity.

use crate::effects::declarative::{Effect, SimpleEffect, Target, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::AddStatus(Target::Player, sid::DEXTERITY, AmountSource::Fixed(1))),
];

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::OnAttackPlayed,
        condition: TriggerCondition::CounterReached,
        effects: &EFFECTS,
        counter: Some((sid::KUNAI_COUNTER, 3)),
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Kunai",
    name: "Kunai",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
