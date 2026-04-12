//! Cracked Core: Channel 1 Lightning orb at combat start.
//! Uses complex_hook because channeling requires engine access.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition, TriggerContext};
use crate::engine::CombatEngine;
use crate::status_ids::sid;

fn hook(engine: &mut CombatEngine, _ctx: &TriggerContext) {
    // Set the flag for the deferred channel in start_combat()
    engine.state.player.set_status(sid::CHANNEL_LIGHTNING_START, 1);
}

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::CombatStart,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Cracked Core",
    name: "Cracked Core",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
