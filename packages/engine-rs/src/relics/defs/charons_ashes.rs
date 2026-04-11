//! Charon's Ashes: Deal 3 damage to ALL enemies whenever a card is exhausted.

use crate::effects::declarative::{Effect, SimpleEffect, Target, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::DealDamage(Target::AllEnemies, AmountSource::Fixed(3))),
];

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::OnCardExhaust,
        condition: TriggerCondition::Always,
        effects: &EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Charon's Ashes",
    name: "Charon's Ashes",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
};
