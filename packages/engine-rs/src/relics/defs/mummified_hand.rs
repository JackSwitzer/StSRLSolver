//! Mummified Hand: On Power play, reduce a random hand card's cost to 0.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;

static TRIGGERS: [TriggeredEffect; 1] = [
    TriggeredEffect {
        trigger: Trigger::OnPowerPlayed,
        condition: TriggerCondition::Always,
        effects: &[],
        counter: None,
    },
];

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    if engine.state.hand.is_empty() {
        return;
    }
    let idx = engine.rng_gen_range(0..engine.state.hand.len());
    engine.state.hand[idx].cost = 0;
}

pub static DEF: EntityDef = EntityDef {
    id: "Mummified Hand",
    name: "Mummified Hand",
    kind: EntityKind::Relic,
    triggers: &TRIGGERS,
    complex_hook: Some(hook),
    status_guard: None,
};
