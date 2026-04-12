//! Bag of Preparation: Draw 2 extra cards on turn 1.
//!
//! complex_hook needed: engine.draw_cards(2) requires engine access.
//! Old dispatch: sets BAG_OF_PREP_DRAW status, engine reads at draw time.

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
    id: "Bag of Preparation",
    name: "Bag of Preparation",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None, // TODO: wire complex_hook for engine.draw_cards
    status_guard: None,
};
