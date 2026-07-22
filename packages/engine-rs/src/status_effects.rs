//! Status card triggers — end-of-turn (Burn, Decay, Regret, Doubt, Shame)
//! and on-card-play (Pain).
//!
//! Extracted from engine.rs as a pure refactor.

use crate::effects::types::{CardRuntimeTrigger, EndTurnHandRule, WhileInHandRule};
use crate::engine::{CombatEngine, EndTurnQueuedAction};
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
/// Queue the ordinary actions produced by end-turn hand callbacks.
///
/// `TriggerEndOfTurnOrbsAction` is already in the action list when Java walks
/// the hand. Pride appends its `MakeTempCardInDrawPileAction` immediately after
/// that trigger, while status/curse autoplay goes to `cardQueue` instead.
/// When the orb trigger later executes, its child actions are therefore behind
/// Pride. A pre-trigger lethal action clears Pride; lethal orb damage does not.
/// Java: GameActionManager.java::callEndOfTurnActions,
/// cards/curses/Pride.java, actions/defect/TriggerEndOfTurnOrbsAction.java.
pub fn queue_end_turn_hand_ordinary_actions(engine: &mut CombatEngine) {
    let hand = engine.state.hand.clone();
    for card_inst in hand {
        let card = engine.card_registry.card_def_by_id(card_inst.def_id);
        if card.runtime_triggers().iter().any(|trigger| {
            matches!(
                trigger,
                CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::AddCopy)
            )
        }) {
            engine.queue_end_turn_action_bottom(
                EndTurnQueuedAction::MakeStatEquivalentCopyInDrawPile(card_inst),
            );
        }
    }
}

/// Drain the status/curse `CardQueueItem`s collected by Java's hand callbacks.
/// Ordinary Pride actions must already have drained through the shared action
/// queue before this function is called.
pub fn process_end_turn_card_queue(engine: &mut CombatEngine) -> bool {
    // GameActionManager.callEndOfTurnActions invokes
    // triggerOnEndOfTurnForPlayingCard over the original, unshuffled hand.
    // Retain/selfRetain cards are still in hand here and therefore count for
    // Regret. DiscardAtEndOfTurnAction removes retained cards and performs its
    // separate Collections.shuffle only after these card-queue items settle.
    // Java: actions/GameActionManager.java::callEndOfTurnActions;
    // actions/common/DiscardAtEndOfTurnAction.java; cards/curses/Regret.java.
    let hand = engine.state.hand.clone();
    let hand_size = hand.len() as i32;
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
                if *rule != EndTurnHandRule::AddCopy {
                    card_queue.push((*card_inst, *rule, card.base_magic));
                }
            }
        }
    }

    for (card_inst, rule, base_magic) in card_queue {
        // AbstractPlayer.useCard removes the autoplay CardQueueItem from hand
        // before the card's queued effect resolves. This opens a hand slot for
        // reactions such as Runic Cube's damage draw and remains true on a
        // lethal Burn. Regret still uses the callback-time hand_size above.
        // Java: AbstractPlayer.java::useCard, GameActionManager.java::getNextAction,
        // and UseCardAction.java::update.
        let auto_played = engine
            .state
            .hand
            .iter()
            .position(|candidate| *candidate == card_inst)
            .map(|hand_idx| engine.state.hand.remove(hand_idx));

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
                // Doubt constructs WeakPower(player, 1, true), so the debuff
                // must keep its justApplied latch through the immediately
                // following atEndOfRound pass.
                // Java: cards/curses/Doubt.java::use.
                powers::apply_debuff_from_enemy(&mut engine.state.player, sid::WEAKENED, 1);
            }
            EndTurnHandRule::Frail => {
                // Shame constructs FrailPower(player, 1, true), whose
                // justApplied flag skips this round's decrement.
                powers::apply_debuff_from_enemy(&mut engine.state.player, sid::FRAIL, 1);
            }
            EndTurnHandRule::AddCopy => unreachable!("Pride is an ordinary action"),
        }
        if let Some(auto_played) = auto_played {
            engine.state.discard_pile.push(auto_played);
        }
        if engine.state.combat_over {
            return true;
        }
    }

    false
}

/// Standalone compatibility helper used by focused card-runtime tests.
/// Full end-turn sequencing queues Pride before draining the card queue via
/// the two functions above.
pub fn process_end_turn_hand_cards(engine: &mut CombatEngine) -> bool {
    let hand = engine.state.hand.clone();
    for card_inst in hand {
        let card = engine.card_registry.card_def_by_id(card_inst.def_id);
        if card.runtime_triggers().iter().any(|trigger| {
            matches!(
                trigger,
                CardRuntimeTrigger::EndTurnInHand(EndTurnHandRule::AddCopy)
            )
        }) && !engine.state.is_victory()
        {
            let copy = engine.fresh_stat_copy(card_inst);
            engine.state.draw_pile.push(copy);
        }
    }
    process_end_turn_card_queue(engine)
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
