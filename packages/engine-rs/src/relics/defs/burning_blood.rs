//! Burning Blood: Heal 6 HP at the end of combat.

use crate::effects::declarative::{Effect, SimpleEffect, Target, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::HealHp(Target::Player, AmountSource::Fixed(6))),
];

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::CombatVictory,
        condition: TriggerCondition::Always,
        effects: &EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Burning Blood",
    name: "Burning Blood",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
};
