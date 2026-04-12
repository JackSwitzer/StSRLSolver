//! Holy Water: Add 3 HolyWater cards to hand at combat start.
//! Uses complex_hook because it needs engine.temp_card() and hand limit check.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition, TriggerContext};
use crate::engine::CombatEngine;

fn hook(engine: &mut CombatEngine, _ctx: &TriggerContext) {
    for _ in 0..3 {
        if engine.state.hand.len() < 10 {
            let card = engine.temp_card("HolyWater");
            engine.state.hand.push(card);
        }
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
    id: "HolyWater",
    name: "Holy Water",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
