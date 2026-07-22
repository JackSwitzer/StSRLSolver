//! Clockwork Souvenir: 1 Artifact at combat start.

use crate::effects::declarative::{AmountSource, Effect, SimpleEffect, Target};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static EFFECTS: [Effect; 1] = [Effect::Simple(SimpleEffect::AddStatus(
    Target::Player,
    sid::ARTIFACT,
    AmountSource::Fixed(1),
))];

static TRIGGERS: [TriggeredEffect; 1] = [
    // Source: reference/extracted/methods/relic/ClockworkSouvenir.java
    // atBattleStart applies exactly one ArtifactPower to the player.
    TriggeredEffect {
        trigger: Trigger::CombatStartTop,
        condition: TriggerCondition::Always,
        effects: &EFFECTS,
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "ClockworkSouvenir",
    name: "Clockwork Souvenir",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
