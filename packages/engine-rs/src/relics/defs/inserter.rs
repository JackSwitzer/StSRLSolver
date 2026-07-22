//! Inserter: every two turns, gain one orb slot.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::{CombatEngine, TurnStartQueuedAction};

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    state: &mut EffectState,
) {
    if event.kind != Trigger::TurnStart {
        return;
    }
    let next = state.get(0) + 1;
    if next >= 2 {
        state.set(0, 0);
        // Inserter.java queues IncreaseMaxOrbAction(1), which mutates the live
        // orb collection and no-ops at Java's ten-slot cap.
        if engine.is_collecting_turn_start_actions() {
            engine.queue_turn_start_action_bottom(TurnStartQueuedAction::IncreaseOrbSlots(1));
        } else {
            let before = engine.state.orb_slots.max_slots;
            engine.state.orb_slots.add_slot();
            let gained = engine.state.orb_slots.max_slots.saturating_sub(before) as i32;
            if gained > 0 {
                engine
                    .state
                    .player
                    .add_status(crate::status_ids::sid::ORB_SLOTS, gained);
            }
        }
    } else {
        state.set(0, next);
    }
}

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "Inserter",
    name: "Inserter",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
