//! Mutagenic Strength: +3 Strength and -3 at end of turn at combat start.
//!
//! Source: `reference/extracted/methods/relic/MutagenicStrength.java` —
//! `atBattleStart` applies 3 Strength and 3 LoseStrength to the player.

use crate::effects::declarative::{AmountSource, Effect, SimpleEffect, Target};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

// Java queues Strength then LoseStrength with addToTop, so the LIFO action
// manager applies LoseStrength first and Strength second.
static EFFECTS: [Effect; 2] = [
    Effect::Simple(SimpleEffect::AddStatus(
        Target::Player,
        sid::LOSE_STRENGTH,
        AmountSource::Fixed(3),
    )),
    Effect::Simple(SimpleEffect::AddStatus(
        Target::Player,
        sid::STRENGTH,
        AmountSource::Fixed(3),
    )),
];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::CombatStartTop,
    condition: TriggerCondition::Always,
    effects: &EFFECTS,
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "MutagenicStrength",
    name: "Mutagenic Strength",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
