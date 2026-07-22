//! Nilry's Codex: choose one of three random Watcher cards at end of turn.
//!
//! Sources: `reference/extracted/methods/relic/NilrysCodex.java` and
//! `decompiled/java-src/com/megacrit/cardcrawl/actions/unique/CodexAction.java`.

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
    if event.kind != Trigger::TurnEndPreCard || engine.state.living_enemy_indices().is_empty() {
        return;
    }
    // CodexAction is addToBot during relic callback collection. Card RNG is
    // consumed only when that queued action later executes.
    engine.queue_end_turn_action_bottom(EndTurnQueuedAction::NilrysCodex);
}

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnEndPreCard,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "Nilry's Codex",
    name: "Nilry's Codex",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
