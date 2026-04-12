//! Snecko Eye: Set CONFUSION + SNECKO_EYE flags + 2 extra draw at combat start.
//! Requires complex_hook because it sets multiple interacting statuses.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition, TriggerContext};
use crate::engine::CombatEngine;
use crate::status_ids::sid;

fn hook(engine: &mut CombatEngine, _ctx: &TriggerContext) {
    engine.state.player.set_status(sid::SNECKO_EYE, 1);
    engine.state.player.set_status(sid::CONFUSION, 1);
    engine.state.player.set_status(sid::BAG_OF_PREP_DRAW, 2);
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
    id: "Snecko Eye",
    name: "Snecko Eye",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
