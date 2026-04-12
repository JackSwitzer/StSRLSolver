//! Du-Vu Doll: +1 Strength per curse in deck at combat start.
//! Requires complex_hook to read DU_VU_DOLL_CURSES counter (set by Python).

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition, TriggerContext};
use crate::engine::CombatEngine;
use crate::status_ids::sid;

fn hook(engine: &mut CombatEngine, _ctx: &TriggerContext) {
    let curse_count = engine.state.player.status(sid::DU_VU_DOLL_CURSES);
    if curse_count > 0 {
        engine.state.player.add_status(sid::STRENGTH, curse_count);
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
    id: "Du-Vu Doll",
    name: "Du-Vu Doll",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
