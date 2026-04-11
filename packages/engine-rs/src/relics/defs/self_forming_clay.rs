//! Self-Forming Clay: Gain 3 Block next turn whenever you lose HP.

use crate::effects::declarative::{Effect, SimpleEffect, Target, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::AddStatus(Target::Player, sid::NEXT_TURN_BLOCK, AmountSource::Fixed(3))),
];

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::OnPlayerHpLoss,
        condition: TriggerCondition::Always,
        effects: &EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Self Forming Clay",
    name: "Self-Forming Clay",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
};
