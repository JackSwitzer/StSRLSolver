use super::prelude::*;
use crate::engine::CombatEngine;
use crate::orbs::OrbType;

static TRIGGERS: [TriggeredEffect; 1] = [TriggeredEffect {
    trigger: Trigger::ManualActivation,
    condition: TriggerCondition::Always,
    effects: &[],
    counter: None,
}];

/// Essence of Darkness: Channel Dark orbs equal to orb slots.
/// complex_hook because it reads max_slots and channels via engine.
fn essence_of_darkness_hook(
    engine: &mut CombatEngine,
    _owner: crate::effects::runtime::EffectOwner,
    _event: &crate::effects::runtime::GameEvent,
    _state: &mut crate::effects::runtime::EffectState,
) {
    let slots = engine.state.orb_slots.get_slot_count();
    for _ in 0..slots {
        engine.channel_orb(OrbType::Dark);
    }
}

pub static DEF: EntityDef = EntityDef {
    id: "EssenceOfDarkness",
    name: "Essence of Darkness",
    kind: EntityKind::Potion,
    triggers: &TRIGGERS,
    complex_hook: Some(essence_of_darkness_hook),
    status_guard: None,
};
