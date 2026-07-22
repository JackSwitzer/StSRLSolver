//! Mummified Hand: On Power play, reduce a random valid hand card's cost to 0.
//!
//! Source: `reference/extracted/methods/relic/MummifiedHand.java`. Candidates
//! require base and turn cost above zero and must not be free-to-play; selection
//! consumes `cardRandomRng` even when only one candidate remains.

use crate::effects::entity_def::{EntityDef, EntityKind, TriggeredEffect};
use crate::effects::runtime::{EffectOwner, EffectState, GameEvent};
use crate::effects::trigger::{Trigger, TriggerCondition};
use crate::engine::CombatEngine;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::OnPowerPlayed,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn hook(
    engine: &mut CombatEngine,
    _owner: EffectOwner,
    _event: &GameEvent,
    _state: &mut EffectState,
) {
    let candidates: Vec<usize> = engine
        .state
        .hand
        .iter()
        .enumerate()
        .filter_map(|(idx, card)| {
            let def = engine.card_registry.card_def_by_id(card.def_id);
            let base_cost = if card.base_cost >= 0 {
                card.base_cost
            } else {
                def.cost as i8
            };
            let turn_cost = if card.cost >= 0 { card.cost } else { base_cost };
            (base_cost > 0 && turn_cost > 0 && !card.is_free()).then_some(idx)
        })
        .collect();
    if candidates.is_empty() {
        return;
    }
    let selected = engine
        .card_random_rng
        .random_int_range(0, (candidates.len() - 1) as i32) as usize;
    let idx = candidates[selected];
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
