//! Ink Bottle: Every 10 cards played, draw 1 card.
//! Counter persists across combats.

use crate::effects::declarative::{Effect, SimpleEffect, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::DrawCards(AmountSource::Fixed(1))),
];

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::OnAnyCardPlayed,
        condition: TriggerCondition::CounterReached,
        effects: &EFFECTS,
        counter: Some((sid::INK_BOTTLE_COUNTER, 10)),
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "InkBottle",
    name: "Ink Bottle",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
