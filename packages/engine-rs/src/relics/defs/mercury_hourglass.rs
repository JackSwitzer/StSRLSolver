//! Mercury Hourglass: Deal 3 damage to ALL enemies at start of turn.
//!
//! Source: `reference/extracted/methods/relic/MercuryHourglass.java` uses a
//! pure damage matrix resolved as `DamageType.THORNS`.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    for idx in engine.state.living_enemy_indices() {
        engine.deal_thorns_damage_to_enemy(idx, 3);
    }
}

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::TurnStart,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "Mercury Hourglass",
    name: "Mercury Hourglass",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
