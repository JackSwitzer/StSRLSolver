use super::prelude::*;
use crate::engine::CombatEngine;
use crate::effects::trigger::TriggerContext;

/// Bottled Miracle: Add Miracle cards to hand (2 base, scaled by Sacred Bark).
/// complex_hook because MasterReality may upgrade the card.
fn bottled_miracle_hook(_engine: &mut CombatEngine, _ctx: &TriggerContext) {
    // Stub: actual logic adds Miracle (or Miracle+ with MasterReality) to hand
}

pub static DEF: EntityDef = EntityDef {
    id: "BottledMiracle",
    name: "Bottled Miracle",
    kind: EntityKind::Potion,
    triggers: &[],
    complex_hook: Some(bottled_miracle_hook),
    status_guard: None,
};
