//! Mark of Pain: Add 2 Wounds to draw pile at combat start.

use crate::effects::declarative::{AmountSource, Effect, SimpleEffect};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddCardToRandomDrawSpot(
    "Wound",
    AmountSource::Fixed(2),
))];

static TRIGGERS: [TriggeredEffect; 1] = [
    // MarkOfPain.java passes randomSpot=true to MakeTempCardInDrawPileAction,
    // so each Wound consumes cardRandomRng rather than shuffling the deck.
    TriggeredEffect {
        trigger: Trigger::CombatStart,
        condition: TriggerCondition::Always,
        effects: &EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Mark of Pain",
    name: "Mark of Pain",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
