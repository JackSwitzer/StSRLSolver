use super::prelude::*;
use crate::engine::CombatEngine;
use crate::effects::trigger::TriggerContext;

/// Skill Potion: Add random Skill card to hand (cost 0 this turn).
/// complex_hook because it requires RNG to pick a random skill.
fn skill_potion_hook(_engine: &mut CombatEngine, _ctx: &TriggerContext) {
    // Stub: actual logic picks random skill from class pool, adds at cost 0
}

pub static DEF: EntityDef = EntityDef {
    id: "SkillPotion",
    name: "Skill Potion",
    kind: EntityKind::Potion,
    triggers: &[],
    complex_hook: Some(skill_potion_hook),
    status_guard: None,
};
