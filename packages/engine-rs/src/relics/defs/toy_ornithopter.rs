//! Toy Ornithopter: Heal 5 HP whenever a potion is used.

use crate::effects::declarative::{Effect, SimpleEffect, Target, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::HealHp(Target::Player, AmountSource::Fixed(5))),
];

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::OnPotionUsed,
        condition: TriggerCondition::Always,
        effects: &EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Toy Ornithopter",
    name: "Toy Ornithopter",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
};
