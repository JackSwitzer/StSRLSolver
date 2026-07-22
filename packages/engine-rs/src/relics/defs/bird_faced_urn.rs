//! Bird-Faced Urn: Heal 2 HP whenever a Power card is played.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/BirdFacedUrn.java.

use crate::effects::declarative::{AmountSource, Effect, SimpleEffect, Target};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::HealHp(
    Target::Player,
    AmountSource::Fixed(2),
))];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnPowerPlayed,
    condition: TriggerCondition::Always,
    effects: &EFFECTS,
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "Bird Faced Urn",
    name: "Bird-Faced Urn",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
