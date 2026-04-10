//! modify_cost hooks — adjust effective card cost dynamically.

use crate::cards::CardDef;
use crate::combat_types::CardInstance;
use crate::powers;
use crate::state::CombatState;
use crate::status_ids::sid;

/// Blood for Blood: reduce cost by HP lost this combat.
pub fn hook_cost_reduce_on_hp_loss(state: &CombatState, _card: &CardDef, _card_inst: CardInstance, cost: i32) -> i32 {
    let hp_lost = state.player.status(sid::HP_LOSS_THIS_COMBAT);
    (cost - hp_lost).max(0)
}

/// Force Field: reduce cost by number of active powers on player.
pub fn hook_reduce_cost_per_power(state: &CombatState, _card: &CardDef, _card_inst: CardInstance, cost: i32) -> i32 {
    let power_count = powers::registry::count_active_powers(&state.player);
    (cost - power_count).max(0)
}

/// Eviscerate: reduce cost by cards discarded this turn.
pub fn hook_cost_reduce_on_discard(state: &CombatState, _card: &CardDef, _card_inst: CardInstance, cost: i32) -> i32 {
    let discarded = state.player.status(sid::DISCARDED_THIS_TURN);
    (cost - discarded).max(0)
}

/// Masterful Stab: increase cost by total damage taken this combat.
pub fn hook_cost_increase_on_hp_loss(state: &CombatState, _card: &CardDef, _card_inst: CardInstance, cost: i32) -> i32 {
    cost + state.total_damage_taken
}
