//! Lantern: +1 Energy on turn 1.
//!
//! Source: `reference/extracted/methods/relic/Lantern.java`
//! (`atPreBattle` arms it; the first `atTurnStart` queues one energy).

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::{CombatEngine, TurnStartQueuedAction};

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::FirstTurn,
    effects: &[],
    counter: None,
}];

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    if engine.is_collecting_turn_start_actions() {
        engine.queue_turn_start_action_top(TurnStartQueuedAction::GainEnergy(1));
    } else {
        engine.state.energy += 1;
    }
}

pub static DEF: EntityDef = EntityDef {
    id: "Lantern",
    name: "Lantern",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
