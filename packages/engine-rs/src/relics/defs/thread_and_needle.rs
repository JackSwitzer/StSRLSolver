//! Thread and Needle: 4 Plated Armor at combat start.

use crate::effects::declarative::{AmountSource, Effect, SimpleEffect, Target};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddStatus(
    Target::Player,
    sid::PLATED_ARMOR,
    AmountSource::Fixed(4),
))];

static TRIGGERS: [TriggeredEffect; 1] = [
    // Source: reference/extracted/methods/relic/ThreadAndNeedle.java
    // atBattleStart applies exactly 4 Plated Armor to the player.
    TriggeredEffect {
        trigger: Trigger::CombatStartTop,
        condition: TriggerCondition::Always,
        effects: &EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Thread and Needle",
    name: "Thread and Needle",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
