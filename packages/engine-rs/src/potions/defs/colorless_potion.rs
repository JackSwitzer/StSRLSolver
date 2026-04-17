use super::prelude::*;
use crate::engine::CombatEngine;
use crate::effects::declarative::{GeneratedCardPool, GeneratedCostRule, GeneratedUpgradeRule};

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn colorless_potion_hook(
    engine: &mut CombatEngine,
    _owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    crate::effects::interpreter::open_generated_discovery_choice_scaled(
        engine,
        GeneratedCardPool::Colorless,
        3,
        GeneratedCostRule::ZeroThisTurn,
        crate::potions::effective_potency_runtime(&engine.state, "ColorlessPotion") as usize,
        GeneratedUpgradeRule::Base,
    );
}

pub static DEF: EntityDef = EntityDef {
    id: "ColorlessPotion",
    name: "Colorless Potion",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(colorless_potion_hook),
    status_guard: None,
};
