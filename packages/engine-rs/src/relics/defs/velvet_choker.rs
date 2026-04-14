//! Velvet Choker: Track cards played per turn (limit 6).
//!
//! This EntityDef handles the OnAnyCardPlayed counter increment and turn reset.
//! The actual play gate reads that canonical counter directly in the engine.

use crate::effects::declarative::{Effect, SimpleEffect, Target, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static INCREMENT_EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::IncrementCounter(sid::VELVET_CHOKER_COUNTER, 1)),
];

static RESET_EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::SetStatus(Target::Player, sid::VELVET_CHOKER_COUNTER, AmountSource::Fixed(0))),
];

static TRIGGERS: [TriggeredEffect; 2] = [
    TriggeredEffect {
        trigger: Trigger::OnAnyCardPlayed,
        condition: TriggerCondition::Always,
        effects: &INCREMENT_EFFECTS,
        counter: None,
    },
    TriggeredEffect {
        trigger: Trigger::TurnStart,
        condition: TriggerCondition::Always,
        effects: &RESET_EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Velvet Choker",
    name: "Velvet Choker",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
