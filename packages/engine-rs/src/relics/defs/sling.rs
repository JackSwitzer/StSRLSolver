//! Sling of Courage: +2 Strength if elite fight.

use crate::effects::declarative::{Effect, SimpleEffect, Target, AmountSource};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static EFFECTS: [Effect; 1] = [
    Effect::Simple(SimpleEffect::AddStatus(Target::Player, sid::STRENGTH, AmountSource::Fixed(2))),
];

static TRIGGERS: [TriggeredEffect; 1] = [
    // Source: reference/extracted/methods/relic/Sling.java
    // atBattleStart applies exactly 2 Strength only when eliteTrigger is set.
    TriggeredEffect {
        trigger: Trigger::CombatStart,
        condition: TriggerCondition::IsEliteFight,
        effects: &EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Sling",
    name: "Sling of Courage",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
