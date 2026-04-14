//! Pocketwatch: If <= 3 cards were played last turn, draw 3 extra this turn.

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
        Trigger::CombatStart => {
            state.set(0, 0);
            state.set(1, 1);
        }
        Trigger::OnPlayCard => {
            state.add(0, 1);
        }
        Trigger::TurnStartPostDraw => {
            if engine.state.turn == 1 {
                state.set(1, 0);
            } else if state.get(1) > 0 {
                state.set(1, 0);
            } else if state.get(0) <= 3 {
                engine.draw_cards(3);
            }
            state.set(0, 0);
        }
        Trigger::CombatVictory => {
            state.set(0, -1);
        }
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
        trigger: Trigger::OnPlayCard,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
    TriggeredEffect {
        trigger: Trigger::TurnStartPostDraw,
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
    id: "Pocketwatch",
    name: "Pocketwatch",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
