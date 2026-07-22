use super::prelude::*;
use crate::effects::declarative::{GeneratedCardPool, GeneratedCostRule, GeneratedUpgradeRule};
use crate::engine::CombatEngine;

// Java: decompiled/java-src/com/megacrit/cardcrawl/potions/AttackPotion.java
// and actions/unique/DiscoveryAction.java. Present three unique base Watcher
// attacks, then add potency copies of the selected card at zero cost this turn.

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn attack_potion_hook(
    engine: &mut CombatEngine,
    _owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    crate::effects::interpreter::open_generated_discovery_choice_scaled(
        engine,
        GeneratedCardPool::Attack,
        3,
        GeneratedCostRule::ZeroThisTurn,
        crate::potions::effective_potency_runtime(&engine.state, "AttackPotion") as usize,
        GeneratedUpgradeRule::Base,
    );
}

pub static DEF: EntityDef = EntityDef {
    id: "AttackPotion",
    name: "Attack Potion",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(attack_potion_hook),
    status_guard: None,
};
