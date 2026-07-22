//! Cloak Clasp: At end of turn, gain 1 Block per card in hand.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::{CombatEngine, EndTurnQueuedAction};

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if event.kind == Trigger::TurnEndPreCard && !engine.state.hand.is_empty() {
        engine.queue_end_turn_action_bottom(EndTurnQueuedAction::GainBlock(
            engine.state.hand.len() as i32,
        ));
    }
}

static TRIGGERS: [TriggeredEffect; 1] = [
    // Source: reference/extracted/methods/relic/CloakClasp.java
    // (`onPlayerEndTurn`: current hand size * 1).
    TriggeredEffect {
        trigger: Trigger::TurnEndPreCard,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "CloakClasp",
    name: "Cloak Clasp",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
