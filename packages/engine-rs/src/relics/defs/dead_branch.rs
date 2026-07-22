//! Dead Branch: generate one random non-healing class card per exhaust.
//! Source: decompiled/java-src/com/megacrit/cardcrawl/relics/DeadBranch.java.

use crate::effects::declarative::GeneratedCardPool;
use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    _state: &mut EffectState,
) {
    if event.kind != Trigger::OnCardExhaust || engine.state.living_enemy_indices().is_empty() {
        return;
    }
    if let Some(card) =
        crate::effects::interpreter::generate_random_card(engine, GeneratedCardPool::WatcherAny)
    {
        if engine.state.hand.len() < 10 {
            engine.state.hand.push(card);
        } else {
            engine.state.discard_pile.push(card);
        }
    }
}

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnCardExhaust,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

pub static DEF: EntityDef = EntityDef {
    id: "Dead Branch",
    name: "Dead Branch",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
