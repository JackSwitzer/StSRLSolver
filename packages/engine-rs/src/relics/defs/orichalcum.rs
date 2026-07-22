//! Orichalcum: If player has 0 Block at end of turn, gain 6 Block.

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
    if event.kind == Trigger::TurnEndPreCard {
        // Orichalcum uses addToTop, so its block resolves ahead of every
        // bottom-queued relic/power action after all callbacks are collected.
        // Java: relics/Orichalcum.java::onPlayerEndTurn.
        engine.queue_end_turn_action_top(EndTurnQueuedAction::GainBlock(6));
    }
}

static TRIGGERS: [TriggeredEffect; 1] = [
    // Source: reference/extracted/methods/relic/Orichalcum.java
    // onPlayerEndTurn queues exactly 6 Block only when currentBlock is zero.
    TriggeredEffect {
        trigger: Trigger::TurnEndPreCard,
        condition: TriggerCondition::NoBlock,
        effects: &[],
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Orichalcum",
    name: "Orichalcum",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
