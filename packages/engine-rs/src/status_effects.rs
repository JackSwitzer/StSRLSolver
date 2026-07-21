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
    // DiscardAtEndOfTurnAction first moves retain/selfRetain cards to limbo,
    // then snapshots the remaining hand for triggerOnEndOfPlayerTurn. Retained
    // cards therefore neither fire end-turn card hooks nor count toward
    // Regret's hand-size snapshot. Runic Pyramid and Equilibrium do not mark
    // their blanket-kept cards retained, so those cards remain in this list.
    // Java: decompiled/java-src/com/megacrit/cardcrawl/actions/common/
    // DiscardAtEndOfTurnAction.java
    let mut hand: Vec<_> = engine
        .state
        .hand
        .iter()
        .copied()
        .filter(|card_inst| {
            let card = engine.card_registry.card_def_by_id(card_inst.def_id);
            !card.runtime_traits().retain && !card_inst.is_retained()
        })
        .collect();
    // DiscardAtEndOfTurnAction clones this filtered hand and invokes the
    // no-argument Collections.shuffle before firing card-owned callbacks. That
    // shuffle uses Java's separate static default java.util.Random, not any
    // AbstractDungeon stream.
    engine.shuffle_end_turn_trigger_snapshot(&mut hand);
    let hand_size = hand.len() as i32;
    let mut ordinary_actions = Vec::new();
    let mut card_queue = Vec::new();

    // Card callbacks only enqueue work. GameActionManager drains ordinary
    // actions before cardQueue, regardless of the shuffled callback order.
    // Pride therefore creates its copy before a lethal Burn/Decay/Regret can
    // stop later card-queue items.
    // Java: GameActionManager.java::getNextAction, Pride.java, Burn.java.
    for card_inst in &hand {
        let card = engine.card_registry.card_def_by_id(card_inst.def_id);

        for trigger in card.runtime_triggers() {
            if let CardRuntimeTrigger::EndTurnInHand(rule) = trigger {
                if *rule == EndTurnHandRule::AddCopy {
                    ordinary_actions.push(*card_inst);
                } else {
                    card_queue.push((*card_inst, *rule, card.base_magic));
                }
            }
        }
    }

    for card_inst in ordinary_actions {
        // Pride passes false for randomSpot, so the copied card is added to the
        // top without consuming cardRandomRng.
        let copy = engine.fresh_stat_copy(card_inst);
        engine.state.draw_pile.push(copy);
    }

    for (_card_inst, rule, base_magic) in card_queue {
        match rule {
            EndTurnHandRule::Damage => {
                let raw = if base_magic > 0 { base_magic } else { 2 };
                // Burn/Decay autoplay through cardQueue, then queue a
                // player-owned THORNS DamageAction.
                engine.deal_self_thorns_damage_to_player(raw);
            }
            EndTurnHandRule::Regret => {
                // Regret snapshots the hand size during its callback, before
                // the queued card is eventually played.
                engine.player_lose_hp_from_damage(hand_size);
            }
            EndTurnHandRule::Weak => {
                powers::apply_debuff(&mut engine.state.player, sid::WEAKENED, 1);
            }
            EndTurnHandRule::Frail => {
                // Shame constructs FrailPower(player, 1, true), whose
                // justApplied flag skips this round's decrement.
                powers::apply_debuff_from_enemy(&mut engine.state.player, sid::FRAIL, 1);
            }
            EndTurnHandRule::AddCopy => unreachable!("Pride is an ordinary action"),
        }
        if engine.state.combat_over {
            return true;
        }
    }

    false
}

/// Process Pain curse triggers when ANY card is played.
///
/// Pain: deal 1 HP loss per Pain card in hand. Each trigger adds LoseHPAction
/// to the top after the played card queues its effects, so these resolve first.
/// Java: decompiled/java-src/com/megacrit/cardcrawl/cards/curses/Pain.java
/// Java: decompiled/java-src/com/megacrit/cardcrawl/characters/AbstractPlayer.java
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
