//! Girya: +Strength based on lift count at combat start.
//! Requires complex_hook to read GIRYA_COUNTER (set by Python rest site).

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition, TriggerContext};
use crate::engine::CombatEngine;
use crate::status_ids::sid;

fn hook(engine: &mut CombatEngine, _ctx: &TriggerContext) {
    let lift_count = engine.state.player.status(sid::GIRYA_COUNTER);
    if lift_count > 0 {
        engine.state.player.add_status(sid::STRENGTH, lift_count);
    }
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
    id: "Girya",
    name: "Girya",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
