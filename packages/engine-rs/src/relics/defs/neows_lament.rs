//! Neow's Lament: Set all enemies to 1 HP at combat start while the blessing
//! counter remains above 0.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;
use crate::relic_flags::counter;

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if event.kind != Trigger::CombatStart {
        return;
    }

    let remaining = engine.state.relic_counters[counter::NEOWS_LAMENT];
    if remaining <= 0 {
        return;
    }

    let next = remaining - 1;
    if next <= 0 {
        engine.state.relic_counters[counter::NEOWS_LAMENT] = -2;
    } else {
        engine.state.relic_counters[counter::NEOWS_LAMENT] = next;
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
