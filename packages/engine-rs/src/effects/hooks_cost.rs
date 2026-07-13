//! modify_cost hooks — adjust effective card cost dynamically.

use crate::cards::CardDef;
use crate::combat_types::CardInstance;
use crate::state::CombatState;
use crate::status_ids::sid;

/// Blood for Blood: reduce cost by positive damage events this combat.
pub fn hook_cost_reduce_on_hp_loss(state: &CombatState, _card: &CardDef, _card_inst: CardInstance, cost: i32) -> i32 {
    let damage_events = state.player.status(sid::HP_LOSS_THIS_COMBAT);
    (cost - damage_events).max(0)
}

/// Force Field: reduce cost once per Power card played this combat.
pub fn hook_reduce_cost_per_power(state: &CombatState, _card: &CardDef, card_inst: CardInstance, cost: i32) -> i32 {
    let baseline = card_inst.misc.max(0) as i32;
    (cost - (state.power_cards_played_this_combat - baseline).max(0)).max(0)
}
