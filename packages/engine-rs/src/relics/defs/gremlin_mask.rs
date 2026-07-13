//! Gremlin Mask: apply 1 Weak to the player at combat start.
//!
//! Source: `reference/extracted/methods/relic/GremlinMask.java` —
//! `atBattleStart` queues `WeakPower(player, 1, false)` through ApplyPowerAction.

use crate::effects::declarative::{AmountSource, Effect, SimpleEffect, Target};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddStatus(
    Target::Player,
    sid::WEAKENED,
    AmountSource::Fixed(1),
))];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::CombatStart,
    condition: TriggerCondition::Always,
    effects: &EFFECTS,
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "GremlinMask",
    name: "Gremlin Mask",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
