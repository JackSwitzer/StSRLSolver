//! Red Skull: +3 Strength when HP <= 50% at combat start.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;
use crate::effects::trigger::TriggerContext;
use crate::status_ids::sid;

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::CombatStart,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
];

fn hook(engine: &mut CombatEngine, _ctx: &TriggerContext) {
    let hp = engine.state.player.hp;
    let max_hp = engine.state.player.max_hp;
    if hp <= max_hp / 2 {
        engine.state.player.add_status(sid::STRENGTH, 3);
    }
}

pub static DEF: EntityDef = EntityDef {
    id: "Red Skull",
    name: "Red Skull",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
