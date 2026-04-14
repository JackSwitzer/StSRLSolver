use crate::state::CombatState;
// ==========================================================================
// DAMAGE / FOLLOW-UP HELPERS
// ==========================================================================

/// Strike Dummy: +3 damage on Strikes (simplified passive).
/// Unceasing Top: if hand is empty, draw 1.
pub fn unceasing_top_should_draw(state: &CombatState) -> bool {
    (state.has_relic("Unceasing Top") || state.has_relic("UnceasingTop"))
        && state.hand.is_empty()
        && (!state.draw_pile.is_empty() || !state.discard_pile.is_empty())
}

/// Runic Pyramid: don't discard hand at end of turn.
pub fn has_runic_pyramid(state: &CombatState) -> bool {
    state.has_relic("Runic Pyramid") || state.has_relic("RunicPyramid")
}
