//! Gremlin Horn: Gain 1 Energy and draw 1 card whenever an enemy dies.

use crate::effects::declarative::{Effect, SimpleEffect, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static EFFECTS: [Effect; 2] = [
    Effect::Simple(SimpleEffect::GainEnergy(AmountSource::Fixed(1))),
    Effect::Simple(SimpleEffect::DrawCards(AmountSource::Fixed(1))),
];

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::OnEnemyDeath,
        condition: TriggerCondition::Always,
        effects: &EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Gremlin Horn",
    name: "Gremlin Horn",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
};
