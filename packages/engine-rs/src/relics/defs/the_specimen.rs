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
        // ApplyPowerToRandomEnemyAction selects through cardRandomRng even
        // with one living target, then constructs a player-sourced
        // ApplyPowerAction. That shared constructor applies Snake Skull's +1
        // and lets Artifact block the whole Poison application.
        // Java: relics/TheSpecimen.java, actions/common/
        // ApplyPowerToRandomEnemyAction.java, and ApplyPowerAction.java.
        let selected = engine
            .card_random_rng
            .random_int_range(0, (living.len() - 1) as i32) as usize;
        let alive_idx = living[selected];
        engine.apply_player_debuff_to_enemy(alive_idx, sid::POISON, dead_poison);
    }
}

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnEnemyDeath,
    condition: TriggerCondition::Always,
    effects: &[], // complex_hook handles Poison transfer
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "The Specimen",
    name: "The Specimen",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
