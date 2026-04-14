//! Emotion Chip: On HP loss, trigger front orb passive next turn.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;
use crate::status_ids::sid;

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::OnPlayerHpLoss,
        condition: TriggerCondition::Always,
        effects: &[], // complex_hook handles orb passive trigger
        counter: None,
    },
];

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    engine.state.player.set_status(sid::EMOTION_CHIP_TRIGGER, 1);
}

pub static DEF: EntityDef = EntityDef {
    id: "Emotion Chip",
    name: "Emotion Chip",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
