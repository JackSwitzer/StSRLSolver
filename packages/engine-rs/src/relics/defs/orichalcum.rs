//! Orichalcum: If player has 0 Block at end of turn, gain 6 Block.

use crate::effects::declarative::{Effect, SimpleEffect, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::GainBlock(AmountSource::Fixed(6))),
];

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::TurnEnd,
        condition: TriggerCondition::NoBlock,
        effects: &EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Orichalcum",
    name: "Orichalcum",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
};
