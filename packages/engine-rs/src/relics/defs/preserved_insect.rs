//! Preserved Insect: At elite fights, reduce every enemy to 75% of max HP
//! if they currently exceed that threshold.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;
fn hook(
    engine: &mut CombatEngine,
    _owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    for enemy in &mut engine.state.enemies {
        if !enemy.is_alive() {
            continue;
        }
        let threshold = (enemy.entity.max_hp * 3) / 4;
        if enemy.entity.hp > threshold {
            enemy.entity.hp = threshold;
        }
    }
}

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::CombatStart,
        condition: TriggerCondition::IsEliteFight,
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
