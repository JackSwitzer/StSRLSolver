//! Gremlin Horn: Gain 1 Energy and draw 1 card when a non-final enemy dies.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/GremlinHorn.java.

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
    if engine.state.enemies.iter().any(|enemy| enemy.is_alive()) {
        engine.state.energy += 1;
        engine.draw_cards(1);
    }
}

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnEnemyDeath,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "Gremlin Horn",
    name: "Gremlin Horn",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
