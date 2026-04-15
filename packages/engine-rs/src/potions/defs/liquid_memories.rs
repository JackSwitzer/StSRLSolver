use super::prelude::*;
use crate::engine::CombatEngine;
use crate::potions::effective_potency_runtime;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

/// Liquid Memories: Return card(s) from discard to hand (cost 0).
/// complex_hook because it must open a discard-choice selection and apply
/// a temporary zero-cost override to the chosen cards.
fn liquid_memories_hook(
    engine: &mut CombatEngine,
    _owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    let potency = effective_potency_runtime(&engine.state, "LiquidMemories");
    if potency <= 0 || engine.state.discard_pile.is_empty() {
        return;
    }

    let pick_count = potency as usize;
    if engine.state.discard_pile.len() <= pick_count {
        let moved = crate::potions::return_discard_to_hand(&mut engine.state, potency);
        if moved > 0 {
            let hand_len = engine.state.hand.len();
            let start = hand_len.saturating_sub(moved as usize);
            for card in &mut engine.state.hand[start..] {
                card.cost = 0;
            }
        }
        return;
    }

    let options = engine
        .state
        .discard_pile
        .iter()
        .enumerate()
        .map(|(idx, _)| crate::engine::ChoiceOption::DiscardCard(idx))
        .collect();
    engine.begin_choice(
        crate::engine::ChoiceReason::ReturnFromDiscard,
        options,
        pick_count,
        pick_count,
    );
    if let Some(choice) = engine.choice.as_mut() {
        choice.returned_card_cost_override = Some(0);
    }
}

pub static DEF: EntityDef = EntityDef {
    id: "LiquidMemories",
    name: "Liquid Memories",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(liquid_memories_hook),
    status_guard: None,
};
