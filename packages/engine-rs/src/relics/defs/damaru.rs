//! Damaru: Gain 1 Mantra at the start of each turn.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::{CombatEngine, TurnStartQueuedAction};

static TRIGGERS: [TriggeredEffect; 1] = [
    // Source: reference/extracted/methods/relic/Damaru.java (`atTurnStart`).
    TriggeredEffect {
        trigger: Trigger::TurnStart,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
];

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    if engine.is_collecting_turn_start_actions() {
        engine.queue_turn_start_action_bottom(TurnStartQueuedAction::GainMantra(1));
    } else {
        engine.gain_mantra(1);
    }
}

pub static DEF: EntityDef = EntityDef {
    id: "Damaru",
    name: "Damaru",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
