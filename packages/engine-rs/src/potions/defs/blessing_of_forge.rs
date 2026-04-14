use super::prelude::*;
use crate::engine::CombatEngine;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

/// Blessing of the Forge: Upgrade all cards in hand for combat.
/// complex_hook because it must iterate hand and modify card instances.
fn blessing_of_forge_hook(
    engine: &mut CombatEngine,
    _owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    crate::potions::upgrade_hand_for_combat(&mut engine.state);
}

pub static DEF: EntityDef = EntityDef {
    id: "BlessingOfTheForge",
    name: "Blessing of the Forge",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(blessing_of_forge_hook),
    status_guard: None,
};
