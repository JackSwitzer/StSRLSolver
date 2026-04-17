//! Neow's Lament: Set all enemies to 1 HP at combat start while the blessing
//! counter remains above 0.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;
use crate::status_ids::sid;

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if event.kind != Trigger::CombatStart {
        return;
    }

    let counter = engine.state.player.status(sid::NEOWS_LAMENT_COUNTER);
    if counter <= 0 {
        return;
    }

    let next = counter - 1;
    if next <= 0 {
        engine.state.player.set_status(sid::NEOWS_LAMENT_COUNTER, -2);
    } else {
        engine.state.player.set_status(sid::NEOWS_LAMENT_COUNTER, next);
    }

    for enemy in &mut engine.state.enemies {
        enemy.entity.hp = 1;
    }
}

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::CombatStart,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "NeowsBlessing",
    name: "Neow's Lament",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
