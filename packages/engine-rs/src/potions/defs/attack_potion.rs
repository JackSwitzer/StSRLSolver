use super::prelude::*;
use crate::engine::CombatEngine;
use crate::effects::trigger::TriggerContext;

/// Attack Potion: Add random Attack card to hand (cost 0 this turn).
/// complex_hook because it requires RNG to pick a random attack.
fn attack_potion_hook(_engine: &mut CombatEngine, _ctx: &TriggerContext) {
    // Stub: actual logic picks random attack from class pool, adds at cost 0
}

pub static DEF: EntityDef = EntityDef {
    id: "AttackPotion",
    name: "Attack Potion",
    kind: EntityKind::Potion,
    triggers: &[],
    complex_hook: Some(attack_potion_hook),
    status_guard: None,
};
