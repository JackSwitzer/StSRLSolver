use super::prelude::*;
use crate::engine::CombatEngine;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

fn ambrosia_hook(
    engine: &mut CombatEngine,
    _owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    // Ambrosia.java queues ChangeStanceAction("Divinity"). Route through the
    // stance engine so Calm exit, Violet Lotus, Divinity energy, and power
    // hooks all resolve; the reported potency is not used by the action.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/potions/Ambrosia.java
    engine.change_stance(crate::state::Stance::Divinity);
}

pub static DEF: EntityDef = EntityDef {
    id: "Ambrosia",
    name: "Ambrosia",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(ambrosia_hook),
    status_guard: None,
};
