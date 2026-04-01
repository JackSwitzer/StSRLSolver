//! End-of-turn status card triggers (Burn, Decay, Regret, Doubt, Shame).
//!
//! Extracted from engine.rs `end_turn()` as a pure refactor.

use crate::cards::CardRegistry;
use crate::potions;
use crate::powers;
use crate::state::CombatState;

/// Process end-of-turn hand card triggers before discarding.
///
/// Handles: Burn/Burn+ damage, Decay damage, Regret (damage = hand size),
/// Doubt (apply Weak), Shame (apply Frail).
///
/// Returns `true` if the player died from status damage (combat should end).
pub fn process_end_turn_hand_cards(state: &mut CombatState, card_registry: &CardRegistry) -> bool {
    let hand = state.hand.clone();
    let hand_size = hand.len() as i32;
    for card_id in &hand {
        let card = card_registry.get_or_default(card_id);
        // Burn/Burn+/Decay: deal damage to player
        if card.effects.contains(&"end_turn_damage") && card.base_magic > 0 {
            state.player.hp -= card.base_magic;
            state.total_damage_taken += card.base_magic;
        }
        // Regret: deal damage equal to cards in hand
        if card.effects.contains(&"end_turn_regret") {
            state.player.hp -= hand_size;
            state.total_damage_taken += hand_size;
        }
        // Doubt: apply 1 Weak
        if card.effects.contains(&"end_turn_weak") {
            powers::apply_debuff(&mut state.player, "Weakened", card.base_magic.max(1));
        }
        // Shame: apply 1 Frail
        if card.effects.contains(&"end_turn_frail") {
            powers::apply_debuff(&mut state.player, "Frail", card.base_magic.max(1));
        }
    }

    // Check player death from status card damage
    if state.player.hp <= 0 {
        let revive_hp = potions::check_fairy_revive(state);
        if revive_hp > 0 {
            potions::consume_fairy(state);
            state.player.hp = revive_hp;
            false
        } else {
            state.player.hp = 0;
            state.combat_over = true;
            state.player_won = false;
            true // player died
        }
    } else {
        false
    }
}
