//! Red Skull: +3 Strength when HP <= 50% at combat start.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::engine::CombatEngine;
use crate::status_ids::sid;

static TRIGGERS: [TriggeredEffect; 2] = [
    TriggeredEffect {
        trigger: Trigger::CombatStart,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
    TriggeredEffect {
        trigger: Trigger::OnPlayerHpLoss,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
];

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    state: &mut EffectState,
) {
    let hp = engine.state.player.hp;
    let max_hp = engine.state.player.max_hp;
    if event.kind == Trigger::CombatStart {
        state.set(0, 0);
    }

    let active = state.get(0) > 0 && hp <= max_hp / 2;
    if hp <= max_hp / 2 && !active {
        engine.state.player.add_status(sid::STRENGTH, 3);
        state.set(0, 1);
    } else if hp > max_hp / 2 && active {
        engine.state.player.add_status(sid::STRENGTH, -3);
        state.set(0, 0);
    }
}

pub static DEF: EntityDef = EntityDef {
    id: "Red Skull",
    name: "Red Skull",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
