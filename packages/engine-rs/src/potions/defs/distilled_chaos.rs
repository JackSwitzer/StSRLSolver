use super::prelude::*;
use crate::engine::CombatEngine;
use crate::effects::trigger::TriggerContext;

/// Distilled Chaos: Play top N cards from draw pile (3 base, 6 with Sacred Bark).
/// complex_hook because it must drive the card-play pipeline for each card.
fn distilled_chaos_hook(_engine: &mut CombatEngine, _ctx: &TriggerContext) {
    // Stub: actual logic calls play_top_cards_from_draw_pile(potency)
}

pub static DEF: EntityDef = EntityDef {
    id: "DistilledChaos",
    name: "Distilled Chaos",
    kind: EntityKind::Potion,
    triggers: &[],
    complex_hook: Some(distilled_chaos_hook),
    status_guard: None,
};
