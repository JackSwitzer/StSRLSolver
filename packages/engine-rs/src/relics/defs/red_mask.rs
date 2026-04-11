//! Red Mask: Apply 1 Weak to ALL enemies at combat start.

use crate::effects::declarative::{Effect, SimpleEffect, Target, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::AddStatus(Target::AllEnemies, sid::WEAKENED, AmountSource::Fixed(1))),
];

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::CombatStart,
        condition: TriggerCondition::Always,
        effects: &EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Red Mask",
    name: "Red Mask",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
};
