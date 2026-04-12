//! Pure Water: Add 1 Miracle to hand at combat start.
//! Uses complex_hook because it needs engine.temp_card().

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition, TriggerContext};
use crate::engine::CombatEngine;

fn hook(engine: &mut CombatEngine, _ctx: &TriggerContext) {
    let card = engine.temp_card("Miracle");
    engine.state.hand.push(card);
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
    id: "PureWater",
    name: "Pure Water",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
