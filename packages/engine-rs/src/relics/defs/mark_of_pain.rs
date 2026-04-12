//! Mark of Pain: Add 2 Wounds to draw pile at combat start.
//! Uses complex_hook because it needs card_registry.make_card().

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition, TriggerContext};
use crate::engine::CombatEngine;

fn hook(engine: &mut CombatEngine, _ctx: &TriggerContext) {
    let wound1 = engine.card_registry.make_card("Wound");
    let wound2 = engine.card_registry.make_card("Wound");
    engine.state.draw_pile.push(wound1);
    engine.state.draw_pile.push(wound2);
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
    id: "Mark of Pain",
    name: "Mark of Pain",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
