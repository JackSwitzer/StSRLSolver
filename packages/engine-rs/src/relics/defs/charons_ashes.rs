//! Charon's Ashes: Deal 3 damage to ALL enemies whenever a card is exhausted.

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
    // Source: reference/extracted/methods/relic/CharonsAshes.java. onExhaust
    // builds a pure three-damage matrix and resolves it as DamageType.THORNS.
    for idx in engine.state.living_enemy_indices() {
        engine.deal_thorns_damage_to_enemy(idx, 3);
    }
}

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnCardExhaust,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "Charon's Ashes",
    name: "Charon's Ashes",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
