//! Bag of Preparation: Draw 2 extra cards on turn 1.
//!
//! Runtime-backed via a first-turn declarative draw effect.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/BagOfPreparation.java.

use crate::effects::declarative::{AmountSource, Effect, SimpleEffect};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::DrawCards(
    AmountSource::Fixed(2),
))];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::CombatStart,
    condition: TriggerCondition::FirstTurn,
    effects: &EFFECTS,
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "Bag of Preparation",
    name: "Bag of Preparation",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
