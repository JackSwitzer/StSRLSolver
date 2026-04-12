use super::prelude::*;
use crate::engine::CombatEngine;
use crate::effects::trigger::TriggerContext;

/// Blessing of the Forge: Upgrade all cards in hand for combat.
/// complex_hook because it must iterate hand and modify card instances.
fn blessing_of_forge_hook(_engine: &mut CombatEngine, _ctx: &TriggerContext) {
    // Stub: actual logic upgrades every non-upgraded card in hand
}

pub static DEF: EntityDef = EntityDef {
    id: "BlessingOfTheForge",
    name: "Blessing of the Forge",
    kind: EntityKind::Potion,
    triggers: &[],
    complex_hook: Some(blessing_of_forge_hook),
    status_guard: None,
};
