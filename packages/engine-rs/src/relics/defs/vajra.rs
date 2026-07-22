//! Vajra: +1 Strength at combat start.

use crate::effects::declarative::{AmountSource, Effect, SimpleEffect, Target};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddStatus(
    Target::Player,
    sid::STRENGTH,
    AmountSource::Fixed(1),
))];

static TRIGGERS: [TriggeredEffect; 1] = [
    // Source: reference/extracted/methods/relic/Vajra.java
    // atBattleStart queues exactly one StrengthPower with addToTop. It
    // therefore resolves before first-turn atTurnStart actions such as
    // Damaru's addToBot MantraPower, independent of relic ownership order.
    TriggeredEffect {
        trigger: Trigger::CombatStartTop,
        condition: TriggerCondition::Always,
        effects: &EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Vajra",
    name: "Vajra",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
