//! Pantograph: Heal 25 HP at boss fight start.
//! Java: decompiled/java-src/com/megacrit/cardcrawl/relics/Pantograph.java

use crate::effects::declarative::{AmountSource, Effect, SimpleEffect, Target};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::HealHp(
    Target::Player,
    AmountSource::Fixed(25),
))];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::CombatStartTop,
    condition: TriggerCondition::IsBossFight,
    effects: &EFFECTS,
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "Pantograph",
    name: "Pantograph",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
