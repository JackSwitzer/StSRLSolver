//! Emotion Chip: after HP loss, trigger every orb's callbacks next turn.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::{CombatEngine, TurnStartQueuedAction};
use crate::status_ids::sid;

static TRIGGERS: [TriggeredEffect; 2] = [
    TriggeredEffect {
        trigger: Trigger::OnPlayerHpLoss,
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
];

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    match event.kind {
        Trigger::OnPlayerHpLoss => {
            // EmotionChip.java stores a boolean pulse, so repeated HP-loss
            // callbacks before the next atTurnStart schedule one ImpulseAction.
            engine.state.player.set_status(sid::EMOTION_CHIP_TRIGGER, 1);
        }
        Trigger::TurnStart if engine.state.player.status(sid::EMOTION_CHIP_TRIGGER) > 0 => {
            engine.state.player.set_status(sid::EMOTION_CHIP_TRIGGER, 0);
            if engine.is_collecting_turn_start_actions() {
                engine.queue_turn_start_action_bottom(TurnStartQueuedAction::TriggerOrbImpulse);
            } else {
                engine.trigger_orb_impulse();
            }
        }
        _ => {}
    }
}

pub static DEF: EntityDef = EntityDef {
    id: "Emotion Chip",
    name: "Emotion Chip",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
