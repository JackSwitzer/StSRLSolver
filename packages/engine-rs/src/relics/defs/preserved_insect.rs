//! Preserved Insect: At elite fights, reduce strongest enemy HP by 25%.
//! Requires complex_hook to find strongest enemy and reduce HP.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition, TriggerContext};
use crate::engine::CombatEngine;
use crate::status_ids::sid;

fn hook(engine: &mut CombatEngine, _ctx: &TriggerContext) {
    // Only fire if elite flag is set (Python-side detection)
    if engine.state.player.status(sid::PRESERVED_INSECT_ELITE) == 0 {
        return;
    }
    // Find enemy with most HP
    if let Some(idx) = engine.state.enemies.iter()
        .enumerate()
        .filter(|(_, e)| e.is_alive())
        .max_by_key(|(_, e)| e.entity.hp)
        .map(|(i, _)| i)
    {
        let reduction = engine.state.enemies[idx].entity.hp / 4;
        engine.state.enemies[idx].entity.hp -= reduction;
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
    id: "PreservedInsect",
    name: "Preserved Insect",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
