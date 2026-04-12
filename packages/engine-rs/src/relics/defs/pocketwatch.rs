//! Pocketwatch: Track cards played per turn. If <= 3 cards played,
//! draw 3 extra next turn.
//!
//! Counter increments on each card play. Turn-start logic checks previous
//! turn's counter and grants extra draw. complex_hook needed for the
//! turn-start draw decision.

use crate::effects::declarative::{Effect, SimpleEffect, Target, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static INIT_EFFECTS: [Effect; 2] = [
    Effect::Simple(SimpleEffect::SetStatus(Target::Player, sid::POCKETWATCH_COUNTER, AmountSource::Fixed(0))),
    Effect::Simple(SimpleEffect::SetStatus(Target::Player, sid::POCKETWATCH_FIRST_TURN, AmountSource::Fixed(1))),
];

static INCREMENT_EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::IncrementCounter(sid::POCKETWATCH_COUNTER, 1)),
];

static TRIGGERS: [TriggeredEffect; 2] = [
    TriggeredEffect {
        trigger: Trigger::CombatStart,
        condition: TriggerCondition::Always,
        effects: &INIT_EFFECTS,
        counter: None,
    },
    TriggeredEffect {
        trigger: Trigger::OnAnyCardPlayed,
        condition: TriggerCondition::Always,
        effects: &INCREMENT_EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Pocketwatch",
    name: "Pocketwatch",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None, // TODO: wire complex_hook for turn-start draw logic
    status_guard: None,
};
