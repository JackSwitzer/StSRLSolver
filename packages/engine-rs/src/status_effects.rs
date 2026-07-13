//! Status card triggers — end-of-turn (Burn, Decay, Regret, Doubt, Shame)
//! and on-card-play (Pain).
//!
//! Extracted from engine.rs as a pure refactor.

use crate::engine::CombatEngine;
use crate::effects::types::{CardRuntimeTrigger, EndTurnHandRule, WhileInHandRule};
use crate::powers;
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
pub fn process_end_turn_hand_cards(engine: &mut CombatEngine) -> bool {
    let hand = engine.state.hand.clone();
    let hand_size = hand.len() as i32;

    for card_inst in &hand {
        let card = engine.card_registry.card_def_by_id(card_inst.def_id);

        for trigger in card.runtime_triggers() {
            if let CardRuntimeTrigger::EndTurnInHand(rule) = trigger {
                match rule {
                    EndTurnHandRule::Damage => {
                        let raw = if card.base_magic > 0 { card.base_magic } else { 2 };
                        // Burn/Decay queue DamageAction with THORNS damage.
                        // Java: decompiled/java-src/com/megacrit/cardcrawl/cards/status/Burn.java
                        engine.deal_thorns_damage_to_player(raw);
                    }
                    EndTurnHandRule::Regret => {
                        engine.player_lose_hp_from_damage(hand_size);
                    }
                    EndTurnHandRule::Weak => {
                        powers::apply_debuff(&mut engine.state.player, sid::WEAKENED, 1);
                    }
                    EndTurnHandRule::Frail => {
                        powers::apply_debuff(&mut engine.state.player, sid::FRAIL, 1);
                    }
                    EndTurnHandRule::AddCopy => {
                        engine.state.draw_pile.push(*card_inst);
                    }
                }
            }
        }
        if engine.state.combat_over {
            return true;
        }
    }

    false
}

/// Process Pain curse triggers when ANY card is played.
///
/// Pain: deal 1 HP loss per Pain card in hand. This fires on every card play,
/// not on draw or end of turn. HP_LOSS type (bypasses block).
///
/// Returns `true` if the player died.
pub fn process_pain_on_card_play(engine: &mut CombatEngine) -> bool {
    // Count Pain cards currently in hand
    let pain_count = engine
        .state
        .hand
        .iter()
        .filter(|c| {
            let card = engine.card_registry.card_def_by_id(c.def_id);
            card.runtime_triggers().iter().any(|trigger| {
                matches!(
                    trigger,
                    CardRuntimeTrigger::WhileInHand(WhileInHandRule::PainOnOtherCardPlayed)
                )
            }) || engine.card_registry.card_name(c.def_id) == "Pain"
        })
        .count() as i32;

    // Each Pain queues its own LoseHPAction and therefore its own separately
    // mitigated AbstractPlayer.damage event.
    for _ in 0..pain_count {
        engine.player_lose_hp_from_damage(1);
        if engine.state.combat_over {
            return true;
        }
    }

    false
}
