use crate::state::CombatState;
use crate::status_ids::sid;

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

/// Necronomicon: first 2+-cost attack per turn plays twice.
pub fn necronomicon_should_trigger(state: &CombatState, card_cost: i32, is_attack: bool) -> bool {
    if !state.has_relic("Necronomicon") {
        return false;
    }
    is_attack && card_cost >= 2 && state.player.status(sid::NECRONOMICON_USED) == 0
}

/// Mark Necronomicon as used for this turn.
pub fn necronomicon_mark_used(state: &mut CombatState) {
    state.player.set_status(sid::NECRONOMICON_USED, 1);
}

/// Reset Necronomicon at turn start.
pub fn necronomicon_reset(state: &mut CombatState) {
    if state.has_relic("Necronomicon") {
        state.player.set_status(sid::NECRONOMICON_USED, 0);
    }
}
