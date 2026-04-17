//! Status card triggers — end-of-turn (Burn, Decay, Regret, Doubt, Shame)
//! and on-card-play (Pain).
//!
//! Extracted from engine.rs as a pure refactor.

use crate::cards::CardRegistry;
use crate::damage;
use crate::effects::types::{CardRuntimeTrigger, EndTurnHandRule, WhileInHandRule};
use crate::potions;
use crate::powers;
use crate::state::CombatState;
use crate::status_ids::sid;

/// Process end-of-turn hand card triggers before discarding.
///
/// Handles: Burn (2 dmg), Burn+ (4 dmg), Decay (2 dmg), Regret (hand-size HP loss),
/// Doubt (1 Weak), Shame (1 Frail).
///
/// Burn/Decay deal DAMAGE (goes through Block first, then HP).
/// Regret is HP_LOSS (bypasses Block, affected by Intangible/Tungsten Rod).
///
/// Returns `true` if the player died from status damage (combat should end).
pub fn process_end_turn_hand_cards(state: &mut CombatState, card_registry: &CardRegistry) -> bool {
    let hand = state.hand.clone();
    let hand_size = hand.len() as i32;

    let intangible = state.player.status(sid::INTANGIBLE) > 0;
    let tungsten = state.has_relic("Tungsten Rod") || state.has_relic("TungstenRod");

    for card_inst in &hand {
        let card = card_registry.card_def_by_id(card_inst.def_id);

        for trigger in card.runtime_triggers() {
            if let CardRuntimeTrigger::EndTurnInHand(rule) = trigger {
                match rule {
                    EndTurnHandRule::Damage => {
                        let raw = if card.base_magic > 0 { card.base_magic } else { 2 };
                        let mut dmg = raw;
                        if intangible && dmg > 1 {
                            dmg = 1;
                        }
                        let blocked = state.player.block.min(dmg);
                        let mut hp_damage = dmg - blocked;
                        state.player.block -= blocked;
                        if tungsten && hp_damage > 0 {
                            hp_damage = (hp_damage - 1).max(0);
                        }
                        if hp_damage > 0 {
                            state.player.hp -= hp_damage;
                            state.total_damage_taken += hp_damage;
                        }
                    }
                    EndTurnHandRule::Regret => {
                        let raw = hand_size;
                        let hp_loss = damage::apply_hp_loss(raw, intangible, tungsten);
                        if hp_loss > 0 {
                            state.player.hp -= hp_loss;
                            state.total_damage_taken += hp_loss;
                        }
                    }
                    EndTurnHandRule::Weak => {
                        powers::apply_debuff(&mut state.player, sid::WEAKENED, 1);
                    }
                    EndTurnHandRule::Frail => {
                        powers::apply_debuff(&mut state.player, sid::FRAIL, 1);
                    }
                    EndTurnHandRule::AddCopy => {
                        state.draw_pile.push(*card_inst);
                    }
                }
            }
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

/// Process Pain curse triggers when ANY card is played.
///
/// Pain: deal 1 HP loss per Pain card in hand. This fires on every card play,
/// not on draw or end of turn. HP_LOSS type (bypasses block).
///
/// Returns `true` if the player died.
pub fn process_pain_on_card_play(state: &mut CombatState, card_registry: &CardRegistry) -> bool {
    let intangible = state.player.status(sid::INTANGIBLE) > 0;
    let tungsten = state.has_relic("Tungsten Rod") || state.has_relic("TungstenRod");

    // Count Pain cards currently in hand
    let pain_count = state
        .hand
        .iter()
        .filter(|c| {
            let card = card_registry.card_def_by_id(c.def_id);
            card.runtime_triggers().iter().any(|trigger| {
                matches!(
                    trigger,
                    CardRuntimeTrigger::WhileInHand(WhileInHandRule::PainOnOtherCardPlayed)
                )
            }) || card_registry.card_name(c.def_id) == "Pain"
        })
        .count() as i32;

    if pain_count > 0 {
        let hp_loss_each = damage::apply_hp_loss(1, intangible, tungsten);
        let total_loss = hp_loss_each * pain_count;
        if total_loss > 0 {
            state.player.hp -= total_loss;
            state.total_damage_taken += total_loss;
        }
    }

    // Check player death
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
            true
        }
    } else {
        false
    }
}
