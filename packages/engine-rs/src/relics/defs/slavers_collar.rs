//! Slaver's Collar: +1 energy at elite, +3 energy at boss.
//! Requires complex_hook to read SLAVERS_COLLAR_ENERGY counter (set by Python).

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition, TriggerContext};
use crate::engine::CombatEngine;
use crate::status_ids::sid;

fn hook(engine: &mut CombatEngine, _ctx: &TriggerContext) {
    let energy_bonus = engine.state.player.status(sid::SLAVERS_COLLAR_ENERGY);
    if energy_bonus > 0 {
        engine.state.energy += energy_bonus;
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
    id: "SlaversCollar",
    name: "Slaver's Collar",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
