//! Centennial Puzzle: Draw 3 cards on first HP loss per combat.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    event: &GameEvent,
    state: &mut EffectState,
) {
    // Source: decompiled/java-src/com/megacrit/cardcrawl/relics/CentennialPuzzle.java
    // resets at pre-battle and draws exactly 3 cards on the first positive HP loss.
    match event.kind {
        Trigger::CombatSetup => {
            state.set(0, 1);
        }
        Trigger::OnPlayerHpLoss if state.get(0) > 0 => {
            state.set(0, 0);
            engine.draw_cards(3);
        }
        _ => {}
    }
}

static TRIGGERS: [TriggeredEffect; 2] = [
    TriggeredEffect {
        trigger: Trigger::CombatSetup,
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

pub static DEF: EntityDef = EntityDef {
    id: "Centennial Puzzle",
    name: "Centennial Puzzle",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
