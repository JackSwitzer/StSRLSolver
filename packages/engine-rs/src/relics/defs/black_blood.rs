//! Black Blood: Heal 12 HP at the end of combat.
//! (Upgraded Burning Blood.)

use crate::effects::declarative::{AmountSource, Effect, SimpleEffect, Target};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};

static EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::HealHp(
    Target::Player,
    AmountSource::Fixed(12),
))];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::CombatVictory,
    // Source: reference/extracted/methods/relic/BlackBlood.java. onVictory
    // heals twelve only when currentHealth is still positive.
    condition: TriggerCondition::PlayerAlive,
    effects: &EFFECTS,
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "Black Blood",
    name: "Black Blood",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
