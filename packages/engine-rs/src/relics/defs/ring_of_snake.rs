//! Ring of the Snake: Draw 2 extra cards on turn 1.
//!
//! Same pattern as Bag of Preparation (Silent starter relic).
//! complex_hook needed: engine.draw_cards(2) requires engine access.

use crate::effects::declarative::{Effect, SimpleEffect, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::DrawCards(AmountSource::Fixed(2))),
];

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::TurnStart,
        condition: TriggerCondition::FirstTurn,
        effects: &EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Ring of the Snake",
    name: "Ring of the Snake",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None, // TODO: wire complex_hook for engine.draw_cards
    status_guard: None,
};
