//! Cracked Core: Channel 1 Lightning orb at combat start.
//! Uses complex_hook because channeling requires engine access.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;
use crate::orbs::OrbType;

fn hook(
    engine: &mut CombatEngine,
    _owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    // Source: reference/extracted/methods/relic/CrackedCore.java. atPreBattle
    // channels immediately, preserving ownership order with other relics.
    engine.channel_orb(OrbType::Lightning);
}

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::CombatSetup,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "Cracked Core",
    name: "Cracked Core",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
