//! Ninja Scroll: Add 3 Shivs to hand at combat start.

use crate::effects::declarative::{AmountSource, Effect, Pile, SimpleEffect};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddCard(
    "Shiv",
    Pile::Hand,
    AmountSource::Fixed(3),
))];

static TRIGGERS: [TriggeredEffect; 1] = [
    // NinjaScroll.java declares ID "Ninja Scroll" and creates exactly three
    // unupgraded Shivs before the opening draw.
    TriggeredEffect {
        trigger: Trigger::CombatStartPreDraw,
        condition: TriggerCondition::Always,
        effects: &EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Ninja Scroll",
    name: "Ninja Scroll",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
