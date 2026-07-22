//! Philosopher's Stone: +1 Strength to all enemies at combat start.
//! (Energy bonus is handled via max_energy on equip.)

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;
use crate::status_ids::sid;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::CombatStartDirect,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if event.kind != Trigger::CombatStartDirect {
        return;
    }
    // PhilosopherStone.atBattleStart calls AbstractMonster.addPower directly,
    // which appends without ApplyPowerAction's priority sort.
    // Java: relics/PhilosopherStone.java:38-42.
    for enemy_idx in engine.state.living_enemy_indices() {
        engine.state.enemies[enemy_idx]
            .entity
            .add_status_direct(sid::STRENGTH, 1);
    }
}

pub static DEF: EntityDef = EntityDef {
    id: "Philosopher's Stone",
    name: "Philosopher's Stone",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
