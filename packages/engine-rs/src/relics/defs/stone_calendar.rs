//! Stone Calendar: deal 52 damage to all enemies at the end of turn 7.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    state: &mut EffectState,
) {
    match event.kind {
        Trigger::CombatStart => state.set(0, 0),
        Trigger::TurnStart => state.add(0, 1),
        Trigger::TurnEnd if state.get(0) == 7 => {
            for idx in engine.state.living_enemy_indices() {
                let enemy = &mut engine.state.enemies[idx];
                let blocked = enemy.entity.block.min(52);
                enemy.entity.block -= blocked;
                let hp_damage = 52 - blocked;
                enemy.entity.hp = (enemy.entity.hp - hp_damage).max(0);
                engine.state.total_damage_dealt += hp_damage;
            }
        }
        Trigger::CombatVictory => state.set(0, -1),
        _ => {}
    }
}

static TRIGGERS: [TriggeredEffect; 4] = [
    TriggeredEffect {
        trigger: Trigger::CombatStart,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
    TriggeredEffect {
        trigger: Trigger::TurnStart,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
    TriggeredEffect {
        trigger: Trigger::TurnEnd,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
    TriggeredEffect {
        trigger: Trigger::CombatVictory,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "StoneCalendar",
    name: "Stone Calendar",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
