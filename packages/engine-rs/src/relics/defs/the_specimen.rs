//! The Specimen: Transfer Poison from a dead enemy to a random living enemy.

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
    let dead_idx = event.target_idx.max(0) as usize;
    if dead_idx >= engine.state.enemies.len() {
        return;
    }

    let dead_poison = engine.state.enemies[dead_idx].entity.status(sid::POISON);
    if dead_poison <= 0 {
        return;
    }

    let living: Vec<usize> = engine
        .state
        .enemies
        .iter()
        .enumerate()
        .filter(|(idx, enemy)| *idx != dead_idx && enemy.is_alive())
        .map(|(idx, _)| idx)
        .collect();

    if !living.is_empty() {
        let alive_idx = living[engine.rng_gen_range(0..living.len())];
        engine.state.enemies[alive_idx]
            .entity
            .add_status(sid::POISON, dead_poison);
    }
}

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::OnEnemyDeath,
        condition: TriggerCondition::Always,
        effects: &[], // complex_hook handles Poison transfer
        counter: None,
    },
];

pub static DEF: EntityDef = EntityDef {
    id: "The Specimen",
    name: "The Specimen",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
