//! Horn Cleat: gain 14 Block on turn 2 only once per combat.

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
        Trigger::TurnStartPostDrawLate => {
            let counter = state.get(0);
            if (0..2).contains(&counter) {
                let next = counter + 1;
                if next == 2 {
                    engine.gain_block_player(14);
                    state.set(0, -1);
                } else {
                    state.set(0, next);
                }
            }
        }
        Trigger::CombatVictory => state.set(0, -1),
        _ => {}
    }
}

static TRIGGERS: [TriggeredEffect; 3] = [
    TriggeredEffect {
        trigger: Trigger::CombatStart,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
    TriggeredEffect {
        trigger: Trigger::TurnStartPostDrawLate,
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
    id: "HornCleat",
    name: "Horn Cleat",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
