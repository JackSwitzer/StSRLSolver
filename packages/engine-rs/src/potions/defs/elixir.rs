use super::prelude::*;
use crate::engine::CombatEngine;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

/// Elixir: choose any number of cards in hand to exhaust, including zero.
fn elixir_hook(
    engine: &mut CombatEngine,
    _owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    // Source: reference/extracted/methods/potion/Elixir.java. Its
    // ExhaustAction(false, true, true) is non-random, any-number, and permits
    // confirming zero cards; amount 99 only supplies the upper bound.
    let options: Vec<crate::engine::ChoiceOption> = (0..engine.state.hand.len())
        .map(crate::engine::ChoiceOption::HandCard)
        .collect();
    let max_picks = options.len();
    engine.begin_choice(
        crate::engine::ChoiceReason::ExhaustFromHand,
        options,
        0,
        max_picks,
    );
}

pub static DEF: EntityDef = EntityDef {
    id: "Elixir",
    name: "Elixir",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(elixir_hook),
    status_guard: None,
};
