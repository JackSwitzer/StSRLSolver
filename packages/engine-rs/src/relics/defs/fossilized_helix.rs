//! Fossilized Helix: 1 Buffer at combat start.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/FossilizedHelix.java.

use crate::effects::declarative::{AmountSource, Effect, SimpleEffect, Target};
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::status_ids::sid;

static EFFECTS: [Effect; 1] = [
    // FossilizedHelix.java::atBattleStart applies exactly one BufferPower.
    Effect::Simple(SimpleEffect::AddStatus(
        Target::Player,
        sid::BUFFER,
        AmountSource::Fixed(1),
    )),
];

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::CombatStart,
    condition: TriggerCondition::Always,
    effects: &EFFECTS,
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "FossilizedHelix",
    name: "Fossilized Helix",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: None,
    status_guard: None,
};
