use crate::state::CombatState;
use crate::status_ids::sid;

// ==========================================================================
// DAMAGE / FOLLOW-UP HELPERS
// ==========================================================================

/// Champion's Belt: whenever applying Vulnerable, also apply 1 Weak.
pub fn champion_belt_on_vulnerable(state: &CombatState) -> bool {
    state.has_relic("Champion Belt")
}

/// Strike Dummy: +3 damage on Strikes (simplified passive).
pub fn strike_dummy_bonus(state: &CombatState) -> i32 {
    if state.has_relic("StrikeDummy") {
        3
    } else {
        0
    }
}

/// Wrist Blade: +4 damage on 0-cost attacks.
pub fn wrist_blade_bonus(state: &CombatState) -> i32 {
    if state.has_relic("WristBlade") {
        4
    } else {
        0
    }
}

/// Snecko Skull: +1 Poison when applying Poison.
pub fn snecko_skull_bonus(state: &CombatState) -> i32 {
    if state.has_relic("Snake Skull") || state.has_relic("SneckoSkull") {
        1
    } else {
        0
    }
}

/// Chemical X: +2 to X-cost effects.
pub fn chemical_x_bonus(state: &CombatState) -> i32 {
    if state.has_relic("Chemical X") || state.has_relic("ChemicalX") {
        2
    } else {
        0
    }
}

/// Apply Violet Lotus: +1 energy on Calm exit.
pub fn violet_lotus_calm_exit_bonus(state: &CombatState) -> i32 {
    if state.has_relic("Violet Lotus") || state.has_relic("VioletLotus") {
        1
    } else {
        0
    }
}

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

/// Calipers: retain up to 15 Block between turns.
pub fn calipers_block_retention(state: &CombatState, current_block: i32) -> i32 {
    if state.has_relic("Calipers") {
        current_block.min(15).max(0)
    } else {
        0
    }
}

/// Ice Cream: energy carries over between turns.
pub fn has_ice_cream(state: &CombatState) -> bool {
    state.has_relic("Ice Cream") || state.has_relic("IceCream")
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
