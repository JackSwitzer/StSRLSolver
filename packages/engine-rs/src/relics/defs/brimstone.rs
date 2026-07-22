//! Brimstone: +2 Strength to player, +1 Strength to all enemies at turn start.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::{CombatEngine, TurnStartQueuedAction};
use crate::status_ids::sid;

// Source: reference/extracted/methods/relic/Brimstone.java. Every turn start
// grants the player two Strength and each living monster one Strength.
static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::TurnStart,
    condition: TriggerCondition::Always,
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
        // Brimstone adds player Strength to top first, then each monster's
        // Strength to top. Repeated front insertion preserves Java's reverse
        // execution order for those ApplyPowerActions.
        engine
            .queue_turn_start_action_top(TurnStartQueuedAction::AddPlayerStatus(sid::STRENGTH, 2));
        for enemy_idx in 0..engine.state.enemies.len() {
            engine.queue_turn_start_action_top(TurnStartQueuedAction::AddEnemyStatus(
                enemy_idx,
                sid::STRENGTH,
                1,
            ));
        }
    } else {
        engine.state.player.add_status(sid::STRENGTH, 2);
        for enemy in &mut engine.state.enemies {
            if enemy.is_alive() {
                enemy.entity.add_status(sid::STRENGTH, 1);
            }
        }
    }
}

pub static DEF: EntityDef = EntityDef {
    id: "Brimstone",
    name: "Brimstone",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
