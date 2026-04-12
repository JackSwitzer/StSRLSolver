use super::prelude::*;
use crate::engine::CombatEngine;
use crate::effects::trigger::TriggerContext;

/// Essence of Darkness: Channel Dark orbs equal to orb slots.
/// complex_hook because it reads max_slots and channels via engine.
fn essence_of_darkness_hook(_engine: &mut CombatEngine, _ctx: &TriggerContext) {
    // Stub: actual logic channels Dark * max_slots * potency
}

pub static DEF: EntityDef = EntityDef {
    id: "EssenceOfDarkness",
    name: "Essence of Darkness",
    kind: EntityKind::Potion,
    triggers: &[],
    complex_hook: Some(essence_of_darkness_hook),
    status_guard: None,
};
