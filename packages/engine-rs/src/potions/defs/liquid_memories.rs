use super::prelude::*;
use crate::engine::CombatEngine;
use crate::effects::trigger::TriggerContext;

/// Liquid Memories: Return card(s) from discard to hand (cost 0).
/// complex_hook because it must pop from discard and set cost overrides.
fn liquid_memories_hook(_engine: &mut CombatEngine, _ctx: &TriggerContext) {
    // Stub: actual logic returns card(s) from discard pile to hand at cost 0
}

pub static DEF: EntityDef = EntityDef {
    id: "LiquidMemories",
    name: "Liquid Memories",
    kind: EntityKind::Potion,
    triggers: &[],
    complex_hook: Some(liquid_memories_hook),
    status_guard: None,
};
