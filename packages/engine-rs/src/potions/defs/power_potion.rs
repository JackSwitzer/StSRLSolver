use super::prelude::*;
use crate::engine::CombatEngine;
use crate::effects::declarative::{GeneratedCardPool, GeneratedCostRule, GeneratedUpgradeRule};

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn power_potion_hook(
    engine: &mut CombatEngine,
    _owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    crate::effects::interpreter::open_generated_discovery_choice_scaled(
        engine,
        GeneratedCardPool::Power,
        3,
        GeneratedCostRule::ZeroThisTurn,
        crate::potions::effective_potency_runtime(&engine.state, "PowerPotion") as usize,
        GeneratedUpgradeRule::Base,
    );
}

pub static DEF: EntityDef = EntityDef {
    id: "PowerPotion",
    name: "Power Potion",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(power_potion_hook),
    status_guard: None,
};
