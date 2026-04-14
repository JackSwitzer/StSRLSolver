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
/// complex_hook because it must pop from discard and set cost overrides.
fn liquid_memories_hook(
    engine: &mut CombatEngine,
    _owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    let potency = effective_potency_runtime(&engine.state, "LiquidMemories");
    let moved = crate::potions::return_discard_to_hand(&mut engine.state, potency);
    if moved > 0 {
        let hand_len = engine.state.hand.len();
        let start = hand_len.saturating_sub(moved as usize);
        for card in &mut engine.state.hand[start..] {
            card.cost = 0;
        }
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
